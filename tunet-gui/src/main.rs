use gtk::prelude::*;
use relm4::*;
use tunet_rust::*;

#[tokio::main(worker_threads = 4)]
async fn main() {
    let model = MainModel::default();
    let app = RelmApp::new(model);
    app.run();
}

#[allow(dead_code)]
struct MainWidgets {
    window: gtk::ApplicationWindow,
    vbox: gtk::Box,
    label: gtk::Label,
}

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

impl Widgets<MainModel, ()> for MainWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(_model: &MainModel, _parent_widgets: &(), sender: Sender<MainMsg>) -> Self {
        let window = gtk::ApplicationWindow::builder()
            .title("清华校园网")
            .default_width(600)
            .default_height(800)
            .build();
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .build();
        vbox.set_margin_all(5);
        let label = gtk::Label::new(None);
        label.set_margin_all(5);

        window.set_child(Some(&vbox));
        vbox.append(&label);

        tokio::spawn(async move {
            let mut flux = NetFlux::default();
            flux.username = "test".to_owned();
            sender.send(MainMsg::Flux(flux))
        });

        Self {
            window,
            vbox,
            label,
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, model: &MainModel, _sender: Sender<MainMsg>) {
        self.label.set_text(&format!("{}", model.flux.username));
    }
}
