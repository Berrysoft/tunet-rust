#![feature(option_result_unwrap_unchecked)]

#[cfg(target_os = "windows")]
mod winrt;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod libiw;

mod platform {
    #[cfg(target_os = "windows")]
    pub use super::winrt::*;

    #[cfg(target_os = "macos")]
    pub use super::macos::*;

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
