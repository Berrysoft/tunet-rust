pub use crossterm::event::Event as TerminalEvent;
use futures_util::{pin_mut, Stream, StreamExt};
use std::future::Future;
use tokio::sync::mpsc::*;
use tunet_rust::{usereg::*, *};

#[derive(Debug)]
pub enum EventType {
    TerminalEvent(TerminalEvent),
    Tick,
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
                loop {
                    let res = tx
                        .send(
                            crossterm::event::read()
                                .map(EventType::TerminalEvent)
                                .map_err(anyhow::Error::from),
                        )
                        .await;
                    if res.is_err() {
                        break;
                    }
                }
            });
        }

        {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                loop {
                    interval.tick().await;
                    if tx.is_closed() {
                        break;
                    }
                    let tx = tx.clone();
                    tokio::spawn(async move { tx.send(Ok(EventType::Tick)).await });
                }
            });
        }

        {
            let tx = tx.clone();
            let client = client.clone();
            tokio::spawn(async move {
                let flux = client.flux().await;
                tx.send(flux.map(EventType::Flux)).await
            });
        }

        {
            let tx = tx.clone();
            let mut usereg = usereg.clone();
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
            let mut usereg = usereg.clone();
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
        let f = self.rx.recv();
        pin_mut!(f);
        f.poll(cx)
    }
}
