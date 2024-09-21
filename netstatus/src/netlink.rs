use crate::*;
use netlink_wi::NlSocket;

pub fn current() -> NetStatus {
    if let Ok(mut sock) = NlSocket::connect() {
        if let Ok(interfaces) = sock.list_interfaces() {
            for interface in interfaces {
                if let Some(ssid) = interface.ssid {
                    return NetStatus::Wlan(ssid);
                }
            }
        }
    }
    NetStatus::Unknown
}

pub fn watch() -> impl Stream<Item = ()> {
    futures_util::stream::pending()
}
