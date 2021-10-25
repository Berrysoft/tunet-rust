use crate::*;
use netstatus::*;

pub enum SettingsMsg {
    Show,
}

pub struct SettingsModel {
    status: NetStatus,
}

impl Model for SettingsModel {
    type Msg = SettingsMsg;
    type Widgets = SettingsWidgets;
    type Components = ();
}

impl ComponentUpdate<MainModel> for SettingsModel {
    fn init_model(_parent_model: &MainModel) -> Self {
        Self {
            status: NetStatus::Unknown,
        }
    }

    fn update(
        &mut self,
        msg: SettingsMsg,
        _components: &(),
        _sender: Sender<SettingsMsg>,
        _parent_sender: Sender<MainMsg>,
    ) {
        match msg {
            SettingsMsg::Show => self.status = NetStatus::current(),
        }
    }
}

#[relm4_macros::widget(pub)]
impl Widgets<SettingsModel, MainModel> for SettingsWidgets {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            set_margin_all: 5,

            append = &gtk::Label {
                set_xalign: 0.,
                set_markup: "<b>当前用户：</b>",
            },
            append = &gtk::Label {
                set_xalign: 0.,
                set_markup: watch! { &format!("<b>网络状态：</b>{}", model.status) },
            },
        }
    }
}
