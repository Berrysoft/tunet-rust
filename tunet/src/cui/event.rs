pub use crossterm::event::Event as TerminalEvent;
use futures_util::{pin_mut, Stream, StreamExt};
use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};
use tokio::sync::mpsc::*;
use tunet_rust::{usereg::*, *};

#[derive(Debug)]
pub enum EventType {
    TerminalEvent(TerminalEvent),
    Tick,
    Log(LogType),
    LogDone(LogType),
    Flux(NetFlux),
    ClearOnline,
    AddOnline(NetUser),
    ClearDetail,
    AddDetail(NetDetail),
}

#[derive(Debug)]
pub enum LogType {
    Login(Option<String>),
    Logout(Option<String>),
    Flux,
    Online,
    Detail,
}

pub struct Event {
    client: TUNetConnect,
    usereg: UseregHelper,
    log_busy: Arc<AtomicBool>,
    online_busy: Arc<AtomicBool>,
    detail_busy: Arc<AtomicBool>,
    tx: Sender<Result<EventType>>,
    rx: Receiver<Result<EventType>>,
}

impl Event {
    pub fn new(client: TUNetConnect, usereg: UseregHelper) -> Self {
        let (tx, rx) = channel(32);
        let e = Self {
            client,
            usereg,
            log_busy: Arc::new(AtomicBool::new(false)),
            online_busy: Arc::new(AtomicBool::new(false)),
            detail_busy: Arc::new(AtomicBool::new(false)),
            tx,
            rx,
        };
        e.spawn_terminal_event();
        e.spawn_timer();
        e.spawn_login();
        e.spawn_online();
        e.spawn_details();
        e
    }

    fn spawn_terminal_event(&self) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let stream = crossterm::event::EventStream::new();
            pin_mut!(stream);
            while let Some(e) = stream.next().await {
                tx.send(e.map(EventType::TerminalEvent).map_err(anyhow::Error::from))
                    .await?;
            }
            #[allow(unreachable_code)]
            Ok::<_, anyhow::Error>(())
        });
    }

    fn spawn_timer(&self) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                tx.send(Ok(EventType::Tick)).await?;
            }
            #[allow(unreachable_code)]
            Ok::<_, anyhow::Error>(())
        });
    }

    pub fn spawn_login(&self) {
        let mut lock = BusyGuard::new(self.log_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            let client = self.client.clone();
            tokio::spawn(async move {
                let _lock = lock;
                tx.send(Ok(EventType::Log(LogType::Login(None)))).await?;
                let res = client.login().await;
                tx.send(res.map(|res| EventType::LogDone(LogType::Login(Some(res)))))
                    .await?;
                let flux = client.flux().await;
                tx.send(flux.map(EventType::Flux)).await?;
                Ok::<_, anyhow::Error>(())
            });
        }
    }

    pub fn spawn_logout(&self) {
        let mut lock = BusyGuard::new(self.log_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            let client = self.client.clone();
            tokio::spawn(async move {
                let _lock = lock;
                tx.send(Ok(EventType::Log(LogType::Logout(None)))).await?;
                let res = client.logout().await;
                tx.send(res.map(|res| EventType::LogDone(LogType::Logout(Some(res)))))
                    .await?;
                Ok::<_, anyhow::Error>(())
            });
        }
    }

    pub fn spawn_flux(&self) {
        let mut lock = BusyGuard::new(self.log_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            let client = self.client.clone();
            tokio::spawn(async move {
                let _lock = lock;
                tx.send(Ok(EventType::Log(LogType::Flux))).await?;
                let flux = client.flux().await;
                tx.send(flux.map(EventType::Flux)).await?;
                tx.send(Ok(EventType::LogDone(LogType::Flux))).await?;
                Ok::<_, anyhow::Error>(())
            });
        }
    }

    pub fn spawn_online(&self) {
        let mut lock = BusyGuard::new(self.online_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            let usereg = self.usereg.clone();
            tokio::spawn(async move {
                let _lock = lock;
                tx.send(Ok(EventType::Log(LogType::Online))).await?;
                tx.send(Ok(EventType::ClearOnline)).await?;
                usereg.login().await?;
                let users = usereg.users();
                pin_mut!(users);
                while let Some(u) = users.next().await {
                    tx.send(u.map(EventType::AddOnline)).await?;
                }
                tx.send(Ok(EventType::LogDone(LogType::Online))).await?;
                Ok::<_, anyhow::Error>(())
            });
        }
    }

    pub fn spawn_details(&self) {
        let mut lock = BusyGuard::new(self.detail_busy.clone());
        if lock.lock() {
            let tx = self.tx.clone();
            let usereg = self.usereg.clone();
            tokio::spawn(async move {
                let _lock = lock;
                tx.send(Ok(EventType::Log(LogType::Detail))).await?;
                tx.send(Ok(EventType::ClearDetail)).await?;
                usereg.login().await?;
                let details = usereg.details(NetDetailOrder::LogoutTime, false);
                pin_mut!(details);
                while let Some(d) = details.next().await {
                    tx.send(d.map(EventType::AddDetail)).await?;
                }
                tx.send(Ok(EventType::LogDone(LogType::Detail))).await?;
                Ok::<_, anyhow::Error>(())
            });
        }
    }
}

impl Stream for Event {
    type Item = Result<EventType>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

#[derive(Debug)]
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
