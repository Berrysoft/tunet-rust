#[cfg(target_os = "windows")]
mod winrt;

#[cfg(target_os = "macos")]
mod sc;

#[cfg(target_os = "linux")]
mod netlink;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod stub;

mod platform {
    #[cfg(target_os = "windows")]
    pub use super::winrt::*;

    #[cfg(target_os = "macos")]
    pub use super::sc::*;

    #[cfg(target_os = "linux")]
    pub use super::netlink::*;

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub use super::stub::*;
}

#[derive(Debug, PartialEq, Eq)]
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
