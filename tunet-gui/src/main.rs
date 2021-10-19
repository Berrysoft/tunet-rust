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
    let app = gtk::Application::builder().build();

    let model = MainModel::new(app.clone());
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
    Resize,
    Mode(MainMode),
}

#[allow(dead_code)]
struct MainModel {
    dpi: f64,
    style: gtk::CssProvider,
    app: gtk::Application,
    child: Option<gtk::Box>,
}

impl MainModel {
    pub fn new(app: gtk::Application) -> Self {
        Self {
            dpi: 1.0,
            style: gtk::CssProvider::new(),
            app,
            child: None,
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
        components: &MainComponents,
        sender: Sender<MainMsg>,
    ) -> bool {
        match msg {
            MainMsg::Show => {
                send!(sender, MainMsg::Mode(MainMode::Info));
                components.info.send(info::InfoMsg::Show).unwrap();
                components.detail.send(detail::DetailMsg::Show).unwrap();
            }
            MainMsg::Resize =>
            {
                #[cfg(windows)]
                if let Some(window) = self.app.active_window() {
                    let factor = win32::get_scale_factor(window);
                    if self.dpi != factor {
                        self.dpi = factor;
                        let display = gtk::gdk::Display::default().unwrap();
                        gtk::StyleContext::remove_provider_for_display(&display, &self.style);
                        self.style = gtk::CssProvider::new();
                        self.style.load_from_data(
                            format!("*{{font-size:{}px;}}", 16.0 * factor).as_bytes(),
                        );
                        gtk::StyleContext::add_provider_for_display(
                            &display,
                            &self.style,
                            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                        );
                    }
                }
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
            set_default_width: 400,
            set_default_height: 400,

            set_titlebar: component!(Some(components.header.root_widget())),
            set_child: component!(Some(components.info.root_widget())),

            connect_show(sender) => move |_| {
                send!(sender, MainMsg::Show);
            },
            connect_default_width_notify(sender) => move |_| {
                send!(sender, MainMsg::Resize);
            },
            connect_default_height_notify(sender) => move |_| {
                send!(sender, MainMsg::Resize);
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
