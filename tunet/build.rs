fn main() {
    #[cfg(target_os = "windows")]
    {
        winres::WindowsResource::new()
            .set_icon("../logo.ico")
            .compile()
            .unwrap();
    }
}
