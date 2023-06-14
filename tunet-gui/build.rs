fn main() {
    slint_build::compile("ui/main.slint").unwrap();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winres::WindowsResource::new()
            .set_icon("../logo.ico")
            .compile()
            .unwrap();
    }
}
