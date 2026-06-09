#[cfg(not(target_os = "android"))]
fn main() -> main::Result<()> {
    use main::MainModel;
    use winio::prelude::*;
    App::builder()
        .name("io.github.berrysoft.tunet")
        .build()?
        .block_on(MainModel::run_until_event(()))
}

#[cfg(target_os = "android")]
fn main() {
    unreachable!("Android entry point is `android_main` in `android.rs`")
}
