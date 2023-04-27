pub use crossterm::event::Event as TerminalEvent;
use crossterm::event::{KeyCode, MouseButton, MouseEventKind};
use futures_util::{pin_mut, Stream, StreamExt};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::mpsc::*;
use tui::layout::Rect;
use tunet_helper::*;
use tunet_model::*;

#[derive(Debug)]
pub enum EventType {
    TerminalEvent(TerminalEvent),
    ModelAction(Action),
    UpdateState,
}

pub struct Event {
    pub model: Model,
    tx: Sender<Result<EventType>>,
    rx: Receiver<Result<EventType>>,
}

impl Event {
    pub fn new() -> Result<Self> {
        let (tx, rx) = channel(32);
        let (mtx, mrx) = channel(32);
        let mut e = Self {
            model: Model::new(mtx)?,
            tx,
            rx,
        };
        e.attach_callback();
        e.spawn_terminal_event();
        e.spawn_model_action(mrx);
        Ok(e)
    }

    pub fn start(&self) {
        self.spawn_watch_status();
        self.spawn_timer();
        self.spawn_online();
        self.spawn_details();
    }

    #[allow(clippy::single_match)]
    fn attach_callback(&mut self) {
        let tx = self.tx.clone();
        self.model.update = Some(Box::new(move |m| match m {
            UpdateMsg::State => {
                let tx = tx.clone();
                tokio::spawn(async move { tx.send(Ok(EventType::UpdateState)).await.ok() });
            }
            _ => {}
        }));
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
            Ok::<_, anyhow::Error>(())
        });
    }

    fn spawn_model_action(&self, mut mrx: Receiver<Action>) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            while let Some(a) = mrx.recv().await {
                tx.send(Ok(EventType::ModelAction(a))).await?;
            }
            Ok::<_, anyhow::Error>(())
        });
    }

    fn spawn_watch_status(&self) {
        self.model.queue(Action::WatchStatus);
    }

    fn spawn_timer(&self) {
        self.model.queue(Action::Timer);
    }

    pub fn spawn_login(&self) {
        self.model.queue(Action::Login);
    }

    pub fn spawn_logout(&self) {
        self.model.queue(Action::Logout);
    }

    pub fn spawn_flux(&self) {
        self.model.queue(Action::Flux);
    }

    pub fn spawn_online(&self) {
        self.model.queue(Action::Online);
    }

    pub fn spawn_details(&self) {
        self.model.queue(Action::Details);
    }

    pub fn handle(&mut self, e: EventType, rect: Rect) -> bool {
        match e {
            EventType::TerminalEvent(e) => match e {
                TerminalEvent::Key(k) => match k.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return false,
                    KeyCode::F(func) => {
                        if !self.handle_functions(func) {
                            return false;
                        }
                    }
                    _ => {}
                },
                TerminalEvent::Mouse(m) => {
                    if m.kind == MouseEventKind::Up(MouseButton::Left)
                        && m.row == (rect.height - 1)
                        && !self.handle_functions((m.column / 10 + 1) as u8)
                    {
                        return false;
                    }
                }
                _ => {}
            },
            EventType::ModelAction(a) => {
                self.model.handle(a);
            }
            EventType::UpdateState => {
                self.spawn_flux();
            }
        }
        true
    }

    fn handle_functions(&self, func: u8) -> bool {
        match func {
            1 => self.spawn_login(),
            2 => self.spawn_logout(),
            3 => self.spawn_flux(),
            4 => self.spawn_online(),
            5 => self.spawn_details(),
            6 => return false,
            _ => {}
        };
        true
    }
}

impl Stream for Event {
    type Item = Result<EventType>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}
