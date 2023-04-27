fn main() {
    slint_build::compile("ui/main.slint").unwrap();
    #[cfg(target_os = "windows")]
    {
        winres::WindowsResource::new()
            .set_icon("../logo.ico")
            .compile()
            .unwrap();
    }
}
