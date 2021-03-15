mod ping;

#[cfg(feature = "netstatus")]
mod ssid_map;

#[cfg(feature = "netstatus")]
pub use ssid_map::suggest;

#[cfg(not(feature = "netstatus"))]
pub use ping::suggest;
