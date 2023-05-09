use crate::{
    accent_color, context::UpdateContext, AboutModel, DetailModel, HomeModel, SettingsModel,
};
use slint::{
    quit_event_loop, ComponentHandle, Model as SlintModel, ModelExt, ModelRc, SortModel,
    StandardListViewItem,
};
use std::{
    cmp::{Ordering, Reverse},
    sync::Arc,
};
use tokio::sync::Mutex;
use tunet_model::{Action, Model};

#[macro_export]
macro_rules! upgrade_spawn_body {
    ($m: ident, $w: expr, $t: expr) => {
        if let Some($m) = $w.upgrade() {
            tokio::spawn($t);
        }
    };
}

#[macro_export]
macro_rules! upgrade_spawn {
    ($m: ident, || $t: expr) => {{
        let weak_model = std::sync::Arc::downgrade(&$m);
        move || $crate::upgrade_spawn_body!($m, weak_model, $t)
    }};
    ($m: ident, | $($args: tt),* | $t: expr) => {{
        let weak_model = std::sync::Arc::downgrade(&$m);
        move |$($args),*| $crate::upgrade_spawn_body!($m, weak_model, $t)
    }};
}

#[macro_export]
macro_rules! upgrade_queue_body {
    ($m: ident, $t: expr) => {
        let model = $m.lock().await;
        model.queue($t);
    };
}

#[macro_export]
macro_rules! upgrade_queue {
    ($m: ident, || $t: expr) => {
        $crate::upgrade_spawn!($m, || async move {
            $crate::upgrade_queue_body!($m, $t);
        })
    };
    ($m: ident, | $($args: tt),* | $t: expr) => {
        $crate::upgrade_spawn!($m, |$($args),*| async move {
            $crate::upgrade_queue_body!($m, $t);
        })
    };
}

#[doc(hidden)]
pub fn sort_by_key<M: SlintModel, K: Ord + 'static>(
    model: M,
    mut key_func: impl FnMut(&M::Data) -> K + 'static,
) -> SortModel<M, impl FnMut(&M::Data, &M::Data) -> Ordering + 'static> {
    model.sort_by(move |lhs, rhs| key_func(lhs).cmp(&key_func(rhs)))
}

#[macro_export]
macro_rules! sort_by_key {
    ($data: expr, $index: expr, $keyf: expr) => {{
        let keyf = $keyf;
        let data: ModelRc<ModelRc<StandardListViewItem>> = std::rc::Rc::new(
            $crate::bind::sort_by_key($data, move |r: &ModelRc<StandardListViewItem>| {
                let c = r.row_data($index as usize).unwrap();
                keyf(c)
            }),
        )
        .into();
        data
    }};
}

#[macro_export]
macro_rules! sort_callback {
    ($app: expr, $mty: ty, $prop: ident, $sortf: expr) => {
        paste::paste! {{
            let weak_app = $app.clone();
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

#[macro_export]
macro_rules! sort_by_key_callback {
    ($app: expr, $mty: ty, $prop: ident, $keyf: expr) => {
        $crate::sort_callback!(
            $app,
            $mty,
            $prop,
            |data: ModelRc<ModelRc<StandardListViewItem>>, index| $crate::sort_by_key!(
                data, index, $keyf
            )
        )
    };
}

pub fn bind_home_model(home_model: &HomeModel, model: &Arc<Mutex<Model>>) {
    let color = accent_color();
    home_model.set_theme_color(slint::Color::from_argb_u8(255, color.r, color.g, color.b));
    home_model.set_theme_color_t1(slint::Color::from_argb_u8(168, color.r, color.g, color.b));
    home_model.set_theme_color_t2(slint::Color::from_argb_u8(84, color.r, color.g, color.b));

    home_model.on_state_changed(upgrade_queue!(model, |s| Action::State(Some(
        s.parse().unwrap()
    ))));

    home_model.on_login(upgrade_queue!(model, || Action::Login));
    home_model.on_logout(upgrade_queue!(model, || Action::Logout));
    home_model.on_refresh(upgrade_queue!(model, || Action::Flux));
}

pub fn bind_detail_model(
    detail_model: &DetailModel,
    model: &Arc<Mutex<Model>>,
    context: &UpdateContext,
) {
    detail_model.on_daily_chart({
        let context = context.clone();
        move |width, height, dark, text_color| {
            context.draw_daily_chart(width, height, dark, text_color)
        }
    });

    detail_model.on_refresh(upgrade_queue!(model, || Action::Details));

    detail_model.on_sort_ascending({
        let context = context.clone();
        move |index| {
            context.sort_details(index, false);
        }
    });
    detail_model.on_sort_descending({
        let context = context.clone();
        move |index| {
            context.sort_details(index, true);
        }
    });
}

pub fn bind_settings_model(
    settings_model: &SettingsModel,
    model: &Arc<Mutex<Model>>,
    context: &UpdateContext,
) {
    settings_model.on_set_credential(upgrade_queue!(model, |username, password| {
        Action::UpdateCredential(username.to_string(), password.to_string())
    }));
    settings_model.on_del_and_exit({
        let context = context.clone();
        move || {
            context.set_del_at_exit();
            quit_event_loop().unwrap();
        }
    });

    settings_model.on_refresh(upgrade_queue!(model, || Action::Online));

    settings_model.on_connect_ip(upgrade_queue!(model, |ip| Action::Connect(
        ip.parse().unwrap()
    )));
    settings_model.on_drop_ip(upgrade_queue!(model, |ip| Action::Drop(
        ip.parse().unwrap()
    )));
}

pub fn bind_about_model(about_model: &AboutModel, context: &UpdateContext) {
    about_model.set_version(env!("CARGO_PKG_VERSION").into());

    about_model.on_sort_ascending(sort_by_key_callback!(
        context.weak_app(),
        AboutModel,
        deps,
        |item: StandardListViewItem| item.text
    ));
    about_model.on_sort_descending(sort_by_key_callback!(
        context.weak_app(),
        AboutModel,
        deps,
        |item: StandardListViewItem| Reverse(item.text)
    ));
}
