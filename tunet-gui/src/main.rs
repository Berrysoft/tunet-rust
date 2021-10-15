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
            MainMsg::Flux(f) => {
                self.flux = f.clone();
                components.flux_area.send(FluxAreaMsg::Flux(f)).unwrap();
            }
        }
        true
    }
}

struct MainComponents {
    flux_area: RelmComponent<FluxAreaModel, MainModel>,
}

impl Components<MainModel> for MainComponents {
    fn init_components(
        parent_model: &MainModel,
        parent_widgets: &MainWidgets,
        parent_sender: Sender<MainMsg>,
    ) -> Self {
        Self {
            flux_area: RelmComponent::new(parent_model, parent_widgets, parent_sender),
        }
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

                append = &gtk::Overlay {
                    set_height_request: 300,

                    add_overlay = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,

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

                    set_child: component!(Some(components.flux_area.root_widget())),
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
                },
            },
        }
    }
}

enum FluxAreaMsg {
    Flux(NetFlux),
}

#[derive(Debug, Default)]
struct FluxAreaModel {
    pub flux: NetFlux,
}

impl Model for FluxAreaModel {
    type Msg = FluxAreaMsg;
    type Widgets = FluxAreaWidgets;
    type Components = ();
}

impl ComponentUpdate<MainModel> for FluxAreaModel {
    fn init_model(parent_model: &MainModel) -> Self {
        Self {
            flux: parent_model.flux.clone(),
        }
    }

    fn update(
        &mut self,
        msg: FluxAreaMsg,
        _components: &(),
        _sender: Sender<FluxAreaMsg>,
        _parent_sender: Sender<MainMsg>,
    ) {
        match msg {
            FluxAreaMsg::Flux(f) => {
                self.flux = f;
            }
        }
    }
}

struct FluxAreaWidgets {
    area: gtk::DrawingArea,
}
impl Widgets<FluxAreaModel, MainModel> for FluxAreaWidgets {
    type Root = gtk::DrawingArea;

    fn init_view(
        _model: &FluxAreaModel,
        _parent_widgets: &MainWidgets,
        _sender: Sender<FluxAreaMsg>,
    ) -> Self {
        let area = gtk::DrawingArea::default();
        area.set_valign(gtk::Align::Fill);
        area.set_halign(gtk::Align::Fill);
        Self { area }
    }

    fn root_widget(&self) -> Self::Root {
        self.area.clone()
    }

    fn view(&mut self, model: &FluxAreaModel, _sender: Sender<FluxAreaMsg>) {
        use relm4::drawing::*;
        use std::f64::consts::PI;

        let width = self.area.width() as f64;
        let height = self.area.height() as f64;

        let radius = width.min(height) * 0.4;

        let mut handler = DrawHandler::new().unwrap();
        handler.init(&self.area);
        let context = handler.get_context().unwrap();
        context.set_line_width(radius * 0.15);

        let max_flux = model.flux.balance.0 + 50.;

        context.set_source_rgba(0., 120. / 255., 215. / 255., 0.5);
        context.arc(width / 2., height / 2., radius, 0., PI * 2.);
        context.stroke().unwrap();

        context.set_source_rgba(0., 120. / 255., 215. / 255., 0.75);
        context.arc(
            width / 2.,
            height / 2.,
            radius,
            PI / 2.,
            PI / 2. + (50. / max_flux * 2. * PI).min(PI * 2. - 0.001),
        );
        context.stroke().unwrap();

        context.set_source_rgb(0., 120. / 255., 215. / 255.);
        context.arc(
            width / 2.,
            height / 2.,
            radius,
            PI / 2.,
            PI / 2. + (model.flux.flux.to_gb() / max_flux * 2. * PI).min(PI * 2. - 0.001),
        );
        context.stroke().unwrap();
    }
}
