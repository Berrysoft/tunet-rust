#[cfg_attr(
    target_os = "windows",
    path = "windows.rs",
    cfg_attr(
        all(target_os = "linux", target_env = "gnu"),
        path = "linux.rs",
        cfg_attr(target_os = "macos", path = "mac.rs", path = "stub.rs")
    )
)]
mod platform;

pub use platform::*;
