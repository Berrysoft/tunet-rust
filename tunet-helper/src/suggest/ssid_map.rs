use crate::{suggest::ping, *};
use netstatus::NetStatus;
use std::collections::BTreeMap;
use std::sync::LazyLock;

static SUGGEST_SSID_MAP: LazyLock<BTreeMap<&'static str, NetState>> = LazyLock::new(|| {
    BTreeMap::from([
        ("Tsinghua", NetState::Auth4),
        ("Tsinghua-5G", NetState::Auth4),
        ("Tsinghua-IPv4", NetState::Auth4),
        ("Tsinghua-IPv6", NetState::Auth6),
        ("Tsinghua-Secure", NetState::Auth4),
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
