use crate::*;
use futures_util::{pin_mut, TryStreamExt};
use mac_address::{MacAddress, MacAddressIterator};
use netstatus::*;
use tunet_rust::usereg::*;

pub enum SettingsMsg {
    Show,
    AddOnline(NetUser),
}

pub struct SettingsModel {
    status: NetStatus,
    mac_addrs: Vec<MacAddress>,
    online: gtk::ListStore,
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
            mac_addrs: MacAddressIterator::new()
                .map(|it| it.collect::<Vec<_>>())
                .unwrap_or_default(),
            online: gtk::ListStore::new(&[
                String::static_type(),
                String::static_type(),
                u64::static_type(),
                String::static_type(),
                String::static_type(),
            ]),
        }
    }

    fn update(
        &mut self,
        msg: SettingsMsg,
        _components: &(),
        sender: Sender<SettingsMsg>,
        _parent_sender: Sender<MainMsg>,
    ) {
        match msg {
            SettingsMsg::Show => {
                self.status = NetStatus::current();
                tokio::spawn(async move {
                    let usereg = clients::usereg();
                    usereg.login().await?;
                    let users = usereg.users();
                    pin_mut!(users);
                    while let Some(u) = users.try_next().await? {
                        send!(sender, SettingsMsg::AddOnline(u));
                    }
                    Ok::<_, anyhow::Error>(())
                });
            }
            SettingsMsg::AddOnline(u) => {
                let is_self = self
                    .mac_addrs
                    .iter()
                    .any(|it| Some(it) == u.mac_address.as_ref());
                self.online.set(
                    &self.online.append(),
                    &[
                        (0, &u.address.to_string()),
                        (1, &u.login_time.to_string()),
                        (2, &u.flux.0),
                        (3, &u.mac_address.map(|a| a.to_string()).unwrap_or_default()),
                        (4, &(if is_self { "本机" } else { "未知" })),
                    ],
                );
            }
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
                set_markup: "<big><b>当前用户</b></big>",
            },
            append = &gtk::Label {
                set_label: &clients::cred().username,
            },
            append = &gtk::Label {
                set_markup: "<big><b>网络状态</b></big>",
            },
            append = &gtk::Label {
                set_label: watch! { &model.status.to_string() },
            },

            append = &gtk::Label {
                set_markup: "<big><b>管理连接</b></big>",
            },
            append = &gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,

                set_child = Some(&gtk::TreeView) {
                    append_column = &gtk::TreeViewColumn::with_attributes("IP地址", &gtk::CellRendererText::new(), &[("text", 0)]) {
                        set_expand: true,
                        set_sort_column_id: 0,
                    },
                    append_column = &gtk::TreeViewColumn::with_attributes("登录时间", &gtk::CellRendererText::new(), &[("text", 1)]) {
                        set_expand: true,
                        set_sort_column_id: 1,
                    },
                    append_column = &gtk::TreeViewColumn::with_attributes("流量", &renderer::CellRendererFlux::new(), &[("value", 2)]) {
                        set_expand: true,
                        set_sort_column_id: 2,
                    },
                    append_column = &gtk::TreeViewColumn::with_attributes("MAC地址", &gtk::CellRendererText::new(), &[("text", 3)]) {
                        set_expand: true,
                    },
                    append_column = &gtk::TreeViewColumn::with_attributes("设备", &gtk::CellRendererText::new(), &[("text", 4)]) {
                        set_expand: true,
                    },

                    set_model: Some(&model.online),
                },
            },
        }
    }
}
