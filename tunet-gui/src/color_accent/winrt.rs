use super::*;
use winrt_bindings::{windows::Result, Windows::UI::ViewManagement::*};

fn accent_impl() -> Result<CairoColor> {
    let settings = UISettings::new()?;
    let accent = settings.GetColorValue(UIColorType::Accent)?;
    Ok(CairoColor(
        accent.R as f64 / 255.,
        accent.G as f64 / 255.,
        accent.B as f64 / 255.,
    ))
}

pub fn accent() -> CairoColor {
    accent_impl().unwrap_or_else(|e| {
        if cfg!(debug_assertions) {
            eprintln!("WARNING: {}", e.message());
        }
        CairoColor(0., 120. / 255., 215. / 255.)
    })
}
