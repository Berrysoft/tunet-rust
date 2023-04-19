#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::Model as SlintModel;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tunet_helper::Result;
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

macro_rules! sort_impl {
    ($left: expr, $right: expr, +) => {
        $left.cmp(&$right)
    };
    ($left: expr, $right: expr, -) => {
        $right.cmp(&$left)
    };
}

macro_rules! sort_callback {
    ($app: expr, $model: expr, $mty: ty, $prop: ident, $order: tt) => {
        paste::paste! {{
            let weak_app = $app.as_weak();
            let data = $model.[<get_ $prop>]();
            move |index| {
                let sort_data = std::rc::Rc::new(data.clone().sort_by(move |r_a, r_b| {
                    let c_a = r_a.row_data(index as usize).unwrap();
                    let c_b = r_b.row_data(index as usize).unwrap();
                    sort_impl!(c_a.text, c_b.text, $order)
                }));
                if let Some(app) = weak_app.upgrade() {
                    app.global::<$mty>().[<set_ $prop>](sort_data.into());
                }
            }
        }}
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::new()?;

    let color = color_theme::Color::accent();
    let home_model = app.global::<HomeModel>();
    home_model.set_theme_color(slint::Color::from_argb_u8(255, color.r, color.g, color.b));
    home_model.set_theme_color_t1(slint::Color::from_argb_u8(191, color.r, color.g, color.b));
    home_model.set_theme_color_t2(slint::Color::from_argb_u8(140, color.r, color.g, color.b));

    let about_model = app.global::<AboutModel>();
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
    }

    home_model.on_state_changed(upgrade_queue!(model, |s| Action::State(Some(
        s.parse().unwrap()
    ))));

    home_model.on_login(upgrade_queue!(model, || Action::Login));
    home_model.on_logout(upgrade_queue!(model, || Action::Logout));
    home_model.on_refresh(upgrade_queue!(model, || Action::Flux));

    about_model.on_sort_ascending(sort_callback!(app, about_model, AboutModel, deps, +));
    about_model.on_sort_descending(sort_callback!(app, about_model, AboutModel, deps, -));

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
        UpdateMsg::LogBusy => {
            let busy = model.log_busy();
            weak_app
                .upgrade_in_event_loop(move |app| app.global::<HomeModel>().set_busy(busy))
                .unwrap();
        }
        _ => {}
    };
}
