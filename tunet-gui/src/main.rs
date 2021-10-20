#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gtk::prelude::*;
use lazy_static::lazy_static;
use relm4::*;
use tunet_rust::*;

#[cfg(windows)]
mod win32;

mod about;
mod clients;
mod detail;
mod header;
mod info;

#[tokio::main(worker_threads = 4)]
async fn main() -> Result<()> {
    clients::init()?;

    gtk::init()?;

    #[cfg(windows)]
    let factor = win32::get_scale_factor();
    #[cfg(not(windows))]
    let factor = 1.0;

    let style = gtk::CssProvider::new();
    style
        .load_from_data(format!("*{{font-size:{}px;}}", 16.0 * factor / factor.floor()).as_bytes());
    gtk::StyleContext::add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &style,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let model = MainModel::new();
    let app = gtk::Application::builder().build();
    let app = RelmApp::with_app(model, app);
    app.run();
    Ok(())
}

enum MainMode {
    Info,
    Detail,
    About,
}

enum MainMsg {
    Show,
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
        sender: Sender<MainMsg>,
    ) -> bool {
        match msg {
            MainMsg::Show => {
                send!(sender, MainMsg::Mode(MainMode::Info));
                components.info.send(info::InfoMsg::Show).unwrap();
                components.detail.send(detail::DetailMsg::Show).unwrap();
            }
            MainMsg::Mode(m) => match m {
                MainMode::Info => self.child = Some(components.info.root_widget().clone()),
                MainMode::Detail => self.child = Some(components.detail.root_widget().clone()),
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
            set_default_width: 500,
            set_default_height: 500,

            set_titlebar: component!(Some(components.header.root_widget())),

            connect_show(sender) => move |_| {
                send!(sender, MainMsg::Show);
            },
        }
    }

    fn manual_view() {
        self.wnd.set_child(model.child.as_ref());
    }
}

struct MainComponents {
    header: RelmComponent<header::HeaderModel, MainModel>,
    info: RelmComponent<info::InfoModel, MainModel>,
    detail: RelmComponent<detail::DetailModel, MainModel>,
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
            detail: RelmComponent::new(parent_model, parent_widgets, parent_sender.clone()),
            about: RelmComponent::new(parent_model, parent_widgets, parent_sender),
        }
    }
}
