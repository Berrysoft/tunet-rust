use crate::*;

pub struct AboutModel {
    libs: gtk::ListStore,
}

impl Model for AboutModel {
    type Msg = ();
    type Widgets = AboutWidgets;
    type Components = ();
}

impl ComponentUpdate<MainModel> for AboutModel {
    fn init_model(_parent_model: &MainModel) -> Self {
        let libs = gtk::ListStore::new(&[String::static_type(), String::static_type()]);
        libs.set(&libs.append(), &[(0, &"tokio"), (1, &"MIT")]);
        Self { libs }
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

            append = &gtk::Label {
                set_margin_top: 10,
                set_markup: "<big>使用的开源库</big>",
            },
            append = &gtk::TreeView {
                append_column: col0 = &gtk::TreeViewColumn {
                    set_expand: true,
                    set_title: "项目",
                    pack_start(true): renderer0 = &gtk::CellRendererText {},
                },
                append_column: col1 = &gtk::TreeViewColumn {
                    set_expand: true,
                    set_title: "许可证",
                    pack_start(true): renderer1 = &gtk::CellRendererText {},
                },

                set_model: Some(&model.libs),
            }
        }
    }

    fn post_connect_components() {
        self.col0.add_attribute(&self.renderer0, "text", 0);
        self.col1.add_attribute(&self.renderer1, "text", 1);
    }
}
