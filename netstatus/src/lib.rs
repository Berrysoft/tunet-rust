#![feature(option_result_unwrap_unchecked)]

#[cfg(target_os = "windows")]
mod winrt;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod sc;

#[cfg(target_os = "linux")]
mod libiw;

mod platform {
    #[cfg(target_os = "windows")]
    pub use super::winrt::*;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub use super::sc::*;

    #[cfg(target_os = "linux")]
    pub use super::libiw::*;
}

pub enum NetStatus {
    Unknown,
    Wwan,
    Wlan(String),
    Lan,
}

impl NetStatus {
    pub fn current() -> Self {
        platform::current()
    }
}
