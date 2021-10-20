use super::*;
use winrt_bindings::{
    windows::Result,
    Windows::UI::{ViewManagement::*, *},
};

fn preferred_impl() -> Result<ColorMode> {
    let settings = UISettings::new()?;
    let background = settings.GetColorValue(UIColorType::Background)?;
    let mode = if (background
        == Color {
            A: 255,
            R: 0,
            G: 0,
            B: 0,
        }) {
        ColorMode::Dark
    } else {
        ColorMode::Light
    };
    Ok(mode)
}

pub fn preferred() -> ColorMode {
    preferred_impl().unwrap_or_else(|e| {
        if cfg!(debug_assertions) {
            eprintln!("WARNING: {}", e.message());
        }
        ColorMode::Light
    })
}
