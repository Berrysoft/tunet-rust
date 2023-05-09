use super::*;
use windows::{core::*, UI::ViewManagement::*};

fn accent_impl() -> Result<Color> {
    let settings = UISettings::new()?;
    let accent = settings.GetColorValue(UIColorType::Accent)?;
    Ok(Color {
        r: accent.R,
        g: accent.G,
        b: accent.B,
    })
}

pub fn accent() -> Option<Color> {
    match accent_impl() {
        Ok(c) => Some(c),
        Err(e) => {
            log::warn!("{}", e.message());
            None
        }
    }
}
