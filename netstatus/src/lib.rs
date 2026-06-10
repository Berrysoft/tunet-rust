use std::fmt::{Display, Formatter};

use futures_util::Stream;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[path = "winrt.rs"]
        mod platform;
    } else if #[cfg(target_os = "linux")] {
        #[path = "netlink.rs"]
        mod platform;
    } else if #[cfg(target_vendor = "apple")] {
        #[path = "sc.rs"]
        mod platform;
    } else if #[cfg(target_os = "android")] {
        #[path = "android.rs"]
        mod platform;
    } else {
        #[path = "stub.rs"]
        mod platform;
    }
}

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

    pub fn watch() -> impl Stream<Item = ()> {
        platform::watch()
    }
}

impl Display for NetStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => f.pad("未知"),
            Self::Wwan => f.pad("移动流量"),
            Self::Wlan(ssid) => {
                if ssid.is_empty() {
                    f.pad("无线网络")
                } else {
                    f.pad(ssid)
                }
            }
            Self::Lan => f.pad("有线网络"),
        }
    }
}
