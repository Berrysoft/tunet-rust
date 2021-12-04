use crate::ping;
use lazy_static::lazy_static;
use netstatus::NetStatus;
use std::collections::BTreeMap;
use tunet_helper::*;

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

pub async fn suggest(client: &HttpClient) -> NetState {
    suggest_with_status(client, NetStatus::current()).await
}

pub async fn suggest_with_status(client: &HttpClient, s: NetStatus) -> NetState {
    let state = match s {
        NetStatus::Unknown => None,
        NetStatus::Wwan => Some(NetState::Unknown),
        NetStatus::Wlan(ssid) => SUGGEST_SSID_MAP.get(ssid.as_str()).copied(),
        NetStatus::Lan => Some(NetState::Auth4),
    };
    match state {
        Some(state) => state,
        None => ping::suggest(client).await,
    }
}
