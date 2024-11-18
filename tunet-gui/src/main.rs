#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bind;
mod context;

use bind::{bind_about_model, bind_detail_model, bind_home_model, bind_settings_model};
use context::UpdateContext;

use anyhow::Result;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::{PhysicalPosition, Window};
use std::sync::{Arc, Mutex};
use tunet_model::{Action, Model, UpdateMsg};
use tunet_settings::SettingsReader;

slint::include_modules!();

fn main() -> Result<()> {
    #[cfg(target_os = "linux")]
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        wayland_set_app_id()?;
    }

    let app = App::new()?;

    let context = UpdateContext::new(&app);
    let (tx, rx) = flume::bounded(32);
    let (ctx_tx, ctx_rx) = flume::bounded(1);
    let (model, u_rx) = context.create_model(tx)?;
    start_runtime(model.clone(), context.clone(), u_rx, rx, ctx_rx);

    {
        let home_model = app.global::<HomeModel>();
        bind_home_model(&home_model, &model);

        let detail_model = app.global::<DetailModel>();
        bind_detail_model(&detail_model, &model, &context);

        let settings_model = app.global::<SettingsModel>();
        bind_settings_model(&settings_model, &model, &context);

        let about_model = app.global::<AboutModel>();
        bind_about_model(&about_model, &context);
    }

    app.show()?;

    center_window(app.window());

    app.run()?;

    ctx_tx.send(context.del_at_exit())?;

    Ok(())
}

fn start_runtime(
    model: Arc<Mutex<Model>>,
    context: UpdateContext,
    update_receiver: flume::Receiver<UpdateMsg>,
    action_receiver: flume::Receiver<Action>,
    ctx_reciver: flume::Receiver<bool>,
) {
    std::thread::spawn(move || {
        compio::runtime::RuntimeBuilder::new()
            .build()?
            .block_on(async {
                start_model(&model)?;
                start_model_loop(model.clone(), context, update_receiver, action_receiver);
                stop_model(&model, ctx_reciver.recv_async().await?)?;
                anyhow::Ok(())
            })
    });
}

fn start_model(model: &Mutex<Model>) -> Result<()> {
    let model = model.lock().unwrap();
    let settings_reader = SettingsReader::new()?;
    if let Ok((u, p)) = settings_reader.read_full() {
        model.queue(Action::Credential(u, p));
    }
    model.queue(Action::Status(None));
    model.queue(Action::WatchStatus);
    model.queue(Action::Timer);
    Ok(())
}

fn start_model_loop(
    model: Arc<Mutex<Model>>,
    context: UpdateContext,
    update_receiver: flume::Receiver<UpdateMsg>,
    rx: flume::Receiver<Action>,
) {
    {
        let model = model.clone();
        compio::runtime::spawn(async move {
            while let Ok(msg) = update_receiver.recv_async().await {
                let model = model.lock().unwrap();
                context.update(&model, msg)
            }
        })
        .detach();
    }
    compio::runtime::spawn(async move {
        while let Ok(a) = rx.recv_async().await {
            model.lock().unwrap().handle(a);
        }
    })
    .detach();
}

fn stop_model(model: &Mutex<Model>, del_at_exit: bool) -> Result<()> {
    let mut settings_reader = SettingsReader::new()?;
    if del_at_exit {
        let model = model.lock().unwrap();
        settings_reader.delete(&model.username)?;
    }
    Ok(())
}

fn center_window(window: &Window) {
    if let Some(new_pos) = window
        .with_winit_window(|window| {
            window.primary_monitor().map(|monitor| {
                let monitor_pos = monitor.position();
                let monitor_size = monitor.size();
                let window_size = window.outer_size();
                PhysicalPosition {
                    x: monitor_pos.x + ((monitor_size.width - window_size.width) / 2) as i32,
                    y: monitor_pos.y + ((monitor_size.height - window_size.height) / 2) as i32,
                }
            })
        })
        .flatten()
    {
        window.set_position(new_pos);
    }
}

fn accent_color() -> color_theme::Color {
    color_theme::Color::accent().unwrap_or(color_theme::Color {
        r: 0,
        g: 120,
        b: 212,
    })
}

#[cfg(target_os = "linux")]
fn wayland_set_app_id() -> Result<(), slint::platform::SetPlatformError> {
    use i_slint_backend_winit::winit::platform::wayland::WindowAttributesExtWayland;

    let backend = i_slint_backend_winit::Backend::builder()
        .with_window_attributes_hook(|attr| attr.with_name("io.github.berrysoft.tunet", ""))
        .build()
        .unwrap();

    slint::platform::set_platform(Box::new(backend))
}
