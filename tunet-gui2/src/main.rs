#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use itertools::Itertools;
use plotters::{
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea, RangedDate},
    series::LineSeries,
    style::{Color, FontFamily, IntoTextStyle, RGBColor, ShapeStyle, BLACK, WHITE},
};
use slint::{
    Image, Model as SlintModel, ModelRc, Rgb8Pixel, Rgba8Pixel, SharedPixelBuffer, SortModel,
    StandardListViewItem, VecModel,
};
use std::{
    cmp::{Ordering, Reverse},
    collections::HashMap,
    rc::Rc,
    sync::Arc,
};
use tokio::sync::{mpsc, Mutex};
use tunet_helper::{
    usereg::{NetDateTime, NetDetail, NetUser},
    Datelike, Flux, Local, Result,
};
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
            model.queue(Action::State(None));
            model.queue(Action::Online);
            model.queue(Action::Details);

            let username = model.cred.username.clone();
            weak_app
                .upgrade_in_event_loop(move |app| {
                    app.global::<SettingsModel>().set_username(username.into());
                })
                .unwrap();
        }
        UpdateMsg::State => {
            model.queue(Action::Flux);

            let state = model.state as i32 - 1;
            weak_app
                .upgrade_in_event_loop(move |app| app.global::<HomeModel>().set_state(state))
                .unwrap();
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
                flux_gb: flux.flux.to_gb() as _,
                flux_str: flux.flux.to_string().into(),
                online_time: flux.online_time.to_string().into(),
                balance: flux.balance.0 as _,
                balance_str: flux.balance.to_string().into(),
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
                .upgrade_in_event_loop(move |app| update_online(app, onlines, is_local))
                .unwrap();
        }
        UpdateMsg::Details => {
            let details = model.details.clone();
            weak_app
                .upgrade_in_event_loop(move |app| {
                    update_details(&app, &details);
                    draw_daily(&app, &details);
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

fn update_online(app: App, onlines: Vec<NetUser>, is_local: Vec<bool>) {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for (user, is_local) in onlines.into_iter().zip(is_local) {
        let items: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::default());
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
}

fn update_details(app: &App, details: &[NetDetail]) {
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for d in details {
        let items: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::default());
        items.push(d.login_time.to_string().as_str().into());
        items.push(d.logout_time.to_string().as_str().into());
        items.push(d.flux.to_string().as_str().into());
        row_data.push(items.into());
    }
    app.global::<DetailModel>().set_details(row_data.into());
}

fn draw_daily(app: &App, details: &[NetDetail]) {
    let color = color_theme::Color::accent();
    let color = RGBColor(color.r, color.g, color.b);
    let window_size = app.window().size();
    let (width, height) = ((window_size.width as f64 * 1.1) as u32, window_size.height);
    let scale = app.window().scale_factor();
    let dark = app.global::<ChartModel>().get_dark();
    let text_color = if dark { &WHITE } else { &BLACK };
    let back_color = if dark { &BLACK } else { &WHITE };

    let details = details
        .iter()
        .group_by(|d| d.logout_time.date())
        .into_iter()
        .map(|(key, group)| (key.day(), group.map(|d| d.flux.0).sum::<u64>()))
        .collect::<HashMap<_, _>>();
    let mut grouped_details = vec![];
    let now = Local::now().date_naive();
    let mut max = 0;
    for d in 1u32..=now.day() {
        if let Some(f) = details.get(&d) {
            max += *f;
        }
        grouped_details.push((now.with_day(d).unwrap(), max))
    }
    let date_range = (now.with_day(1).unwrap(), now);
    let flux_range = (0, max);

    let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width, height);
    let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), (width, height));
    {
        let root = backend.into_drawing_area();
        root.fill(back_color).unwrap();

        let label_style = (FontFamily::SansSerif, 16.0 * scale)
            .with_color(text_color)
            .into_text_style(&root);

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(20.0 * scale)
            .y_label_area_size(50.0 * scale)
            .margin(5.0 * scale)
            .build_cartesian_2d(
                RangedDate::from(date_range.0..date_range.1),
                flux_range.0..flux_range.1,
            )
            .unwrap();
        chart
            .configure_mesh()
            .disable_mesh()
            .axis_style(ShapeStyle {
                color: text_color.to_rgba(),
                filled: false,
                stroke_width: scale as _,
            })
            .x_desc("日期")
            .x_label_style(label_style.clone())
            .y_desc("流量")
            .y_label_style(label_style)
            .y_label_formatter(&|f| Flux(*f).to_string())
            .draw()
            .unwrap();
        chart
            .draw_series(
                LineSeries::new(
                    grouped_details,
                    ShapeStyle {
                        color: color.to_rgba(),
                        filled: true,
                        stroke_width: (scale * 2.0) as _,
                    },
                )
                .point_size((scale * 3.0) as _),
            )
            .unwrap();

        root.present().unwrap();
    }

    app.global::<ChartModel>()
        .set_daily_chart(image_from_rgb8_with_transparency(pixel_buffer, back_color));
}

fn image_from_rgb8_with_transparency(
    buffer: SharedPixelBuffer<Rgb8Pixel>,
    filter: &RGBColor,
) -> Image {
    let filter = Rgb8Pixel {
        r: filter.0,
        g: filter.1,
        b: filter.2,
    };
    let transparent = Rgba8Pixel {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    let mut new_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(buffer.width(), buffer.height());
    for (oldc, newc) in buffer.as_slice().iter().zip(new_buffer.make_mut_slice()) {
        if *oldc == filter {
            *newc = transparent;
        } else {
            newc.r = oldc.r;
            newc.g = oldc.g;
            newc.b = oldc.b;
            newc.a = 0xFF;
        }
    }
    Image::from_rgba8(new_buffer)
}
