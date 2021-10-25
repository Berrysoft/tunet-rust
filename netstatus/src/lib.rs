use std::fmt::{Display, Formatter};

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

impl Display for NetStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => f.pad("未知"),
            Self::Wwan => f.pad("移动流量"),
            Self::Wlan(ssid) => f.pad(&format!("无线网络（{}）", ssid)),
            Self::Lan => f.pad("有线网络"),
        }
    }
}
