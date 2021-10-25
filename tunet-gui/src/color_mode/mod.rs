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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    Light,
    Dark,
}

impl ColorMode {
    pub fn preferred() -> Self {
        platform::preferred()
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }
}
