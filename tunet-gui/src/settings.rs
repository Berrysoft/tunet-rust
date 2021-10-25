use crate::*;
use futures_util::{pin_mut, TryStreamExt};
use netstatus::*;
use tunet_rust::usereg::*;

pub enum SettingsMsg {
    Show,
    AddOnline(NetUser),
}

pub struct SettingsModel {
    status: NetStatus,
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
            online: gtk::ListStore::new(&[
                String::static_type(),
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
            SettingsMsg::AddOnline(u) => self.online.set(
                &self.online.append(),
                &[
                    (0, &u.address.to_string()),
                    (1, &u.login_time.to_string()),
                    (2, &u.mac_address.map(|a| a.to_string()).unwrap_or_default()),
                ],
            ),
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
                    append_column: col0 = &gtk::TreeViewColumn {
                        set_expand: true,
                        set_title: "IP地址",
                        pack_start(true): renderer0 = &gtk::CellRendererText {},
                    },
                    append_column: col1 = &gtk::TreeViewColumn {
                        set_expand: true,
                        set_title: "登录时间",
                        pack_start(true): renderer1 = &gtk::CellRendererText {},
                    },
                    append_column: col2 = &gtk::TreeViewColumn {
                        set_expand: true,
                        set_title: "MAC地址",
                        pack_start(true): renderer2 = &gtk::CellRendererText {},
                    },

                    set_model: Some(&model.online),
                },
            },
        }
    }

    fn post_connect_components() {
        self.col0.add_attribute(&self.renderer0, "text", 0);
        self.col1.add_attribute(&self.renderer1, "text", 1);
        self.col2.add_attribute(&self.renderer2, "text", 2);
    }
}
