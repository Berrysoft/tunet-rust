#[cfg(target_os = "windows")]
mod winrt;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod stub;

mod platform {
    #[cfg(target_os = "windows")]
    pub use super::winrt::*;

    #[cfg(target_os = "macos")]
    pub use super::mac::*;

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    pub use super::stub::*;
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
