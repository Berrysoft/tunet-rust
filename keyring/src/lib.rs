cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[path = "windows.rs"]
        mod platform;
    } else if #[cfg(all(target_os = "linux", target_env = "gnu"))] {
        #[path = "linux.rs"]
        mod platform;
    } else if #[cfg(target_os = "macos")] {
        #[path = "mac.rs"]
        mod platform;
    } else {
        #[path = "stub.rs"]
        mod platform;
    }
}

pub use platform::*;
