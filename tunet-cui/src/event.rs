use anyhow::Result;
pub use crossterm::event::Event as TerminalEvent;
use crossterm::event::{EventStream, KeyCode, MouseButton, MouseEventKind};
use flume::{bounded, Receiver};
use futures_util::{FutureExt, StreamExt};
use ratatui::layout::Size;
use tunet_model::*;

#[derive(Debug)]
pub enum EventType {
    TerminalEvent(TerminalEvent),
    ModelAction(Action),
    UpdateState,
}

pub struct Event {
    pub model: Model,
    event: EventStream,
    mrx: Receiver<Action>,
    urx: Receiver<UpdateMsg>,
}

impl Event {
    pub fn new() -> Result<Self> {
        let (mtx, mrx) = bounded(32);
        let (utx, urx) = bounded(32);
        Ok(Self {
            model: Model::new(mtx, utx)?,
            event: EventStream::new(),
            mrx,
            urx,
        })
    }

    pub fn start(&self) {
        self.spawn_watch_status();
        self.spawn_timer();
        self.spawn_online();
        self.spawn_details();
    }

    pub async fn next_event(&mut self) -> Result<Option<EventType>> {
        loop {
            futures_util::select! {
                e = self.event.next().fuse() => break if let Some(e) = e {
                    Ok(Some(EventType::TerminalEvent(e?)))
                } else {
                    Ok(None)
                },
                a = self.mrx.recv_async().fuse() => break Ok(a.map(EventType::ModelAction).ok()),
                u = self.urx.recv_async().fuse() => match u {
                    Err(_) => break Ok(None),
                    Ok(UpdateMsg::State) => break Ok(Some(EventType::UpdateState)),
                    _ => {}
                },
            }
        }
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

    pub fn handle(&mut self, e: EventType, rect: Size) -> bool {
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
