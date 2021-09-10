#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod stub;

mod platform {
    #[cfg(target_os = "windows")]
    pub use super::windows::Keyring;

    #[cfg(target_os = "macos")]
    pub use super::mac::Keyring;

    #[cfg(target_os = "linux")]
    pub use super::linux::Keyring;

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub use super::stub::Keyring;
}

pub use platform::*;
