use anyhow::anyhow;
use gtk::prelude::*;
use once_cell::sync::OnceCell;
use relm4::*;
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

    let model = MainModel::default();
    let app = RelmApp::new(model);
    app.run();
    Ok(())
}

static TUNET_CLIENT: OnceCell<TUNetConnect> = OnceCell::new();
static USEREG_CLIENT: OnceCell<UseregHelper> = OnceCell::new();

enum MainMsg {
    Flux(NetFlux),
}

#[derive(Debug, Default)]
struct MainModel {
    pub flux: NetFlux,
}

impl Model for MainModel {
    type Msg = MainMsg;
    type Widgets = MainWidgets;
    type Components = ();
}

impl AppUpdate for MainModel {
    fn update(&mut self, msg: MainMsg, _components: &(), _sender: Sender<MainMsg>) -> bool {
        match msg {
            MainMsg::Flux(f) => self.flux = f,
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
            set_default_height: 400,
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Center,

                    append = &gtk::Label {
                        set_xalign: 0.,
                        set_margin_all: 5,
                        set_label: watch! { &format!("用户：{}", model.flux.username) },
                    },
                    append = &gtk::Label {
                        set_xalign: 0.,
                        set_margin_all: 5,
                        set_label: watch! { &format!("流量：{}", model.flux.flux) },
                    },
                    append = &gtk::Label {
                        set_xalign: 0.,
                        set_margin_all: 5,
                        set_label: watch! { &format!("时长：{}", model.flux.online_time) },
                    },
                    append = &gtk::Label {
                        set_xalign: 0.,
                        set_margin_all: 5,
                        set_label: watch! { &format!("余额：{}", model.flux.balance) },
                    },
                },

                append = &gtk::Button {
                    set_label: "刷新",
                    connect_clicked(sender) => move |_| {
                        let sender = sender.clone();
                        tokio::spawn(async move {
                            let flux = TUNET_CLIENT.get().unwrap().flux().await?;
                            sender.send(MainMsg::Flux(flux))?;
                            Ok::<_, anyhow::Error>(())
                        });
                    },
                }
            },
        }
    }
}
