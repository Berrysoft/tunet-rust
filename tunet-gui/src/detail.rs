use crate::*;
use futures_util::{pin_mut, TryStreamExt};
use tunet_rust::usereg::*;

pub enum DetailMsg {
    Show,
    AddDetail(NetDetail),
}

pub struct DetailModel {
    details: gtk::ListStore,
}

impl Model for DetailModel {
    type Msg = DetailMsg;
    type Widgets = DetailWidgets;
    type Components = ();
}

impl ComponentUpdate<MainModel> for DetailModel {
    fn init_model(_parent_model: &MainModel) -> Self {
        Self {
            details: gtk::ListStore::new(&[
                String::static_type(),
                String::static_type(),
                u64::static_type(),
            ]),
        }
    }

    fn update(
        &mut self,
        msg: DetailMsg,
        _components: &(),
        sender: Sender<DetailMsg>,
        _parent_sender: Sender<MainMsg>,
    ) {
        match msg {
            DetailMsg::Show => {
                tokio::spawn(async move {
                    let usereg = clients::usereg();
                    usereg.login().await?;
                    let details = usereg.details(NetDetailOrder::LogoutTime, false);
                    pin_mut!(details);
                    while let Some(d) = details.try_next().await? {
                        send!(sender, DetailMsg::AddDetail(d));
                    }
                    Ok::<_, anyhow::Error>(())
                });
            }
            DetailMsg::AddDetail(d) => {
                self.details.set(
                    &self.details.append(),
                    &[
                        (0, &d.login_time.to_string()),
                        (1, &d.logout_time.to_string()),
                        (2, &d.flux.0),
                    ],
                );
            }
        }
    }
}

#[relm4_macros::widget(pub)]
impl Widgets<DetailModel, MainModel> for DetailWidgets {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,

            append = &gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,

                set_child = Some(&gtk::TreeView) {
                    append_column: col0 = &gtk::TreeViewColumn {
                        set_expand: true,
                        set_title: "登录时间",
                        set_sort_column_id: 0,
                        pack_start(true): renderer0 = &gtk::CellRendererText {},
                    },
                    append_column: col1 = &gtk::TreeViewColumn {
                        set_expand: true,
                        set_title: "注销时间",
                        set_sort_column_id: 1,
                        pack_start(true): renderer1 = &gtk::CellRendererText {},
                    },
                    append_column: col2 = &gtk::TreeViewColumn {
                        set_expand: true,
                        set_title: "流量",
                        set_sort_column_id: 2,
                        pack_start(true): renderer2 = &renderer::CellRendererFlux {},
                    },

                    set_model: Some(&model.details),
                },
            },
        }
    }

    fn post_connect_components() {
        self.col0.add_attribute(&self.renderer0, "text", 0);
        self.col1.add_attribute(&self.renderer1, "text", 1);
        self.col2.add_attribute(&self.renderer2, "value", 2);
    }
}

mod renderer {
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    mod imp {
        use super::*;
        use std::sync::atomic::{AtomicU64, Ordering};

        #[derive(Debug, Default)]
        pub struct CellRendererFlux {
            value: AtomicU64,
            r: gtk::CellRendererText,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for CellRendererFlux {
            const NAME: &'static str = "CellRendererFlux";

            const ABSTRACT: bool = false;

            type Type = super::CellRendererFlux;

            type ParentType = gtk::CellRenderer;

            fn new() -> Self {
                Self {
                    value: AtomicU64::default(),
                    r: gtk::CellRendererText::new(),
                }
            }
        }

        impl ObjectImpl for CellRendererFlux {
            fn properties() -> &'static [glib::ParamSpec] {
                lazy_static::lazy_static! {
                    static ref PROPS: [glib::ParamSpec; 1] = [glib::ParamSpec::new_uint64(
                        "value",
                        "Value",
                        "Flux byte value",
                        0,
                        u64::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    )];
                }

                &*PROPS
            }

            fn property(
                &self,
                _obj: &Self::Type,
                _id: usize,
                pspec: &glib::ParamSpec,
            ) -> glib::Value {
                match pspec.name() {
                    "value" => self.value.load(Ordering::Acquire).to_value(),
                    _ => unreachable!(),
                }
            }

            fn set_property(
                &self,
                _obj: &Self::Type,
                _id: usize,
                value: &glib::Value,
                pspec: &glib::ParamSpec,
            ) {
                match pspec.name() {
                    "value" => {
                        let value = value.get::<u64>().unwrap();
                        self.value.store(value, Ordering::Release);
                        self.r
                            .set_property("text", tunet_rust::Flux(value).to_string())
                            .unwrap();
                    }
                    _ => unreachable!(),
                }
            }
        }

        impl CellRendererImpl for CellRendererFlux {
            fn snapshot<P: gtk::prelude::IsA<gtk::Widget>>(
                &self,
                _renderer: &Self::Type,
                snapshot: &gtk::Snapshot,
                widget: &P,
                background_area: &gtk::gdk::Rectangle,
                cell_area: &gtk::gdk::Rectangle,
                flags: gtk::CellRendererState,
            ) {
                self.r
                    .snapshot(snapshot, widget, background_area, cell_area, flags)
            }
        }
    }

    glib::wrapper! {
        pub struct CellRendererFlux(ObjectSubclass<imp::CellRendererFlux>)
            @extends gtk::CellRenderer;
    }

    impl CellRendererFlux {
        pub fn new() -> Self {
            glib::Object::new(&[]).unwrap()
        }
    }

    impl Default for CellRendererFlux {
        fn default() -> Self {
            Self::new()
        }
    }
}
