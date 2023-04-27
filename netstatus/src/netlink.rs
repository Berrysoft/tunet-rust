use crate::*;
use netlink_wi::{AttrParseError, NlSocket};

type Result<T> = std::result::Result<T, AttrParseError>;

fn current_impl() -> Result<NetStatus> {
    if let Ok(sock) = NlSocket::connect() {
        if let Ok(interfaces) = sock.list_interfaces() {
            for interface in interfaces {
                let interface = interface?;
                if let Some(ssid) = interface.ssid {
                    return Ok(NetStatus::Wlan(ssid));
                }
            }
        }
    }
    Ok(NetStatus::Unknown)
}

pub fn current() -> NetStatus {
    current_impl().unwrap_or_else(|e| {
        if cfg!(debug_assertions) {
            eprintln!("WARNING: {}", e);
        }
        NetStatus::Unknown
    })
}

pub fn watch() -> impl Stream<Item = ()> {
    tokio_stream::pending()
}
