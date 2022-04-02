use std::fmt::{Display, Formatter};

#[cfg_attr(
    target_os = "windows",
    path = "winrt.rs",
    cfg_attr(
        target_os = "linux",
        path = "netlink.rs",
        cfg_attr(target_os = "macos", path = "sc.rs", path = "stub.rs")
    )
)]
mod platform;

#[derive(Debug, Clone, PartialEq, Eq)]
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
