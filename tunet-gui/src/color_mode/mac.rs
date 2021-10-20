use super::*;
use objc::{
    rc::StrongPtr,
    runtime::{Class, Object},
    *,
};

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    #[link_name = "OBJC_CLASS_$_NSString"]
    static OBJC_CLASS__NSString: Class;
    #[link_name = "OBJC_CLASS_$_NSUserDefaults"]
    static OBJC_CLASS__NSUserDefaults: Class;
}

pub fn preferred() -> ColorMode {
    let defaults = StrongPtr::new(msg_send![&OBJC_CLASS__NSUserDefaults, standardUserDefaults]);
    let key = StrongPtr::new(
        msg_send![&OBJC_CLASS__NSString, stringWithUTF8String:b"AppleInterfaceStyle\0".as_ptr()],
    );
    let mode: *mut Object = msg_send![*defaults, stringForKey:*key];
    if !mode.is_null() {
        let mode = StrongPtr::new(mode);
        let mode = std::ffi::CStr::from_ptr(msg_send![*mode, UTF8String]).to_string_lossy();
        if &mode == "Dark" {
            return ColorMode::Dark;
        }
    }
    ColorMode::Light
}
