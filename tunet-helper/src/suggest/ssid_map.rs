use crate::{suggest::ping, *};
use netstatus::NetStatus;
use once_cell::sync::Lazy;
use std::collections::BTreeMap;

static SUGGEST_SSID_MAP: Lazy<BTreeMap<&'static str, NetState>> = Lazy::new(|| {
    BTreeMap::from([
        ("Tsinghua", NetState::Auth4),
        ("Tsinghua-5G", NetState::Auth4),
        ("Tsinghua-IPv4", NetState::Auth4),
        ("Tsinghua-IPv6", NetState::Auth6),
        ("Tsinghua-Secure", NetState::Net),
        ("Wifi.郑裕彤讲堂", NetState::Net),
    ])
});

pub async fn suggest(client: &HttpClient) -> NetState {
    suggest_with_status(client, &NetStatus::current()).await
}

pub async fn suggest_with_status(client: &HttpClient, s: &NetStatus) -> NetState {
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
