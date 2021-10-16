#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::anyhow;
use gtk::prelude::*;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use relm4::{drawing::*, *};
use std::sync::Arc;
use tokio::sync::Mutex;
use tunet_rust::{usereg::*, *};

#[tokio::main(worker_threads = 4)]
async fn main() -> Result<()> {
    let cred = Arc::new(NetCredential::default());
    CREDENTIAL
        .set(cred.clone())
        .map_err(|_| anyhow!("Cannot set CREDENTIAL."))?;
    let usereg = UseregHelper::new(cred, HTTP_CLIENT.clone());
    USEREG_CLIENT
        .set(usereg)
        .map_err(|_| anyhow!("Cannot set USEREG_CLIENT."))?;

    let model = MainModel::new();
    let app = RelmApp::new(model);
    app.run();
    Ok(())
}

static CREDENTIAL: OnceCell<Arc<NetCredential>> = OnceCell::new();
static USEREG_CLIENT: OnceCell<UseregHelper> = OnceCell::new();

lazy_static! {
    static ref HTTP_CLIENT: HttpClient = create_http_client().unwrap();
    static ref TUNET_CLIENT: Mutex<Option<TUNetConnect>> = Mutex::new(None);
}

async fn tunet_replace(s: NetState) {
    *TUNET_CLIENT.lock().await = Some(match s {
        NetState::Net | NetState::Auth4 | NetState::Auth6 => {
            TUNetConnect::new(s, CREDENTIAL.get().unwrap().clone(), HTTP_CLIENT.clone())
                .await
                .unwrap()
        }
        _ => unreachable!(),
    });
}

async fn tunet_flux() -> Result<NetFlux> {
    TUNET_CLIENT
        .lock()
        .await
        .as_ref()
        .ok_or_else(|| anyhow!("请选择连接方式"))?
        .flux()
        .await
}

enum MainMsg {
    Refresh,
    Log(String),
    Flux(NetFlux),
    StartFlux,
    ChooseState(NetState),
}

#[derive(Debug)]
struct MainModel {
    pub log: String,
    pub flux: NetFlux,
    pub state: NetState,
}

impl MainModel {
    pub fn new() -> Self {
        Self {
            log: String::default(),
            flux: NetFlux::default(),
            state: NetState::Auto,
        }
    }

    async fn fetch_flux(s: Sender<MainMsg>) {
        match tunet_flux().await {
            Ok(flux) => {
                send!(s, MainMsg::Flux(flux));
                send!(s, MainMsg::Log(String::new()));
            }
            Err(e) => send!(s, MainMsg::Log(e.to_string())),
        }
    }
}

impl Model for MainModel {
    type Msg = MainMsg;
    type Widgets = MainWidgets;
    type Components = ();
}

impl AppUpdate for MainModel {
    fn update(&mut self, msg: MainMsg, _components: &(), sender: Sender<MainMsg>) -> bool {
        match msg {
            MainMsg::Refresh => {}
            MainMsg::Log(s) => self.log = s,
            MainMsg::Flux(f) => self.flux = f,
            MainMsg::StartFlux => {
                tokio::spawn(Self::fetch_flux(sender));
            }
            MainMsg::ChooseState(s) => {
                self.state = s;
                tokio::spawn(async move {
                    tunet_replace(s).await;
                    send!(sender, MainMsg::StartFlux);
                });
            }
        }
        true
    }
}

#[relm4_macros::widget]
impl Widgets<MainModel, ()> for MainWidgets {
    view! {
        gtk::ApplicationWindow {
            set_title: Some("清华校园网"),
            set_default_width: 300,
            set_default_height: 300,

            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Overlay {
                    set_height_request: 300,

                    add_overlay = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,

                        append = &gtk::Label {
                            set_xalign: 0.,
                            set_label: watch! { &format!("用户：{}", model.flux.username) },
                        },
                        append = &gtk::Label {
                            set_xalign: 0.,
                            set_label: watch! { &format!("流量：{}", model.flux.flux) },
                        },
                        append = &gtk::Label {
                            set_xalign: 0.,
                            set_label: watch! { &format!("时长：{}", model.flux.online_time) },
                        },
                        append = &gtk::Label {
                            set_xalign: 0.,
                            set_label: watch! { &format!("余额：{}", model.flux.balance) },
                        },
                    },

                    set_child: area = Some(&gtk::DrawingArea) {
                        set_vexpand: true,
                        set_hexpand: true,

                        connect_resize(sender) => move |_, _, _| {
                            send!(sender, MainMsg::Refresh);
                        },
                    }
                },

                append = &gtk::ComboBoxText {
                    append_text: "Net",
                    append_text: "Auth4",
                    append_text: "Auth6",

                    set_active: watch! {
                        match model.state {
                            NetState::Net => Some(0),
                            NetState::Auth4 => Some(1),
                            NetState::Auth6 => Some(2),
                            _ => None
                        }
                    },

                    connect_changed(sender) => move |c| {
                        let state = match c.active() {
                            Some(i) => match i {
                                0 => NetState::Net,
                                1 => NetState::Auth4,
                                2 => NetState::Auth6,
                                _ => unreachable!(),
                            },
                            None => NetState::Unknown,
                        };
                        send!(sender, MainMsg::ChooseState(state));
                    }
                },

                append = &gtk::Label {
                    set_wrap: true,
                    set_label: watch! { &model.log },
                },

                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_homogeneous: true,
                    set_halign: gtk::Align::Fill,

                    append = &gtk::Button {
                        set_label: "登录",
                    },
                    append = &gtk::Button {
                        set_label: "注销",
                    },
                    append = &gtk::Button {
                        set_label: "刷新",
                        connect_clicked(sender) => move |_| {
                            send!(sender, MainMsg::StartFlux);
                        },
                    },
                },
            },
        }
    }

    additional_fields! {
        handler: DrawHandler,
    }

    fn post_init() {
        let mut handler = DrawHandler::new().unwrap();
        handler.init(&area);
    }

    fn manual_view() {
        use std::f64::consts::PI;

        let width = self.area.width() as f64;
        let height = self.area.height() as f64;

        let radius = width.min(height) * 0.4;

        let context = self.handler.get_context().unwrap();

        context.save().unwrap();
        context.set_operator(gtk::cairo::Operator::Clear);
        context.paint().unwrap();
        context.restore().unwrap();

        context.set_line_width(radius * 0.2);

        let max_flux = model.flux.balance.0 + 50.;

        let flux_angle = PI / 2. + (model.flux.flux.to_gb() / max_flux * 2. * PI).min(PI * 2.);
        let free_angle = PI / 2. + (50. / max_flux * 2. * PI).min(PI * 2.);

        context.set_source_rgba(0., 120. / 255., 215. / 255., 0.55);
        context.arc(width / 2., height / 2., radius, free_angle, PI / 2.);
        context.stroke().unwrap();

        context.set_source_rgba(0., 120. / 255., 215. / 255., 0.75);
        context.arc(width / 2., height / 2., radius, flux_angle, free_angle);
        context.stroke().unwrap();

        context.set_source_rgb(0., 120. / 255., 215. / 255.);
        context.arc(width / 2., height / 2., radius, PI / 2., flux_angle);
        context.stroke().unwrap();
    }
}
