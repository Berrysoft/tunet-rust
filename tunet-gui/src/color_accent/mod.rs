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

#[derive(Debug, Clone, Copy)]
pub struct CairoColor(pub f64, pub f64, pub f64);

impl CairoColor {
    pub fn accent() -> Self {
        platform::accent()
    }
}
