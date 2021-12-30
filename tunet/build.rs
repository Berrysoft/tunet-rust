fn main() {
    #[cfg(windows)]
    {
        winres::WindowsResource::new()
            .set_icon("logo.ico")
            .compile()
            .unwrap();
    }
}
