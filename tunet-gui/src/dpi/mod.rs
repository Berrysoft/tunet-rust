#[cfg(windows)]
mod win32;

#[cfg(not(windows))]
mod stub;

mod platform {
    #[cfg(windows)]
    pub use super::win32::*;

    #[cfg(not(windows))]
    pub use super::stub::*;
}

pub use platform::get_scale_factor;
