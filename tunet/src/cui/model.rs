use crate::cui::event::*;
use crossterm::event::{KeyCode, MouseButton, MouseEventKind};
use std::borrow::Cow;
use tui::layout::Rect;
use tunet_rust::{usereg::*, *};

#[derive(Debug, Default)]
pub struct Model {
    pub log: Option<Cow<'static, str>>,
    pub online: bool,
    pub detail: bool,
    pub flux: Option<NetFlux>,
    pub users: Vec<NetUser>,
    pub details: Vec<NetDetail>,
}

impl Model {
    pub fn handle(&mut self, event: &Event, e: EventType, rect: Rect) -> bool {
        match e {
            EventType::TerminalEvent(e) => match e {
                TerminalEvent::Key(k) => match k.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return false,
                    KeyCode::F(func) => {
                        if !self.handle_functions(event, func) {
                            return false;
                        }
                    }
                    _ => {}
                },
                TerminalEvent::Mouse(m) => {
                    if m.kind == MouseEventKind::Up(MouseButton::Left) {
                        if m.row == (rect.height - 1) {
                            if !self.handle_functions(event, (m.column / 10 + 1) as u8) {
                                return false;
                            }
                        }
                    }
                }
                _ => {}
            },
            EventType::Log(t) => match t {
                LogType::Login(_) => self.log = Some("正在登录".into()),
                LogType::Logout(_) => self.log = Some("正在注销".into()),
                LogType::Flux(_) => self.log = Some("正在刷新流量".into()),
                LogType::Online => self.online = true,
                LogType::Detail => self.detail = true,
            },
            EventType::LogDone(t) => match t {
                LogType::Login(s) => self.log = s.map(Cow::from),
                LogType::Logout(s) => self.log = s.map(Cow::from),
                LogType::Flux(s) => self.log = s.map(Cow::from),
                LogType::Online => self.online = false,
                LogType::Detail => self.detail = false,
            },
            EventType::Flux(f) => {
                self.flux = f;
            }
            EventType::Tick => {
                if let Some(flux) = &mut self.flux {
                    flux.online_time = flux.online_time + Duration::seconds(1);
                }
            }
            EventType::ClearOnline => {
                self.users.clear();
            }
            EventType::AddOnline(u) => {
                self.users.push(u);
            }
            EventType::ClearDetail => {
                self.details.clear();
            }
            EventType::AddDetail(d) => {
                self.details.push(d);
            }
        }
        true
    }

    fn handle_functions(&self, event: &Event, func: u8) -> bool {
        match func {
            1 => event.spawn_login(),
            2 => event.spawn_logout(),
            3 => event.spawn_flux(),
            4 => event.spawn_online(),
            5 => event.spawn_details(),
            6 => return false,
            _ => {}
        };
        true
    }
}
