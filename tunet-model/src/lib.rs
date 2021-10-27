use futures_util::{pin_mut, TryStreamExt};
use std::borrow::Cow;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc::*;
use tunet_rust::{usereg::*, *};
use tunet_settings::*;

pub type UpdateCallback = Box<dyn Fn(UpdateMsg) + 'static>;

pub struct Model {
    update: Option<UpdateCallback>,
    tx: Sender<Action>,
    pub cred: Mutex<Arc<NetCredential>>,
    pub http: HttpClient,
    pub state: NetState,
    pub log: Mutex<Cow<'static, str>>,
    pub log_busy: Arc<AtomicBool>,
    pub online_busy: Arc<AtomicBool>,
    pub detail_busy: Arc<AtomicBool>,
    pub flux: Mutex<NetFlux>,
    pub users: Mutex<Vec<NetUser>>,
    pub details: Mutex<Vec<NetDetail>>,
}

impl Model {
    pub fn new(tx: Sender<Action>) -> Result<Self> {
        Self::new_impl(None, tx)
    }

    pub fn with_callback(update: UpdateCallback, tx: Sender<Action>) -> Result<Self> {
        Self::new_impl(Some(update), tx)
    }

    fn new_impl(update: Option<UpdateCallback>, tx: Sender<Action>) -> Result<Self> {
        let cred = FileSettingsReader::new()
            .and_then(|reader| reader.read_with_password())
            .unwrap_or_default();
        let http = create_http_client()?;

        Ok(Self {
            update,
            tx,
            cred: Mutex::new(Arc::new(cred)),
            http,
            state: NetState::Unknown,
            log: Mutex::new(Cow::default()),
            log_busy: Arc::new(AtomicBool::new(false)),
            online_busy: Arc::new(AtomicBool::new(false)),
            detail_busy: Arc::new(AtomicBool::new(false)),
            flux: Mutex::new(NetFlux::default()),
            users: Mutex::new(Vec::default()),
            details: Mutex::new(Vec::default()),
        })
    }

    pub fn handle(&self, action: Action) {
        match action {
            Action::Login => {
                *self.log.lock().unwrap() = "正在登录".into();
                self.spawn_login();
            }
            Action::Logout => {
                *self.log.lock().unwrap() = "正在注销".into();
                self.spawn_logout();
            }
            Action::Flux => {
                *self.log.lock().unwrap() = "正在刷新流量".into();
                self.spawn_flux();
            }
            Action::LoginDone(s) | Action::LogoutDone(s) => {
                *self.log.lock().unwrap() = s.into();
                self.update(UpdateMsg::Log);
            }
            Action::FluxDone(f, s) => {
                *self.log.lock().unwrap() = s.into();
                self.update(UpdateMsg::Log);
                *self.flux.lock().unwrap() = f;
                self.update(UpdateMsg::Flux);
            }
            Action::Online => {
                self.spawn_online();
            }
            Action::OnlineDone(us) => {
                *self.users.lock().unwrap() = us;
                self.update(UpdateMsg::Online);
            }
            Action::Detail => {
                self.spawn_details();
            }
            Action::DetailDone(ds) => {
                *self.details.lock().unwrap() = ds;
                self.update(UpdateMsg::Detail);
            }
        }
    }

    pub fn update(&self, msg: UpdateMsg) {
        if let Some(f) = &self.update {
            f(msg)
        }
    }

    fn client(&self) -> Option<TUNetConnect> {
        TUNetConnect::new_nosuggest(
            self.state,
            self.cred.lock().unwrap().clone(),
            self.http.clone(),
        )
        .ok()
    }

    fn usereg(&self) -> UseregHelper {
        UseregHelper::new(self.cred.lock().unwrap().clone(), self.http.clone())
    }

    fn spawn_login(&self) {
        let mut lock = BusyGuard::new(self.log_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            if let Some(client) = self.client() {
                tokio::spawn(async move {
                    let _lock = lock;
                    let res = client.login().await;
                    let ok = res.is_ok();
                    tx.send(Action::LoginDone(res.unwrap_or_else(|e| e.to_string())))
                        .await?;
                    if ok {
                        Self::flux_impl(client, tx).await?;
                    }
                    Ok::<_, anyhow::Error>(())
                });
            }
        }
    }

    fn spawn_logout(&self) {
        let mut lock = BusyGuard::new(self.log_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            if let Some(client) = self.client() {
                tokio::spawn(async move {
                    let _lock = lock;
                    let res = client.logout().await;
                    let ok = res.is_ok();
                    tx.send(Action::LoginDone(res.unwrap_or_else(|e| e.to_string())))
                        .await?;
                    if ok {
                        Self::flux_impl(client, tx).await?;
                    }
                    Ok::<_, anyhow::Error>(())
                });
            }
        }
    }

    fn spawn_flux(&self) {
        let mut lock = BusyGuard::new(self.log_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            if let Some(client) = self.client() {
                tokio::spawn(async move {
                    let _lock = lock;
                    Self::flux_impl(client, tx).await
                });
            }
        }
    }

    async fn flux_impl(client: TUNetConnect, tx: Sender<Action>) -> Result<()> {
        let flux = client.flux().await;
        match flux {
            Ok(flux) => {
                tx.send(Action::FluxDone(flux, String::default())).await?;
            }
            Err(err) => {
                tx.send(Action::FluxDone(NetFlux::default(), err.to_string()))
                    .await?
            }
        }
        Ok(())
    }

    fn spawn_online(&self) {
        let mut lock = BusyGuard::new(self.online_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            let usereg = self.usereg();
            tokio::spawn(async move {
                let _lock = lock;
                usereg.login().await?;
                let users = usereg.users();
                pin_mut!(users);
                tx.send(Action::OnlineDone(users.try_collect().await?))
                    .await?;
                Ok::<_, anyhow::Error>(())
            });
        }
    }

    fn spawn_details(&self) {
        let mut lock = BusyGuard::new(self.detail_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            let usereg = self.usereg();
            tokio::spawn(async move {
                let _lock = lock;
                usereg.login().await?;
                let details = usereg.details(NetDetailOrder::LogoutTime, false);
                pin_mut!(details);
                tx.send(Action::DetailDone(details.try_collect().await?))
                    .await?;
                Ok::<_, anyhow::Error>(())
            });
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Login,
    LoginDone(String),
    Logout,
    LogoutDone(String),
    Flux,
    FluxDone(NetFlux, String),
    Online,
    OnlineDone(Vec<NetUser>),
    Detail,
    DetailDone(Vec<NetDetail>),
}

pub enum UpdateMsg {
    Log,
    Flux,
    Online,
    Detail,
}

struct BusyGuard {
    lock: Arc<AtomicBool>,
    locked: bool,
}

impl BusyGuard {
    pub fn new(lock: Arc<AtomicBool>) -> Self {
        Self {
            lock,
            locked: false,
        }
    }

    pub fn lock(&mut self) -> bool {
        self.locked = self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok();
        self.locked
    }
}

impl Drop for BusyGuard {
    fn drop(&mut self) {
        if self.locked {
            self.lock.store(false, Ordering::Release);
        }
    }
}
