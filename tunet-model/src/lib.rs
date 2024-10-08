#![forbid(unsafe_code)]

use anyhow::Result;
use compio::runtime::spawn;
use drop_guard::guard;
use flume::Sender;
use futures_util::{pin_mut, StreamExt, TryStreamExt};
use itertools::Itertools;
use mac_address2::{MacAddress, MacAddressIterator};
use netstatus::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tunet_helper::{usereg::*, *};

#[derive(Debug, Default)]
pub struct DetailDaily {
    pub details: Vec<(NaiveDate, Flux)>,
    pub now: NaiveDate,
    pub max_flux: Flux,
}

impl DetailDaily {
    pub fn new(details: &[NetDetail]) -> Self {
        let details = details
            .iter()
            .chunk_by(|d| d.logout_time.date())
            .into_iter()
            .map(|(key, group)| (key.day(), group.map(|d| d.flux.0).sum::<u64>()))
            .collect::<HashMap<_, _>>();
        let mut grouped_details = vec![];
        let now = Local::now().date_naive();
        let mut max = 0;
        for d in 1u32..=now.day() {
            if let Some(f) = details.get(&d) {
                max += *f;
            }
            grouped_details.push((now.with_day(d).unwrap(), Flux(max)))
        }
        Self {
            details: grouped_details,
            now,
            max_flux: Flux(max),
        }
    }
}

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
    online_busy: BusyBool,
    detail_busy: BusyBool,
    pub flux: NetFlux,
    pub users: Vec<NetUser>,
    pub details: Vec<NetDetail>,
    pub mac_addrs: Vec<MacAddress>,
}

impl Model {
    pub fn new(action_sender: Sender<Action>, update_sender: Sender<UpdateMsg>) -> Result<Self> {
        let http = create_http_client();

        let mac_addrs = MacAddressIterator::new()
            .map(|it| it.collect::<Vec<_>>())
            .unwrap_or_default();

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
            online_busy: BusyBool::new(action_sender.clone(), UpdateMsg::OnlineBusy),
            detail_busy: BusyBool::new(action_sender, UpdateMsg::DetailBusy),
            flux: NetFlux::default(),
            users: Vec::default(),
            details: Vec::default(),
            mac_addrs,
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
            Action::Online => {
                self.spawn_online();
            }
            Action::OnlineDone(us) => {
                self.users = us;
                self.update(UpdateMsg::Online);
            }
            Action::Connect(addr) => {
                let action_sender = self.action_sender.clone();
                let usereg = self.usereg();
                let (u, p) = (self.username.clone(), self.password.clone());
                spawn(async move {
                    match usereg.login(&u, &p).await {
                        Ok(_) => {
                            usereg.connect(addr).await?;
                            action_sender.send_async(Action::Online).await?;
                        }
                        Err(e) => {
                            action_sender
                                .send_async(Action::LogDone(e.to_string()))
                                .await?;
                        }
                    }
                    anyhow::Ok(())
                })
                .detach();
            }
            Action::Drop(addr) => {
                let action_sender = self.action_sender.clone();
                let usereg = self.usereg();
                let (u, p) = (self.username.clone(), self.password.clone());
                spawn(async move {
                    match usereg.login(&u, &p).await {
                        Ok(_) => {
                            usereg.drop(addr).await?;
                            action_sender.send_async(Action::Online).await?;
                        }
                        Err(e) => {
                            action_sender
                                .send_async(Action::LogDone(e.to_string()))
                                .await?;
                        }
                    }
                    anyhow::Ok(())
                })
                .detach();
            }
            Action::Details => {
                self.spawn_details();
            }
            Action::DetailsDone(ds) => {
                self.details = ds;
                self.update(UpdateMsg::Details);
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

    fn usereg(&self) -> UseregHelper {
        UseregHelper::new(self.http.clone())
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

    fn spawn_online(&self) {
        if let Some(lock) = self.online_busy.lock() {
            let action_sender = self.action_sender.clone();
            let usereg = self.usereg();
            let (u, p) = (self.username.clone(), self.password.clone());
            spawn(async move {
                let _lock = lock;
                match usereg.login(&u, &p).await {
                    Ok(_) => {
                        let users = usereg.users();
                        pin_mut!(users);
                        action_sender
                            .send_async(Action::OnlineDone(users.try_collect().await?))
                            .await?;
                    }
                    Err(e) => {
                        action_sender
                            .send_async(Action::LogDone(e.to_string()))
                            .await?;
                    }
                }
                anyhow::Ok(())
            })
            .detach();
        }
    }

    fn spawn_details(&self) {
        if let Some(lock) = self.detail_busy.lock() {
            let action_sender = self.action_sender.clone();
            let usereg = self.usereg();
            let (u, p) = (self.username.clone(), self.password.clone());
            spawn(async move {
                let _lock = lock;
                match usereg.login(&u, &p).await {
                    Ok(_) => {
                        let details = usereg.details(NetDetailOrder::LogoutTime, false);
                        pin_mut!(details);
                        action_sender
                            .send_async(Action::DetailsDone(details.try_collect().await?))
                            .await?;
                    }
                    Err(e) => {
                        action_sender
                            .send_async(Action::LogDone(e.to_string()))
                            .await?;
                    }
                }
                anyhow::Ok(())
            })
            .detach();
        }
    }

    pub fn log_busy(&self) -> bool {
        self.log_busy.get()
    }

    pub fn online_busy(&self) -> bool {
        self.online_busy.get()
    }

    pub fn detail_busy(&self) -> bool {
        self.detail_busy.get()
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
    Online,
    OnlineDone(Vec<NetUser>),
    Connect(Ipv4Addr),
    Drop(Ipv4Addr),
    Details,
    DetailsDone(Vec<NetDetail>),
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
    Online,
    Details,
    LogBusy,
    OnlineBusy,
    DetailBusy,
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
