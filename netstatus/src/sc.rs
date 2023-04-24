use crate::*;
use objc::{
    runtime::{Class, Object},
    *,
};
use std::ffi::CStr;
use system_configuration::network_reachability::{ReachabilityFlags, SCNetworkReachability};

#[link(name = "CoreWLAN", kind = "framework")]
extern "C" {
    #[link_name = "OBJC_CLASS_$_CWWiFiClient"]
    static OBJC_CLASS__CWWiFiClient: Class;
}

unsafe fn get_ssid() -> Option<String> {
    let client: *mut Object = msg_send![&OBJC_CLASS__CWWiFiClient, sharedWiFiClient];
    let interface: *mut Object = msg_send![client, interface];
    let name: *mut Object = msg_send![interface, ssid];
    if !name.is_null() {
        let name = std::ffi::CStr::from_ptr(msg_send![name, UTF8String])
            .to_string_lossy()
            .into_owned();
        Some(name)
    } else {
        None
    }
}

pub fn current() -> NetStatus {
    let host = unsafe { CStr::from_bytes_with_nul_unchecked(b"0.0.0.0\0") };
    if let Some(sc) = SCNetworkReachability::from_host(host) {
        if let Ok(flag) = sc.reachability() {
            if !flag.contains(ReachabilityFlags::REACHABLE) {
                return NetStatus::Unknown;
            }
            if !flag.contains(ReachabilityFlags::CONNECTION_REQUIRED)
                || ((flag.contains(ReachabilityFlags::CONNECTION_ON_DEMAND)
                    || flag.contains(ReachabilityFlags::CONNECTION_ON_TRAFFIC))
                    && !flag.contains(ReachabilityFlags::INTERVENTION_REQUIRED))
            {
                return match unsafe { get_ssid() } {
                    Some(ssid) => NetStatus::Wlan(ssid),
                    None => NetStatus::Unknown,
                };
            }
            if flag == ReachabilityFlags::IS_WWAN {
                return NetStatus::Wwan;
            }
        }
    }
    NetStatus::Unknown
}
