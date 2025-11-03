#![forbid(unsafe_code)]

use anyhow::Result;
use async_once_cell::OnceCell;
use compio::runtime::spawn;
use drop_guard::guard;
use futures_util::StreamExt;
use netstatus::*;
use std::borrow::Cow;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tunet_helper::*;
use winio_elm::{Component, ComponentSender};

pub struct Model {
    pub username: String,
    password: String,
    pub http: Arc<OnceCell<HttpClient>>,
    pub state: NetState,
    pub status: NetStatus,
    pub log: Cow<'static, str>,
    log_busy: BusyBool,
    pub flux: NetFlux,
}

async fn http_client(this: &OnceCell<HttpClient>) -> Result<&HttpClient> {
    Ok(this.get_or_try_init(create_http_client()).await?)
}

impl Component for Model {
    type Init<'a> = ();
    type Message = Action;
    type Event = UpdateMsg;

    fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        Self {
            username: String::default(),
            password: String::default(),
            http: Arc::new(OnceCell::new()),
            state: NetState::Unknown,
            status: NetStatus::Unknown,
            log: Cow::default(),
            log_busy: BusyBool::new(sender.clone(), UpdateMsg::LogBusy),
            flux: NetFlux::default(),
        }
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        std::future::pending().await
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            Action::Credential(u, p) => {
                self.username = u;
                self.password = p;
                sender.output(UpdateMsg::Credential);
            }
            Action::State(s) => {
                match s {
                    None => {
                        let sender = sender.clone();
                        let http = self.http.clone();
                        let status = self.status.clone();
                        spawn(async move {
                            let http = http_client(&http).await?;
                            let state = suggest::suggest_with_status(http, &status).await;
                            sender.post(Action::State(Some(state)));
                            anyhow::Ok(())
                        })
                        .detach();
                    }
                    Some(s) => {
                        self.state = s;
                        sender.output(UpdateMsg::State);
                    }
                };
            }
            Action::WatchStatus => {
                self.spawn_watch_status(sender);
            }
            Action::Status(status) => {
                let status = status.unwrap_or_else(NetStatus::current);
                self.status = status;
                sender.output(UpdateMsg::Status);
            }
            Action::Timer => {
                self.spawn_timer(sender);
            }
            Action::Tick => {
                if !self.flux.username.is_empty() {
                    self.flux.online_time =
                        Duration(self.flux.online_time.0 + NaiveDuration::try_seconds(1).unwrap());
                    sender.output(UpdateMsg::Flux);
                }
            }
            Action::Login => {
                self.log = "正在登录".into();
                sender.output(UpdateMsg::Log);
                self.spawn_login(sender);
            }
            Action::Logout => {
                self.log = "正在注销".into();
                sender.output(UpdateMsg::Log);
                self.spawn_logout(sender);
            }
            Action::Flux => {
                self.spawn_flux(sender);
            }
            Action::LogDone(s) => {
                self.log = s.into();
                sender.output(UpdateMsg::Log);
            }
            Action::FluxDone(f, s, keep) => {
                if keep {
                    if let Some(s) = s {
                        self.log = s.into();
                        sender.output(UpdateMsg::Log);
                    }
                } else {
                    self.log = s.unwrap_or_default().into();
                    sender.output(UpdateMsg::Log);
                }
                self.flux = f;
                sender.output(UpdateMsg::Flux);
            }
        }
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

impl Model {
    fn spawn_watch_status(&self, sender: &ComponentSender<Self>) {
        let sender = sender.clone();
        spawn(async move {
            let mut events = NetStatus::watch();
            while let Some(()) = events.next().await {
                sender.post(Action::Status(None));
            }
            anyhow::Ok(())
        })
        .detach();
    }

    fn spawn_timer(&self, sender: &ComponentSender<Self>) {
        let sender = sender.clone();
        spawn(async move {
            let mut interval = compio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                sender.post(Action::Tick);
            }
        })
        .detach();
    }

    fn spawn_login(&self, sender: &ComponentSender<Self>) {
        if let Some(lock) = self.log_busy.lock() {
            let sender = sender.clone();
            let (u, p) = (self.username.clone(), self.password.clone());
            let http = self.http.clone();
            let state = self.state;
            spawn(async move {
                let _lock = lock;
                let http = http_client(&http).await?;
                let client = TUNetConnect::new(state, http.clone())?;
                let res = client.login(&u, &p).await;
                let ok = res.is_ok();
                sender.post(Action::LogDone(res.unwrap_or_else(|e| e.to_string())));
                if ok {
                    compio::time::sleep(std::time::Duration::from_secs(1)).await;
                    Self::flux_impl(client, &sender, true).await;
                }
                anyhow::Ok(())
            })
            .detach();
        }
    }

    fn spawn_logout(&self, sender: &ComponentSender<Self>) {
        if let Some(lock) = self.log_busy.lock() {
            let sender = sender.clone();
            let u = self.username.clone();
            let http = self.http.clone();
            let state = self.state;
            spawn(async move {
                let _lock = lock;
                let http = http_client(&http).await?;
                let client = TUNetConnect::new(state, http.clone())?;
                let res = client.logout(&u).await;
                let ok = res.is_ok();
                sender.post(Action::LogDone(res.unwrap_or_else(|e| e.to_string())));
                if ok {
                    compio::time::sleep(std::time::Duration::from_secs(1)).await;
                    Self::flux_impl(client, &sender, true).await;
                }
                anyhow::Ok(())
            })
            .detach();
        }
    }

    fn spawn_flux(&self, sender: &ComponentSender<Self>) {
        if let Some(lock) = self.log_busy.lock() {
            let sender = sender.clone();
            let http = self.http.clone();
            let state = self.state;
            spawn(async move {
                let _lock = lock;
                let http = http_client(&http).await?;
                let client = TUNetConnect::new(state, http.clone())?;
                Self::flux_impl(client, &sender, false).await;
                anyhow::Ok(())
            })
            .detach();
        }
    }

    async fn flux_impl(client: TUNetConnect, sender: &ComponentSender<Self>, keep_msg: bool) {
        let flux = client.flux().await;
        match flux {
            Ok(flux) => sender.post(Action::FluxDone(flux, None, keep_msg)),
            Err(err) => sender.post(Action::FluxDone(
                NetFlux::default(),
                Some(err.to_string()),
                keep_msg,
            )),
        }
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
    sender: ComponentSender<Model>,
    msg: UpdateMsg,
}

impl BusyBool {
    pub fn new(sender: ComponentSender<Model>, msg: UpdateMsg) -> Self {
        Self {
            lock: Arc::new(AtomicBool::new(false)),
            sender,
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
            let sender = self.sender.clone();
            sender.output(msg);
            Some(guard(
                (self.lock.clone(), self.sender.clone(), self.msg),
                |(lock, sender, msg)| {
                    lock.store(false, Ordering::Release);
                    sender.output(msg);
                },
            ))
        } else {
            None
        }
    }
}
