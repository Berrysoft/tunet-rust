use crate::*;
use winrt_bindings::{windows::Result, Windows::Networking::Connectivity::*};

fn current_impl() -> Result<NetStatus> {
    let profile = NetworkInformation::GetInternetConnectionProfile()?;
    let cl = profile.GetNetworkConnectivityLevel()?;
    match cl {
        NetworkConnectivityLevel::None => Ok(NetStatus::Unknown),
        _ => {
            if profile.IsWlanConnectionProfile()? {
                let ssid = profile.WlanConnectionProfileDetails()?.GetConnectedSsid()?;
                Ok(NetStatus::Wlan(ssid.to_string_lossy()))
            } else if profile.IsWwanConnectionProfile()? {
                Ok(NetStatus::Wwan)
            } else {
                Ok(NetStatus::Lan)
            }
        }
    }
}

pub fn current() -> NetStatus {
    current_impl().unwrap_or_else(|e| {
        if cfg!(debug_assertions) {
            eprintln!("WARNING: {}", e.message());
        }
        NetStatus::Unknown
    })
}
