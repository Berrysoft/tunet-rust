use crate::{App, HomeModel, NetInfo, SettingsModel};
use anyhow::Result;
use slint::{ComponentHandle, SharedString};
use std::{sync::Arc, sync::Mutex};
use tunet_helper::{NetFlux, NetState};
use tunet_model::{Action, Model, UpdateMsg};

#[derive(Clone)]
pub struct UpdateContext {
    weak_app: slint::Weak<App>,
    data: Arc<Mutex<UpdateData>>,
}

impl UpdateContext {
    pub fn new(app: &App) -> Self {
        Self {
            weak_app: app.as_weak(),
            data: Arc::new(Mutex::new(UpdateData::default())),
        }
    }

    pub fn weak_app(&self) -> &slint::Weak<App> {
        &self.weak_app
    }

    pub fn create_model(
        &self,
        action_sender: flume::Sender<Action>,
    ) -> Result<(Arc<Mutex<Model>>, flume::Receiver<UpdateMsg>)> {
        let (update_sender, update_receiver) = flume::bounded(32);
        let model = Arc::new(Mutex::new(Model::new(action_sender, update_sender)?));
        Ok((model, update_receiver))
    }

    fn update_username(&self, username: impl Into<SharedString>) {
        let username = username.into();
        self.weak_app
            .upgrade_in_event_loop(move |app| {
                app.global::<SettingsModel>().set_username(username);
            })
            .unwrap();
    }

    fn update_state(&self, state: NetState) {
        let state = state as i32 - 1;
        self.weak_app
            .upgrade_in_event_loop(move |app| app.global::<HomeModel>().set_state(state))
            .unwrap();
    }

    fn update_status(&self, status: impl Into<SharedString>) {
        let status = status.into();
        self.weak_app
            .upgrade_in_event_loop(move |app| {
                app.global::<HomeModel>().set_status(status);
            })
            .unwrap();
    }

    fn update_log(&self, log: impl Into<SharedString>) {
        let log = log.into();
        self.weak_app
            .upgrade_in_event_loop(move |app| {
                app.global::<HomeModel>().set_log(log);
            })
            .unwrap();
    }

    fn update_info(&self, flux: &NetFlux) {
        let info = NetInfo {
            username: flux.username.as_str().into(),
            flux_gb: flux.flux.to_gb() as _,
            flux_str: flux.flux.to_string().into(),
            online_time: flux.online_time.to_string().into(),
            balance: flux.balance.0 as _,
            balance_str: flux.balance.to_string().into(),
        };
        self.weak_app
            .upgrade_in_event_loop(move |app| {
                app.global::<HomeModel>().set_info(info);
            })
            .unwrap();
    }

    fn update_log_busy(&self, busy: bool) {
        self.weak_app
            .upgrade_in_event_loop(move |app| app.global::<HomeModel>().set_busy(busy))
            .unwrap();
    }

    pub fn update(&self, model: &Model, msg: UpdateMsg) {
        match msg {
            UpdateMsg::Credential => {
                model.queue(Action::State(None));
                self.update_username(&model.username);
            }
            UpdateMsg::State => {
                model.queue(Action::Flux);
                self.update_state(model.state);
            }
            UpdateMsg::Status => {
                model.queue(Action::State(None));
                self.update_status(model.status.to_string());
            }
            UpdateMsg::Log => {
                self.update_log(model.log.as_ref());
            }
            UpdateMsg::Flux => {
                self.update_info(&model.flux);
            }
            UpdateMsg::LogBusy => {
                self.update_log_busy(model.log_busy());
            }
        };
    }

    pub fn set_del_at_exit(&self) {
        self.data.lock().unwrap().del_at_exit = true;
    }

    pub fn del_at_exit(&self) -> bool {
        self.data.lock().unwrap().del_at_exit
    }
}

#[derive(Debug, Default)]
struct UpdateData {
    del_at_exit: bool,
}
