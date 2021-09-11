use crate::cui::event::*;
use crossterm::event::KeyCode;
use tunet_rust::*;

#[derive(Debug, Default)]
pub struct Model {
    pub flux: Option<NetFlux>,
}

impl Model {
    pub fn handle(&mut self, e: EventType) -> bool {
        match e {
            EventType::TerminalEvent(e) => match e {
                TerminalEvent::Key(k) => match k.code {
                    KeyCode::Char('q') => return false,
                    _ => {}
                },
                _ => {}
            },
            EventType::Flux(f) => {
                self.flux = Some(f);
            }
            EventType::Tick => {
                if let Some(flux) = &mut self.flux {
                    flux.online_time = flux.online_time + Duration::seconds(1);
                }
            }
        }
        true
    }
}
