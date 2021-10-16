#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gtk::prelude::*;
use once_cell::unsync::OnceCell;
use relm4::{drawing::*, *};
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
    mode: MainMode,
}

impl MainModel {
    pub fn new() -> Self {
        Self {
            mode: MainMode::Info,
        }
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
        _components: &MainComponents,
        _sender: Sender<MainMsg>,
    ) -> bool {
        match msg {
            MainMsg::Mode(m) => self.mode = m,
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
        }
    }

    additional_fields! {
        info_widget: OnceCell<gtk::Box>,
        about_widget: OnceCell<gtk::Box>,
    }

    fn post_init() {
        let info_widget = OnceCell::new();
        let about_widget = OnceCell::new();
    }

    fn post_connect_components() {
        self.info_widget
            .set(components.info.root_widget().clone())
            .unwrap();
        self.about_widget
            .set(components.about.root_widget().clone())
            .unwrap();
        self.wnd.set_child(Some(self.info_widget.get().unwrap()));
    }

    fn manual_view() {
        match model.mode {
            MainMode::Info => {
                self.wnd.set_child(Some(self.info_widget.get().unwrap()));
            }
            MainMode::About => {
                self.wnd.set_child(Some(self.about_widget.get().unwrap()));
            }
        }
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
