mod ping;

#[cfg(feature = "netstatus")]
mod ssid_map;

#[cfg(feature = "netstatus")]
pub use ssid_map::*;

#[cfg(not(feature = "netstatus"))]
pub use ping::*;
