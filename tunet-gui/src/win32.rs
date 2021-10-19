use crate::*;
use libloading::os::windows::*;
use std::ffi::c_void;

fn get<T>(this: &Option<Library>, symbol: &[u8]) -> Option<Symbol<T>> {
    match this {
        Some(this) => match unsafe { this.get::<Option<T>>(symbol) } {
            Ok(s) => s.lift_option(),
            Err(_) => None,
        },
        None => None,
    }
}

extern "C" {
    fn gdk_win32_surface_get_handle(surface: *mut c_void) -> *mut c_void;
}

type FnGetDpiForWindow = unsafe extern "system" fn(*mut c_void) -> u32;

lazy_static! {
    static ref USER32: Option<Library> =
        unsafe { Library::load_with_flags("User32.dll", LOAD_LIBRARY_SEARCH_SYSTEM32).ok() };
    static ref GET_DPI_FOR_WINDOW: Option<Symbol<FnGetDpiForWindow>> =
        get(&USER32, b"GetDpiForWindow\0");
}

pub fn get_scale_factor(wnd: impl IsA<gtk::Widget>) -> f64 {
    if let Some(native) = wnd.native() {
        if let Some(surface) = native.surface() {
            if let Some(ref f) = *GET_DPI_FOR_WINDOW {
                let hwnd = unsafe { gdk_win32_surface_get_handle(surface.as_ptr() as _) };
                return unsafe { f(hwnd) } as f64 / 96.0;
            }
        }
    }
    1.0
}
