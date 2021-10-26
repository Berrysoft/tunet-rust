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

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
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
