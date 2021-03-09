use super::SUGGEST_SSID_MAP;
use crate::*;
use winrt_bindings::windows::networking::connectivity::*;

fn suggest_impl() -> winrt_bindings::windows::Result<NetState> {
    let profile = NetworkInformation::get_internet_connection_profile()?;
    let cl = profile.get_network_connectivity_level()?;
    if cl == NetworkConnectivityLevel::None {
        Ok(NetState::Unknown)
    } else {
        if profile.is_wlan_connection_profile()? {
            let ssid = profile
                .wlan_connection_profile_details()?
                .get_connected_ssid()?;
            Ok(SUGGEST_SSID_MAP
                .get(ssid.to_string_lossy().as_str())
                .copied()
                .unwrap_or(NetState::Unknown))
        } else if profile.is_wwan_connection_profile()? {
            Ok(NetState::Unknown)
        } else {
            Ok(NetState::Auth4)
        }
    }
}

pub fn suggest() -> NetState {
    suggest_impl().unwrap_or_else(|_e| {
        #[cfg(debug_assertions)]
        eprintln!("WARNING: {}", _e.message());
        NetState::Unknown
    })
}
