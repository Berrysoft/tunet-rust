use crate::cui::event::*;
use crossterm::event::{KeyCode, MouseButton, MouseEventKind};
use itertools::Itertools;
use mac_address::*;
use std::borrow::Cow;
use std::collections::HashMap;
use tui::layout::Rect;
use tunet_rust::{usereg::*, *};

#[derive(Debug)]
pub struct Model {
    pub now: DateTime<Local>,
    pub log: Option<Cow<'static, str>>,
    pub online: bool,
    pub detail: bool,
    pub flux: Option<NetFlux>,
    pub users: Vec<NetUser>,
    pub details: Vec<(f64, f64)>,
    pub max_flux: Flux,
    pub mac_addrs: Vec<MacAddress>,
}

impl Model {
    pub fn new() -> Self {
        let mac_addrs = MacAddressIterator::new()
            .map(|it| it.collect::<Vec<_>>())
            .unwrap_or_default();
        Self {
            now: Local::now(),
            log: None,
            online: false,
            detail: false,
            flux: None,
            users: Vec::new(),
            details: Vec::new(),
            max_flux: Flux(0),
            mac_addrs,
        }
    }

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
            EventType::Detail(ds) => {
                const GIGABYTES: f64 = 1_000_000_000.0;

                let details_group = ds
                    .into_iter()
                    .group_by(|d| d.logout_time.date())
                    .into_iter()
                    .map(|(key, group)| (key.day(), group.map(|d| d.flux.0).sum::<u64>()))
                    .collect::<HashMap<_, _>>();
                for d in 1u32..=self.now.day() {
                    if let Some(f) = details_group.get(&d) {
                        self.max_flux.0 += *f;
                    }
                    self.details
                        .push((d as f64, self.max_flux.0 as f64 / GIGABYTES));
                }
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
