use anyhow::anyhow;
use gtk::prelude::*;
use once_cell::sync::OnceCell;
use relm4::{drawing::*, *};
use std::sync::Arc;
use tunet_rust::{usereg::*, *};

#[tokio::main(worker_threads = 4)]
async fn main() -> Result<()> {
    let cred = Arc::new(NetCredential::default());
    let client = create_http_client()?;
    let usereg = UseregHelper::new(cred.clone(), client.clone());
    USEREG_CLIENT
        .set(usereg)
        .map_err(|_| anyhow!("Cannot set usereg client."))?;
    let client = TUNetConnect::new(NetState::Net, cred, client).await?;
    TUNET_CLIENT
        .set(client)
        .map_err(|_| anyhow!("Cannot set tunet client."))?;

    let model = MainModel::new();
    let app = RelmApp::new(model);
    app.run();
    Ok(())
}

static TUNET_CLIENT: OnceCell<TUNetConnect> = OnceCell::new();
static USEREG_CLIENT: OnceCell<UseregHelper> = OnceCell::new();

enum MainMsg {
    Log(String),
    Flux(NetFlux),
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
}

impl Model for MainModel {
    type Msg = MainMsg;
    type Widgets = MainWidgets;
    type Components = ();
}

impl AppUpdate for MainModel {
    fn update(&mut self, msg: MainMsg, _components: &(), _sender: Sender<MainMsg>) -> bool {
        match msg {
            MainMsg::Log(s) => self.log = s,
            MainMsg::Flux(f) => self.flux = f,
            MainMsg::ChooseState(s) => self.state = s,
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
            set_resizable: false,

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
                        send!(sender, MainMsg::ChooseState(match c.active() {
                            Some(i) => match i {
                                0 => NetState::Net,
                                1 => NetState::Auth4,
                                2 => NetState::Auth6,
                                _ => unreachable!(),
                            },
                            None => NetState::Unknown,
                        }));
                    }
                },

                append = &gtk::Label {
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
                            let sender = sender.clone();
                            tokio::spawn(async move {
                                match TUNET_CLIENT.get().unwrap().flux().await {
                                    Ok(flux) => send!(sender, MainMsg::Flux(flux)),
                                    Err(e) => send!(sender, MainMsg::Log(e.to_string())),
                                }
                            });
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
