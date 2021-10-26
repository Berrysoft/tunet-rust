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
                    append_column = &gtk::TreeViewColumn::with_attributes("登录时间", &gtk::CellRendererText::new(), &[("text", 0)]) {
                        set_expand: true,
                        set_sort_column_id: 0,
                    },
                    append_column = &gtk::TreeViewColumn::with_attributes("注销时间", &gtk::CellRendererText::new(), &[("text", 1)]) {
                        set_expand: true,
                        set_sort_column_id: 1,
                    },
                    append_column = &gtk::TreeViewColumn::with_attributes("流量", &renderer::CellRendererFlux::new(), &[("value", 2)]) {
                        set_expand: true,
                        set_sort_column_id: 2,
                    },

                    set_model: Some(&model.details),
                },
            },
        }
    }
}
