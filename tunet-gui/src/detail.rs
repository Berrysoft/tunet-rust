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
                String::static_type(),
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
                        (2, &d.flux.to_string()),
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

            append = &gtk::TreeView {
                append_column: col0 = &gtk::TreeViewColumn {
                    set_expand: true,
                    set_title: "登录时间",
                    pack_start(true): renderer0 = &gtk::CellRendererText {},
                },
                append_column: col1 = &gtk::TreeViewColumn {
                    set_expand: true,
                    set_title: "注销时间",
                    pack_start(true): renderer1 = &gtk::CellRendererText {},
                },
                append_column: col2 = &gtk::TreeViewColumn {
                    set_expand: true,
                    set_title: "流量",
                    pack_start(true): renderer2 = &gtk::CellRendererText {},
                },

                set_model: Some(&model.details),
            },
        }
    }

    fn post_connect_components() {
        self.col0.add_attribute(&self.renderer0, "text", 0);
        self.col1.add_attribute(&self.renderer1, "text", 1);
        self.col2.add_attribute(&self.renderer2, "text", 2);
    }
}
