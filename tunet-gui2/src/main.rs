#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::{Model as SlintModel, ModelRc, SortModel, StandardListViewItem, VecModel};
use std::{
    cmp::{Ordering, Reverse},
    rc::Rc,
    sync::Arc,
};
use tokio::sync::{mpsc, Mutex};
use tunet_helper::{usereg::NetDateTime, Flux, Result};
use tunet_model::{Action, Model, UpdateMsg};
use tunet_settings::FileSettingsReader;

slint::include_modules!();

macro_rules! upgrade_spawn_body {
    ($m: ident, $w: expr, $t: expr) => {
        if let Some($m) = $w.upgrade() {
            tokio::spawn($t);
        }
    };
}

macro_rules! upgrade_spawn {
    ($m: ident, || $t: expr) => {{
        let weak_model = std::sync::Arc::downgrade(&$m);
        move || upgrade_spawn_body!($m, weak_model, $t)
    }};
    ($m: ident, | $args: tt | $t: expr) => {{
        let weak_model = std::sync::Arc::downgrade(&$m);
        move |$args| upgrade_spawn_body!($m, weak_model, $t)
    }};
}

macro_rules! upgrade_queue_body {
    ($m: ident, $t: expr) => {
        let model = $m.lock().await;
        model.queue($t);
    };
}

macro_rules! upgrade_queue {
    ($m: ident, || $t: expr) => {
        upgrade_spawn!($m, || async move {
            upgrade_queue_body!($m, $t);
        })
    };
    ($m: ident, | $args: tt | $t: expr) => {
        upgrade_spawn!($m, |$args| async move {
            upgrade_queue_body!($m, $t);
        })
    };
}

fn sort_by_key<M: SlintModel, K: Ord + 'static>(
    model: M,
    mut key_func: impl FnMut(&M::Data) -> K + 'static,
) -> SortModel<M, impl FnMut(&M::Data, &M::Data) -> Ordering + 'static> {
    model.sort_by(move |lhs, rhs| key_func(lhs).cmp(&key_func(rhs)))
}

macro_rules! sort_by_key {
    ($data: expr, $index: expr, $keyf: expr) => {{
        let keyf = $keyf;
        let data: ModelRc<ModelRc<StandardListViewItem>> = std::rc::Rc::new(sort_by_key(
            $data,
            move |r: &ModelRc<StandardListViewItem>| {
                let c = r.row_data($index as usize).unwrap();
                keyf(c)
            },
        ))
        .into();
        data
    }};
}

macro_rules! sort_callback {
    ($app: expr, $mty: ty, $prop: ident, $sortf: expr) => {
        paste::paste! {{
            let weak_app = $app.as_weak();
            let sortf = $sortf;
            move |index| {
                if let Some(app) = weak_app.upgrade() {
                    let model = app.global::<$mty>();
                    let data = model.[<get_ $prop>]();
                    let sort_data = sortf(data.clone(), index);
                    model.[<set_ $prop>](sort_data.into());
                }
            }
        }}
    };
}

macro_rules! sort_by_key_callback {
    ($app: expr, $mty: ty, $prop: ident, $keyf: expr) => {
        sort_callback!(
            $app,
            $mty,
            $prop,
            |data: ModelRc<ModelRc<StandardListViewItem>>, index| sort_by_key!(data, index, $keyf)
        )
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::new()?;

    let color = color_theme::Color::accent();
    let home_model = app.global::<HomeModel>();
    let detail_model = app.global::<DetailModel>();
    let settings_model = app.global::<SettingsModel>();
    let about_model = app.global::<AboutModel>();

    home_model.set_theme_color(slint::Color::from_argb_u8(255, color.r, color.g, color.b));
    home_model.set_theme_color_t1(slint::Color::from_argb_u8(191, color.r, color.g, color.b));
    home_model.set_theme_color_t2(slint::Color::from_argb_u8(140, color.r, color.g, color.b));

    about_model.set_version(env!("CARGO_PKG_VERSION").into());

    let (tx, mut rx) = mpsc::channel(32);
    let model = Arc::new(Mutex::new(Model::new(tx)?));
    {
        let weak_app = app.as_weak();
        let update = upgrade_spawn!(model, |msg| {
            let weak_app = weak_app.clone();
            async move {
                let model = model.lock().await;
                update(&model, msg, weak_app)
            }
        });
        let mut model = model.lock().await;
        model.update = Some(Box::new(update));

        let cred = Arc::new(FileSettingsReader::new()?.read_with_password()?);
        model.queue(Action::Credential(cred));
        model.queue(Action::Timer);

        settings_model.set_status(model.status.to_string().into());
    }

    home_model.on_state_changed(upgrade_queue!(model, |s| Action::State(Some(
        s.parse().unwrap()
    ))));

    home_model.on_login(upgrade_queue!(model, || Action::Login));
    home_model.on_logout(upgrade_queue!(model, || Action::Logout));
    home_model.on_refresh(upgrade_queue!(model, || Action::Flux));

    detail_model.on_refresh(upgrade_queue!(model, || Action::Details));
    // TODO: reduce parsing
    detail_model.on_sort_ascending(sort_callback!(app, DetailModel, details, |data, index| {
        match index {
            0 | 1 => sort_by_key!(data, index, |item: StandardListViewItem| {
                item.text.parse::<NetDateTime>().unwrap()
            }),
            2 => sort_by_key!(data, index, |item: StandardListViewItem| {
                item.text.parse::<Flux>().unwrap()
            }),
            _ => unreachable!(),
        }
    }));
    detail_model.on_sort_descending(sort_callback!(app, DetailModel, details, |data, index| {
        match index {
            0 | 1 => sort_by_key!(data, index, |item: StandardListViewItem| Reverse(
                item.text.parse::<NetDateTime>().unwrap()
            )),
            2 => sort_by_key!(data, index, |item: StandardListViewItem| Reverse(
                item.text.parse::<Flux>().unwrap()
            )),
            _ => unreachable!(),
        }
    }));

    settings_model.on_refresh(upgrade_queue!(model, || Action::Online));

    about_model.on_sort_ascending(sort_by_key_callback!(
        app,
        AboutModel,
        deps,
        |item: StandardListViewItem| item.text
    ));
    about_model.on_sort_descending(sort_by_key_callback!(
        app,
        AboutModel,
        deps,
        |item: StandardListViewItem| Reverse(item.text)
    ));

    tokio::spawn(async move {
        while let Some(a) = rx.recv().await {
            model.lock().await.handle(a);
        }
    });

    app.run()?;
    Ok(())
}

fn update(model: &Model, msg: UpdateMsg, weak_app: slint::Weak<App>) {
    match msg {
        UpdateMsg::Credential => {
            let username = model.cred.username.clone();
            weak_app
                .upgrade_in_event_loop(move |app| {
                    app.global::<SettingsModel>().set_username(username.into());
                })
                .unwrap();
            model.queue(Action::State(None));
            model.queue(Action::Online);
            model.queue(Action::Details);
        }
        UpdateMsg::State => {
            let state = model.state as i32 - 1;
            weak_app
                .upgrade_in_event_loop(move |app| app.global::<HomeModel>().set_state(state))
                .unwrap();
            model.queue(Action::Flux);
        }
        UpdateMsg::Log => {
            let log = model.log.as_ref().into();
            weak_app
                .upgrade_in_event_loop(move |app| {
                    app.global::<HomeModel>().set_log(log);
                })
                .unwrap();
        }
        UpdateMsg::Flux => {
            let flux = &model.flux;
            let info = NetInfo {
                username: flux.username.as_str().into(),
                flux: flux.flux.to_string().into(),
                online_time: flux.online_time.to_string().into(),
                balance: flux.balance.to_string().into(),
            };
            weak_app
                .upgrade_in_event_loop(move |app| {
                    app.global::<HomeModel>().set_info(info);
                })
                .unwrap();
        }
        UpdateMsg::Online => {
            let onlines = model.users.clone();
            let is_local = onlines
                .iter()
                .map(|user| {
                    model
                        .mac_addrs
                        .iter()
                        .any(|it| Some(it) == user.mac_address.as_ref())
                })
                .collect::<Vec<_>>();
            weak_app
                .upgrade_in_event_loop(move |app| {
                    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> =
                        Rc::new(VecModel::default());
                    for (user, is_local) in onlines.into_iter().zip(is_local) {
                        let items: Rc<VecModel<StandardListViewItem>> =
                            Rc::new(VecModel::default());
                        items.push(user.address.to_string().as_str().into());
                        items.push(user.login_time.to_string().as_str().into());
                        items.push(user.flux.to_string().as_str().into());
                        items.push(
                            user.mac_address
                                .map(|addr| addr.to_string())
                                .unwrap_or_default()
                                .as_str()
                                .into(),
                        );
                        items.push(if is_local { "本机" } else { "未知" }.into());
                        row_data.push(items.into());
                    }
                    app.global::<SettingsModel>().set_onlines(row_data.into());
                })
                .unwrap();
        }
        UpdateMsg::Details => {
            let details = model.details.clone();
            weak_app
                .upgrade_in_event_loop(move |app| {
                    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> =
                        Rc::new(VecModel::default());
                    for d in details {
                        let items: Rc<VecModel<StandardListViewItem>> =
                            Rc::new(VecModel::default());
                        items.push(d.login_time.to_string().as_str().into());
                        items.push(d.logout_time.to_string().as_str().into());
                        items.push(d.flux.to_string().as_str().into());
                        row_data.push(items.into());
                    }
                    app.global::<DetailModel>().set_details(row_data.into());
                })
                .unwrap();
        }
        UpdateMsg::LogBusy => {
            let busy = model.log_busy();
            weak_app
                .upgrade_in_event_loop(move |app| app.global::<HomeModel>().set_busy(busy))
                .unwrap();
        }
        UpdateMsg::OnlineBusy => {
            let busy = model.online_busy();
            weak_app
                .upgrade_in_event_loop(move |app| app.global::<SettingsModel>().set_busy(busy))
                .unwrap();
        }
        UpdateMsg::DetailBusy => {
            let busy = model.detail_busy();
            weak_app
                .upgrade_in_event_loop(move |app| app.global::<DetailModel>().set_busy(busy))
                .unwrap();
        }
    };
}
