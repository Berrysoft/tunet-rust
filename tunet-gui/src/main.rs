#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gtk::prelude::*;
use relm4::*;
use tunet_rust::*;

mod about;
mod clients;
mod header;
mod info;

#[tokio::main(worker_threads = 4)]
async fn main() -> Result<()> {
    clients::init()?;

    let model = MainModel::new();
    let app = RelmApp::new(model);
    app.run();
    Ok(())
}

enum MainMode {
    Info,
    About,
}

enum MainMsg {
    Mode(MainMode),
}

struct MainModel {
    child: Option<gtk::Box>,
}

impl MainModel {
    pub fn new() -> Self {
        Self { child: None }
    }
}

impl Model for MainModel {
    type Msg = MainMsg;
    type Widgets = MainWidgets;
    type Components = MainComponents;
}

impl AppUpdate for MainModel {
    fn update(
        &mut self,
        msg: MainMsg,
        components: &MainComponents,
        _sender: Sender<MainMsg>,
    ) -> bool {
        match msg {
            MainMsg::Mode(m) => match m {
                MainMode::Info => self.child = Some(components.info.root_widget().clone()),
                MainMode::About => self.child = Some(components.about.root_widget().clone()),
            },
        }
        true
    }
}

#[relm4_macros::widget]
impl Widgets<MainModel, ()> for MainWidgets {
    view! {
        wnd = gtk::ApplicationWindow {
            set_title: Some("清华校园网"),
            set_default_width: 400,
            set_default_height: 400,

            set_titlebar: component!(Some(components.header.root_widget())),
            set_child: watch!{ model.child.as_ref() },
        }
    }

    fn post_connect_components() {
        self.wnd.set_child(Some(components.info.root_widget()));
    }
}

struct MainComponents {
    header: RelmComponent<header::HeaderModel, MainModel>,
    info: RelmComponent<info::InfoModel, MainModel>,
    about: RelmComponent<about::AboutModel, MainModel>,
}

impl Components<MainModel> for MainComponents {
    fn init_components(
        parent_model: &MainModel,
        parent_widgets: &MainWidgets,
        parent_sender: Sender<MainMsg>,
    ) -> Self {
        Self {
            header: RelmComponent::new(parent_model, parent_widgets, parent_sender.clone()),
            info: RelmComponent::new(parent_model, parent_widgets, parent_sender.clone()),
            about: RelmComponent::new(parent_model, parent_widgets, parent_sender),
        }
    }
}
