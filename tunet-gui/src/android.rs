use std::sync::Mutex;

use android_activity::AndroidApp;
use jni::{Env, objects::JObject};
use tunet_model::Action;
use winio::prelude::*;

use crate::{MainMessage, MainModel};

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_filter(
                android_logger::FilterBuilder::new()
                    .parse("warn,tunet=trace")
                    .build(),
            ),
    );

    let config_dir = app.internal_data_path();
    let app = App::builder()
        .android_app(app)
        .build()
        .expect("cannot create app");
    app.spawn(|| async {
        if let Err(e) = MainModel::run_until_event(config_dir).await {
            log::error!("App error: {e:?}");
        }
    })
}

static MAIN_MODEL_SENDER: Mutex<Option<ComponentSender<MainModel>>> = Mutex::new(None);

pub(crate) fn set_sender(sender: &ComponentSender<MainModel>) {
    let mut guard = MAIN_MODEL_SENDER.lock().unwrap();
    *guard = Some(sender.clone());
}

jni::bind_java_type! {
    MainActivity => io.github.berrysoft.tunet.MainActivity,
    native_methods {
        extern fn on_create_native(),
        extern fn on_location_permission_granted(),
    }
}

impl MainActivityNativeInterface for MainActivityAPI {
    type Error = jni::errors::Error;

    fn on_create_native<'local>(
        env: &mut Env<'local>,
        this: MainActivity<'local>,
    ) -> std::result::Result<(), Self::Error> {
        let context = env.cast_local::<JObject>(this)?;
        rustls_platform_verifier::android::init_with_env(env, context)
    }

    fn on_location_permission_granted<'local>(
        _env: &mut Env<'local>,
        _this: MainActivity<'local>,
    ) -> std::result::Result<(), Self::Error> {
        let sender = MAIN_MODEL_SENDER.lock().unwrap();
        if let Some(sender) = sender.as_ref() {
            sender.post(MainMessage::Action(Action::Status(None)));
        }
        Ok(())
    }
}
