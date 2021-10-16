use crate::*;

pub struct AboutModel {}

impl Model for AboutModel {
    type Msg = ();
    type Widgets = AboutWidgets;
    type Components = ();
}

impl ComponentUpdate<MainModel> for AboutModel {
    fn init_model(_parent_model: &MainModel) -> Self {
        Self {}
    }

    fn update(
        &mut self,
        _msg: (),
        _components: &(),
        _sender: Sender<()>,
        _parent_sender: Sender<MainMsg>,
    ) {
    }
}

#[relm4_macros::widget(pub)]
impl Widgets<AboutModel, MainModel> for AboutWidgets {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,

            append = &gtk::Label {
                set_markup: "<big>清华大学校园网客户端</big>",
            },
            append = &gtk::Label {
                set_label: "tunet-gui 0.1.0",
            },
            append = &gtk::Label {
                set_label: "版权所有 © 2021 Berrysoft",
            },
        }
    }
}