mod ping;

use crate::*;
use lazy_static::lazy_static;
use netstatus::NetStatus;
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

pub fn suggest(client: &HttpClient) -> NetState {
    let state = match NetStatus::current() {
        NetStatus::Unknown | NetStatus::Wwan => NetState::Unknown,
        NetStatus::Wlan(ssid) => SUGGEST_SSID_MAP
            .get(ssid.as_str())
            .copied()
            .unwrap_or(NetState::Unknown),
        NetStatus::Lan => NetState::Auth4,
    };
    match state {
        NetState::Unknown => ping::suggest(client),
        _ => state,
    }
}
