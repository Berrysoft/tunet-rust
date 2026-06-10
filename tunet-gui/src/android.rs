use android_activity::AndroidApp;
use winio::prelude::*;

use crate::MainModel;

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

pub(crate) fn init_rustls(window: &Window) -> Result<()> {
    let context = window.as_window().to_android();
    let vm = jni::JavaVM::singleton()?;
    vm.attach_current_thread(|env| {
        let context = env.new_local_ref(context)?;
        rustls_platform_verifier::android::init_with_env(env, context)
    })?;
    Ok(())
}
