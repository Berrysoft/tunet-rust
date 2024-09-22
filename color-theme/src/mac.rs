use super::*;
use objc2_app_kit::{NSColor, NSColorSpace};
use std::ptr::null_mut;

pub fn accent() -> Option<Color> {
    unsafe {
        let accent = NSColor::controlAccentColor();
        let color_space = NSColorSpace::genericRGBColorSpace();
        let accent = accent.colorUsingColorSpace(&color_space);
        accent.map(|accent| {
            let mut r: f64 = 0.0;
            let mut g: f64 = 0.0;
            let mut b: f64 = 0.0;
            accent.getRed_green_blue_alpha(&mut r, &mut g, &mut b, null_mut());
            Color {
                r: (r * 255.0) as u8,
                g: (g * 255.0) as u8,
                b: (b * 255.0) as u8,
            }
        })
    }
}
