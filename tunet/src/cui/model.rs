use crate::cui::event::*;
use crossterm::event::KeyCode;
use tunet_rust::{usereg::*, *};

#[derive(Debug, Default)]
pub struct Model {
    pub log: Option<String>,
    pub flux: Option<NetFlux>,
    pub users: Vec<NetUser>,
    pub details: Vec<NetDetail>,
}

impl Model {
    pub fn handle(&mut self, event: &Event, e: EventType) -> bool {
        match e {
            EventType::TerminalEvent(e) => match e {
                TerminalEvent::Key(k) => match k.code {
                    KeyCode::Char('q') => return false,
                    KeyCode::F(1) => event.spawn_login(),
                    KeyCode::F(2) => event.spawn_logout(),
                    KeyCode::F(3) => event.spawn_flux(),
                    KeyCode::F(4) => event.spawn_online(),
                    KeyCode::F(5) => event.spawn_details(),
                    _ => {}
                },
                _ => {}
            },
            EventType::Log(m) => {
                self.log = Some(m);
            }
            EventType::Flux(f) => {
                self.flux = Some(f);
            }
            EventType::Tick => {
                if let Some(flux) = &mut self.flux {
                    flux.online_time = flux.online_time + Duration::seconds(1);
                }
            }
            EventType::AddOnline(u) => {
                self.users.push(u);
            }
            EventType::AddDetail(d) => {
                self.details.push(d);
            }
        }
        true
    }
}
