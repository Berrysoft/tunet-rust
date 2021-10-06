use crate::cui::event::*;
use crossterm::event::KeyCode;
use std::borrow::Cow;
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
            EventType::Log(t) => match t {
                LogType::Login(_) => self.log = Some("正在登录".into()),
                LogType::Logout(_) => self.log = Some("正在注销".into()),
                LogType::Flux => self.log = Some("正在刷新流量".into()),
                LogType::Online => self.online = true,
                LogType::Detail => self.detail = true,
            },
            EventType::LogDone(t) => match t {
                LogType::Login(s) => self.log = s.map(Cow::from),
                LogType::Logout(s) => self.log = s.map(Cow::from),
                LogType::Flux => self.log = None,
                LogType::Online => self.online = false,
                LogType::Detail => self.detail = false,
            },
            EventType::Flux(f) => {
                self.flux = Some(f);
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
}
