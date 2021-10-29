#![forbid(unsafe_code)]

use futures_util::{pin_mut, TryStreamExt};
use mac_address::*;
use netstatus::*;
use std::borrow::Cow;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc::*;
use tunet_rust::{usereg::*, *};

pub type UpdateCallback = Box<dyn Fn(UpdateMsg) + Send + Sync + 'static>;

pub struct Model {
    update: Option<UpdateCallback>,
    tx: Sender<Action>,
    pub cred: Arc<NetCredential>,
    pub http: HttpClient,
    pub state: NetState,
    pub status: NetStatus,
    pub log: Cow<'static, str>,
    pub log_busy: Arc<AtomicBool>,
    pub online_busy: Arc<AtomicBool>,
    pub detail_busy: Arc<AtomicBool>,
    pub flux: NetFlux,
    pub users: Vec<NetUser>,
    pub details: Vec<NetDetail>,
    pub mac_addrs: Vec<MacAddress>,
}

impl Model {
    pub fn new(tx: Sender<Action>) -> Result<Self> {
        let http = create_http_client()?;

        let mac_addrs = MacAddressIterator::new()
            .map(|it| it.collect::<Vec<_>>())
            .unwrap_or_default();

        Ok(Self {
            update: None,
            tx,
            cred: Arc::new(NetCredential::default()),
            http,
            state: NetState::Unknown,
            status: NetStatus::current(),
            log: Cow::default(),
            log_busy: Arc::new(AtomicBool::new(false)),
            online_busy: Arc::new(AtomicBool::new(false)),
            detail_busy: Arc::new(AtomicBool::new(false)),
            flux: NetFlux::default(),
            users: Vec::default(),
            details: Vec::default(),
            mac_addrs,
        })
    }

    pub fn set_callback(&mut self, update: Option<UpdateCallback>) {
        self.update = update;
    }

    pub fn queue(&self, action: Action) {
        let tx = self.tx.clone();
        tokio::spawn(async move { tx.send(action).await.ok() });
    }

    pub fn handle(&mut self, action: Action) {
        match action {
            Action::Credential(cred) => self.cred = cred,
            Action::State(s) => {
                let s = s.unwrap_or(NetState::Auto);
                match s {
                    NetState::Auto => {
                        let tx = self.tx.clone();
                        let http = self.http.clone();
                        let status = self.status.clone();
                        tokio::spawn(async move {
                            let state = suggest::suggest_with_status(&http, status).await;
                            tx.send(Action::State(Some(state))).await.ok()
                        });
                    }
                    _ => self.state = s,
                };
            }
            Action::Timer => {
                self.spawn_timer();
            }
            Action::Tick => {
                if !self.flux.username.is_empty() {
                    self.flux.online_time =
                        Duration(self.flux.online_time.0 + NaiveDuration::seconds(1));
                    self.update(UpdateMsg::Flux);
                }
            }
            Action::Login => {
                self.log = "正在登录".into();
                self.update(UpdateMsg::Log);
                self.spawn_login();
            }
            Action::Logout => {
                self.log = "正在注销".into();
                self.update(UpdateMsg::Log);
                self.spawn_logout();
            }
            Action::Flux => {
                self.log = "正在刷新流量".into();
                self.update(UpdateMsg::Log);
                self.spawn_flux();
            }
            Action::LoginDone(s) | Action::LogoutDone(s) => {
                self.log = s.into();
                self.update(UpdateMsg::Log);
            }
            Action::FluxDone(f, s) => {
                self.log = s.into();
                self.update(UpdateMsg::Log);
                self.flux = f;
                self.update(UpdateMsg::Flux);
            }
            Action::Online => {
                self.spawn_online();
            }
            Action::OnlineDone(us) => {
                self.users = us;
                self.update(UpdateMsg::Online);
            }
            Action::Details => {
                self.spawn_details();
            }
            Action::DetailsDone(ds) => {
                self.details = ds;
                self.update(UpdateMsg::Details);
            }
        }
    }

    pub fn update(&self, msg: UpdateMsg) {
        if let Some(f) = &self.update {
            f(msg)
        }
    }

    fn spawn_timer(&self) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                tx.send(Action::Tick).await?;
            }
            #[allow(unreachable_code)]
            Ok::<_, anyhow::Error>(())
        });
    }

    fn client(&self) -> Option<TUNetConnect> {
        TUNetConnect::new_nosuggest(self.state, self.cred.clone(), self.http.clone()).ok()
    }

    fn usereg(&self) -> UseregHelper {
        UseregHelper::new(self.cred.clone(), self.http.clone())
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
                tx.send(Action::DetailsDone(details.try_collect().await?))
                    .await?;
                Ok::<_, anyhow::Error>(())
            });
        }
    }

    pub fn log_busy(&self) -> bool {
        self.log_busy.load(Ordering::Acquire)
    }

    pub fn online_busy(&self) -> bool {
        self.online_busy.load(Ordering::Acquire)
    }

    pub fn detail_busy(&self) -> bool {
        self.detail_busy.load(Ordering::Acquire)
    }
}

#[derive(Debug)]
pub enum Action {
    Credential(Arc<NetCredential>),
    State(Option<NetState>),
    Timer,
    Tick,
    Login,
    LoginDone(String),
    Logout,
    LogoutDone(String),
    Flux,
    FluxDone(NetFlux, String),
    Online,
    OnlineDone(Vec<NetUser>),
    Details,
    DetailsDone(Vec<NetDetail>),
}

#[repr(i32)]
pub enum UpdateMsg {
    Log,
    Flux,
    Online,
    Details,
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
