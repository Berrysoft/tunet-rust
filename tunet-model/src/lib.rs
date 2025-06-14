#![forbid(unsafe_code)]

use anyhow::Result;
use compio::runtime::spawn;
use drop_guard::guard;
use flume::Sender;
use futures_util::StreamExt;
use netstatus::*;
use std::borrow::Cow;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tunet_helper::*;

pub struct Model {
    action_sender: Sender<Action>,
    update_sender: Sender<UpdateMsg>,
    pub username: String,
    password: String,
    pub http: HttpClient,
    pub state: NetState,
    pub status: NetStatus,
    pub log: Cow<'static, str>,
    log_busy: BusyBool,
    pub flux: NetFlux,
}

impl Model {
    pub fn new(action_sender: Sender<Action>, update_sender: Sender<UpdateMsg>) -> Result<Self> {
        let http = create_http_client();

        Ok(Self {
            action_sender: action_sender.clone(),
            update_sender,
            username: String::default(),
            password: String::default(),
            http,
            state: NetState::Unknown,
            status: NetStatus::Unknown,
            log: Cow::default(),
            log_busy: BusyBool::new(action_sender.clone(), UpdateMsg::LogBusy),
            flux: NetFlux::default(),
        })
    }

    pub fn queue(&self, action: Action) {
        self.action_sender.send(action).ok();
    }

    pub fn handle(&mut self, action: Action) {
        match action {
            Action::Credential(u, p) => {
                self.username = u;
                self.password = p;
                self.update(UpdateMsg::Credential);
            }
            Action::State(s) => {
                match s {
                    None => {
                        let action_sender = self.action_sender.clone();
                        let http = self.http.clone();
                        let status = self.status.clone();
                        spawn(async move {
                            let state = suggest::suggest_with_status(&http, &status).await;
                            action_sender
                                .send_async(Action::State(Some(state)))
                                .await
                                .ok()
                        })
                        .detach();
                    }
                    Some(s) => {
                        self.state = s;
                        self.update(UpdateMsg::State);
                    }
                };
            }
            Action::WatchStatus => {
                self.spawn_watch_status();
            }
            Action::Status(status) => {
                let status = status.unwrap_or_else(NetStatus::current);
                self.status = status;
                self.update(UpdateMsg::Status);
            }
            Action::Timer => {
                self.spawn_timer();
            }
            Action::Tick => {
                if !self.flux.username.is_empty() {
                    self.flux.online_time =
                        Duration(self.flux.online_time.0 + NaiveDuration::try_seconds(1).unwrap());
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
                self.spawn_flux();
            }
            Action::LogDone(s) => {
                self.log = s.into();
                self.update(UpdateMsg::Log);
            }
            Action::FluxDone(f, s, keep) => {
                if keep {
                    if let Some(s) = s {
                        self.log = s.into();
                        self.update(UpdateMsg::Log);
                    }
                } else {
                    self.log = s.unwrap_or_default().into();
                    self.update(UpdateMsg::Log);
                }
                self.flux = f;
                self.update(UpdateMsg::Flux);
            }
            Action::Update(msg) => {
                self.update(msg);
            }
        }
    }

    pub fn update(&self, msg: UpdateMsg) {
        let update_sender = self.update_sender.clone();
        spawn(async move { update_sender.send_async(msg).await }).detach();
    }

    fn spawn_watch_status(&self) {
        let action_sender = self.action_sender.clone();
        spawn(async move {
            let mut events = NetStatus::watch();
            while let Some(()) = events.next().await {
                action_sender.send_async(Action::Status(None)).await?;
            }
            anyhow::Ok(())
        })
        .detach();
    }

    fn spawn_timer(&self) {
        let action_sender = self.action_sender.clone();
        spawn(async move {
            let mut interval = compio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                action_sender.send_async(Action::Tick).await?;
            }
            #[allow(unreachable_code)]
            anyhow::Ok(())
        })
        .detach();
    }

    fn client(&self) -> Option<TUNetConnect> {
        TUNetConnect::new(self.state, self.http.clone()).ok()
    }

    fn spawn_login(&self) {
        if let Some(lock) = self.log_busy.lock() {
            let action_sender = self.action_sender.clone();
            let (u, p) = (self.username.clone(), self.password.clone());
            if let Some(client) = self.client() {
                spawn(async move {
                    let _lock = lock;
                    let res = client.login(&u, &p).await;
                    let ok = res.is_ok();
                    action_sender
                        .send_async(Action::LogDone(res.unwrap_or_else(|e| e.to_string())))
                        .await?;
                    if ok {
                        compio::time::sleep(std::time::Duration::from_secs(1)).await;
                        Self::flux_impl(client, action_sender, true).await?;
                    }
                    anyhow::Ok(())
                })
                .detach();
            }
        }
    }

    fn spawn_logout(&self) {
        if let Some(lock) = self.log_busy.lock() {
            let action_sender = self.action_sender.clone();
            let u = self.username.clone();
            if let Some(client) = self.client() {
                spawn(async move {
                    let _lock = lock;
                    let res = client.logout(&u).await;
                    let ok = res.is_ok();
                    action_sender
                        .send_async(Action::LogDone(res.unwrap_or_else(|e| e.to_string())))
                        .await?;
                    if ok {
                        Self::flux_impl(client, action_sender, true).await?;
                    }
                    anyhow::Ok(())
                })
                .detach();
            }
        }
    }

    fn spawn_flux(&self) {
        if let Some(lock) = self.log_busy.lock() {
            let action_sender = self.action_sender.clone();
            if let Some(client) = self.client() {
                spawn(async move {
                    let _lock = lock;
                    Self::flux_impl(client, action_sender, false).await
                })
                .detach();
            }
        }
    }

    async fn flux_impl(
        client: TUNetConnect,
        action_sender: Sender<Action>,
        keep_msg: bool,
    ) -> Result<()> {
        let flux = client.flux().await;
        match flux {
            Ok(flux) => {
                action_sender
                    .send_async(Action::FluxDone(flux, None, keep_msg))
                    .await?;
            }
            Err(err) => {
                action_sender
                    .send_async(Action::FluxDone(
                        NetFlux::default(),
                        Some(err.to_string()),
                        keep_msg,
                    ))
                    .await?
            }
        }
        Ok(())
    }

    pub fn log_busy(&self) -> bool {
        self.log_busy.get()
    }
}

#[derive(Debug)]
pub enum Action {
    Credential(String, String),
    State(Option<NetState>),
    WatchStatus,
    Status(Option<NetStatus>),
    Timer,
    Tick,
    Login,
    Logout,
    LogDone(String),
    Flux,
    FluxDone(NetFlux, Option<String>, bool),
    Update(UpdateMsg),
}

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum UpdateMsg {
    Credential,
    State,
    Status,
    Log,
    Flux,
    LogBusy,
}

struct BusyBool {
    lock: Arc<AtomicBool>,
    action_sender: Sender<Action>,
    msg: UpdateMsg,
}

impl BusyBool {
    pub fn new(action_sender: Sender<Action>, msg: UpdateMsg) -> Self {
        Self {
            lock: Arc::new(AtomicBool::new(false)),
            action_sender,
            msg,
        }
    }

    pub fn get(&self) -> bool {
        self.lock.load(Ordering::Acquire)
    }

    pub fn lock(&self) -> Option<impl Drop> {
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            let msg = self.msg;
            let action_sender = self.action_sender.clone();
            spawn(async move {
                action_sender.send_async(Action::Update(msg)).await.ok();
            })
            .detach();
            Some(guard(
                (self.lock.clone(), self.action_sender.clone(), self.msg),
                |(lock, action_sender, msg)| {
                    lock.store(false, Ordering::Release);
                    spawn(async move {
                        action_sender.send_async(Action::Update(msg)).await.ok();
                    })
                    .detach();
                },
            ))
        } else {
            None
        }
    }
}
