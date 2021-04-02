use crate::*;
use netlink_wi::NlSocket;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn current_impl() -> Result<NetStatus> {
    let sock = NlSocket::connect()?;
    let interfaces = sock.list_interfaces()?;
    for interface in interfaces {
        let interface = interface?;
        if let Some(ssid) = interface.ssid {
            return Ok(NetStatus::Wlan(ssid));
        }
    }
    Ok(NetStatus::Unknown)
}

pub fn current() -> NetStatus {
    current_impl().unwrap_or_else(|_e| {
        #[cfg(debug_assertions)]
        eprintln!("WARNING: {}", _e);
        NetStatus::Unknown
    })
}
