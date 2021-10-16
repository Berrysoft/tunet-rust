#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gtk::prelude::*;
use relm4::{drawing::*, *};
use tunet_rust::*;

mod clients;
mod info;

#[tokio::main(worker_threads = 4)]
async fn main() -> Result<()> {
    clients::init()?;

    let model = MainModel::new();
    let app = RelmApp::new(model);
    app.run();
    Ok(())
}

enum MainMsg {}

#[derive(Debug)]
struct MainModel {}

impl MainModel {
    pub fn new() -> Self {
        Self {}
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
        sender: Sender<MainMsg>,
    ) -> bool {
        true
    }
}

#[relm4_macros::widget]
impl Widgets<MainModel, ()> for MainWidgets {
    view! {
        gtk::ApplicationWindow {
            set_title: Some("清华校园网"),
            set_default_width: 400,
            set_default_height: 400,

            set_child: component!(Some(components.info.root_widget())),
        }
    }
}

struct MainComponents {
    info: RelmComponent<info::InfoModel, MainModel>,
}

impl Components<MainModel> for MainComponents {
    fn init_components(
        parent_model: &MainModel,
        parent_widgets: &MainWidgets,
        parent_sender: Sender<MainMsg>,
    ) -> Self {
        Self {
            info: RelmComponent::new(parent_model, parent_widgets, parent_sender),
        }
    }
}
