use super::*;
use winrt_bindings::{windows::Result, Windows::UI::ViewManagement::*};

fn accent_impl() -> Result<Color> {
    let settings = UISettings::new()?;
    let accent = settings.GetColorValue(UIColorType::Accent)?;
    Ok(Color {
        r: accent.R,
        g: accent.G,
        b: accent.B,
    })
}

pub fn accent() -> Color {
    accent_impl().unwrap_or_else(|e| {
        if cfg!(debug_assertions) {
            eprintln!("WARNING: {}", e.message());
        }
        Color {
            r: 0,
            g: 120,
            b: 215,
        }
    })
}
