use async_stream::try_stream;
pub use crossterm::event::Event as TerminalEvent;
use futures_util::{pin_mut, Stream, StreamExt};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::mpsc::*;
use tunet_rust::{usereg::*, *};

#[derive(Debug)]
pub enum EventType {
    TerminalEvent(TerminalEvent),
    Tick,
    Login(String),
    Flux(NetFlux),
    AddOnline(NetUser),
    AddDetail(NetDetail),
}

pub struct Event {
    rx: Receiver<Result<EventType>>,
}

impl Event {
    pub fn new(client: TUNetConnect, usereg: UseregHelper) -> Self {
        let (tx, rx) = channel(32);

        {
            let tx = tx.clone();
            tokio::spawn(async move {
                let stream = event_stream();
                pin_mut!(stream);
                while let Some(e) = stream.next().await {
                    tx.send(e.map(EventType::TerminalEvent).map_err(anyhow::Error::from))
                        .await?;
                }
                #[allow(unreachable_code)]
                Ok::<_, anyhow::Error>(())
            });
        }

        {
            let tx = tx.clone();
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

        {
            let tx = tx.clone();
            let client = client.clone();
            tokio::spawn(async move {
                let res = client.login().await;
                tx.send(res.map(EventType::Login)).await?;
                let flux = client.flux().await;
                tx.send(flux.map(EventType::Flux)).await?;
                Ok::<_, anyhow::Error>(())
            });
        }

        {
            let tx = tx.clone();
            let usereg = usereg.clone();
            tokio::spawn(async move {
                usereg.login().await?;
                let users = usereg.users();
                pin_mut!(users);
                while let Some(u) = users.next().await {
                    tx.send(u.map(EventType::AddOnline)).await?;
                }
                Ok::<_, anyhow::Error>(())
            });
        }

        {
            let tx = tx.clone();
            let usereg = usereg.clone();
            tokio::spawn(async move {
                usereg.login().await?;
                let details = usereg.details(NetDetailOrder::LogoutTime, false);
                pin_mut!(details);
                while let Some(d) = details.next().await {
                    tx.send(d.map(EventType::AddDetail)).await?;
                }
                Ok::<_, anyhow::Error>(())
            });
        }

        Self { rx }
    }
}

impl Stream for Event {
    type Item = Result<EventType>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

#[derive(Debug, Default)]
struct CrosstermReadFuture;

impl Future for CrosstermReadFuture {
    type Output = std::io::Result<TerminalEvent>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match crossterm::event::poll(std::time::Duration::from_secs(0)) {
            Ok(true) => Poll::Ready(crossterm::event::read()),
            Ok(false) => {
                cx.waker().clone().wake();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

// Simple replacement for `EventStream`.
fn event_stream() -> impl Stream<Item = std::io::Result<TerminalEvent>> {
    try_stream! {
        let f = CrosstermReadFuture::default();
        pin_mut!(f);
        loop {
            let e = f.as_mut().await?;
            yield e;
        }
    }
}
