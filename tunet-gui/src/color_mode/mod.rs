#[cfg(windows)]
mod winrt;

#[cfg(not(windows))]
mod stub;

mod platform {
    #[cfg(windows)]
    pub use super::winrt::*;

    #[cfg(not(windows))]
    pub use super::stub::*;
}

pub enum ColorMode {
    Light,
    Dark,
}

impl ColorMode {
    pub fn preferred() -> Self {
        platform::preferred()
    }

    pub fn is_dark(&self) -> bool {
        if let Self::Dark = self {
            true
        } else {
            false
        }
    }
}
