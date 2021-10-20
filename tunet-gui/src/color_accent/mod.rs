#[cfg(target_os = "windows")]
mod winrt;

#[cfg(not(target_os = "windows"))]
mod stub;

mod platform {
    #[cfg(target_os = "windows")]
    pub use super::winrt::*;

    #[cfg(not(target_os = "windows"))]
    pub use super::stub::*;
}

#[derive(Debug, Clone, Copy)]
pub struct CairoColor(pub f64, pub f64, pub f64);

impl CairoColor {
    pub fn accent() -> Self {
        platform::accent()
    }
}
