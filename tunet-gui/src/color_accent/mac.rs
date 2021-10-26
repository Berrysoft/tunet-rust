use super::*;
use objc::{
    rc::StrongPtr,
    runtime::{Class, Object},
    *,
};
use std::ptr::{addr_of_mut, null_mut};

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    #[link_name = "OBJC_CLASS_$_NSColor"]
    static OBJC_CLASS__NSColor: Class;
}

pub fn accent() -> CairoColor {
    let accent = StrongPtr::new(msg_send![&OBJC_CLASS__NSColor, controlAccentColor]);
    let mut r: f64 = 0;
    let mut g: f64 = 0;
    let mut b: f64 = 0;
    let _: () = msg_send![*accent, getRed:addr_of_mut!(r) green:addr_of_mut!(g) blue:addr_of_mut!(b) alpha:null_mut::<f64>()];
    CairoColor(r, g, b)
}
