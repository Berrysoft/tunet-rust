use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tunet_helper::Result;
use tunet_model::{Action, Model, UpdateMsg};
use tunet_settings::FileSettingsReader;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::new()?;

    let color = color_theme::Color::accent();
    let home_model = app.global::<HomeModel>();
    home_model.set_theme_color(slint::Color::from_argb_u8(255, color.r, color.g, color.b));
    home_model.set_theme_color_t1(slint::Color::from_argb_u8(191, color.r, color.g, color.b));
    home_model.set_theme_color_t2(slint::Color::from_argb_u8(140, color.r, color.g, color.b));

    app.global::<AboutModel>()
        .set_version(env!("CARGO_PKG_VERSION").into());

    let (tx, mut rx) = mpsc::channel(32);
    let model = Arc::new(Mutex::new(Model::new(tx)?));
    {
        let weak_app = app.as_weak();
        let weak_model = Arc::downgrade(&model);
        let mut model = model.lock().await;
        model.update = Some(Box::new(move |msg| {
            if let Some(model) = weak_model.upgrade() {
                let weak_app = weak_app.clone();
                tokio::spawn(async move {
                    let model = model.lock().await;
                    update(&model, msg, weak_app)
                });
            }
        }));

        let cred = Arc::new(FileSettingsReader::new()?.read_with_password()?);
        model.queue(Action::Credential(cred));
        model.queue(Action::Timer);
    }

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
        _ => {}
    };
}
