cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[path = "winrt.rs"]
        mod platform;
    } else if #[cfg(target_os = "macos")] {
        #[path = "mac.rs"]
        mod platform;
    } else {
        #[path = "stub.rs"]
        mod platform;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn accent() -> Self {
        platform::accent()
    }
}
