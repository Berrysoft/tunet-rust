pub use crossterm::event::Event as TerminalEvent;
use std::sync::Arc;
use tokio::sync::mpsc::*;
use tunet_rust::*;

pub enum EventType {
    TerminalEvent(TerminalEvent),
    Flux(NetFlux),
    Tick,
}

pub struct Event {
    rx: Receiver<Result<EventType>>,
}

impl Event {
    pub fn new(client: Arc<TUNetConnect>) -> Self {
        let (tx, rx) = channel(10);

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
            let client = client.clone();
            tokio::spawn(async move {
                let flux = client.flux().await;
                tx.send(flux.map(EventType::Flux)).await.ok();
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
                    tokio::spawn(async move {
                        tx.send(Ok(EventType::Tick)).await.ok();
                    });
                }
            });
        }

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Result<EventType>> {
        self.rx.recv().await
    }
}
