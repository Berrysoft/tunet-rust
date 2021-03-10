mod ping;

use crate::*;
use lazy_static::lazy_static;
use std::collections::BTreeMap;

lazy_static! {
    static ref SUGGEST_SSID_MAP: BTreeMap<&'static str, NetState> = {
        let mut map = BTreeMap::new();
        map.insert("Tsinghua", NetState::Auth4);
        map.insert("Tsinghua-5G", NetState::Auth4);
        map.insert("Tsinghua-IPv4", NetState::Auth4);
        map.insert("Tsinghua-IPv6", NetState::Auth6);
        map.insert("Tsinghua-Secure", NetState::Net);
        map.insert("Wifi.郑裕彤讲堂", NetState::Net);
        map
    };
}

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

pub fn suggest(client: &HttpClient) -> NetState {
    match platform::suggest() {
        NetState::Unknown => ping::suggest(client),
        state => state,
    }
}
