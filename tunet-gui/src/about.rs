use crate::*;

pub struct AboutModel {
    libs: gtk::ListStore,
}

impl Model for AboutModel {
    type Msg = ();
    type Widgets = AboutWidgets;
    type Components = ();
}

static LIBS: &[(&str, &str)] = &[
    ("anyhow", "MIT"),
    ("async-stream", "MIT"),
    ("async-trait", "MIT"),
    ("cairo (C)", "LGPLv2.1"),
    ("chrono", "MIT"),
    ("crossterm", "MIT"),
    ("data-encoding", "MIT"),
    ("data-encoding-macro", "MIT"),
    ("dirs", "MIT"),
    ("futures-core", "MIT"),
    ("futures-util", "MIT"),
    ("glib (C)", "LGPLv2.1"),
    ("gtk (C)", "LGPLv2.1"),
    ("gtk4", "MIT"),
    ("hmac", "MIT"),
    ("itertools", "MIT"),
    ("keyutils", "BSD-3-Clause"),
    ("lazy_static", "MIT"),
    ("libc", "MIT"),
    ("libloading", "ISC"),
    ("mac_address", "MIT"),
    ("md-5", "MIT"),
    ("netlink_wi", "MIT"),
    ("objc", "MIT"),
    ("once_cell", "MIT"),
    ("regex", "MIT"),
    ("relm4", "MIT"),
    ("relm4_macros", "MIT"),
    ("reqwest", "MIT"),
    ("rpassword", "Apache-2.0"),
    ("security-framework", "MIT"),
    ("select", "MIT"),
    ("serde", "MIT"),
    ("serde_json", "MIT"),
    ("sha-1", "MIT"),
    ("structopt", "MIT"),
    ("termcolor", "MIT"),
    ("termcolor_output", "MIT"),
    ("thiserror", "MIT"),
    ("tokio", "MIT"),
    ("trait_enum", "MIT"),
    ("tui", "MIT"),
    ("url", "MIT"),
    ("wide-literials", "Unlicense"),
    ("widestring", "MIT"),
    ("windows", "MIT"),
];

impl ComponentUpdate<MainModel> for AboutModel {
    fn init_model(_parent_model: &MainModel) -> Self {
        let libs = gtk::ListStore::new(&[String::static_type(), String::static_type()]);
        for lib in LIBS {
            libs.set(&libs.append(), &[(0, &lib.0), (1, &lib.1)]);
        }
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
                set_markup: "<big><b>清华大学校园网客户端</b></big>",
            },
            append = &gtk::Label {
                set_label: "tunet-gui 0.1.0",
            },
            append = &gtk::Label {
                set_label: "版权所有 © 2021 Berrysoft",
            },

            append = &gtk::Label {
                set_margin_top: 10,
                set_markup: "<big><b>使用的开源库</b></big>",
            },
            append = &gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,

                set_child = Some(&gtk::TreeView) {
                    append_column = &gtk::TreeViewColumn::with_attributes("项目", &gtk::CellRendererText::new(), &[("text", 0)]) {
                        set_expand: true,
                        set_sort_column_id: 0,
                    },
                    append_column = &gtk::TreeViewColumn::with_attributes("许可证", &gtk::CellRendererText::new(), &[("text", 1)]) {
                        set_expand: true,
                        set_sort_column_id: 1,
                    },

                    set_model: Some(&model.libs),
                },
            },
        }
    }
}
