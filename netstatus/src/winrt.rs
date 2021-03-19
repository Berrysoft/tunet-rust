use crate::*;
use winrt_bindings::windows::networking::connectivity::*;

fn current_impl() -> windows::Result<NetStatus> {
    let profile = NetworkInformation::get_internet_connection_profile()?;
    let cl = profile.get_network_connectivity_level()?;
    match cl {
        NetworkConnectivityLevel::None => Ok(NetStatus::Unknown),
        _ => {
            if profile.is_wlan_connection_profile()? {
                let ssid = profile
                    .wlan_connection_profile_details()?
                    .get_connected_ssid()?;
                Ok(NetStatus::Wlan(ssid.to_string_lossy()))
            } else if profile.is_wwan_connection_profile()? {
                Ok(NetStatus::Wwan)
            } else {
                Ok(NetStatus::Lan)
            }
        }
    }
}

pub fn current() -> NetStatus {
    current_impl().unwrap_or_else(|_e| {
        #[cfg(debug_assertions)]
        eprintln!("WARNING: {}", _e.message());
        NetStatus::Unknown
    })
}
