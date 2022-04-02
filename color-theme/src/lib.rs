#[cfg_attr(
    target_os = "windows",
    path = "winrt.rs",
    cfg_attr(target_os = "macos", path = "mac.rs", path = "stub.rs")
)]
mod platform;

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
