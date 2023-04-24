use super::*;
use objc::{
    runtime::{Class, Object},
    *,
};
use std::ptr::{addr_of_mut, null_mut};

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    #[link_name = "OBJC_CLASS_$_NSColor"]
    static OBJC_CLASS__NSColor: Class;

    #[link_name = "OBJC_CLASS_$_NSColorSpace"]
    static OBJC_CLASS__NSColorSpace: Class;
}

pub fn accent() -> Color {
    unsafe {
        let accent: *mut Object = msg_send![&OBJC_CLASS__NSColor, controlAccentColor];
        let color_space: *mut Object = msg_send![&OBJC_CLASS__NSColorSpace, genericRGBColorSpace];
        let accent: *mut Object = msg_send![accent, colorUsingColorSpace: color_space];
        let mut r: f64 = 0.0;
        let mut g: f64 = 0.0;
        let mut b: f64 = 0.0;
        let _: () = msg_send![accent, getRed:addr_of_mut!(r) green:addr_of_mut!(g) blue:addr_of_mut!(b) alpha:null_mut::<f64>()];
        Color {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }
}
