use crate::*;
use libloading::os::windows::*;

fn get<T>(this: &Option<Library>, symbol: &[u8]) -> Option<Symbol<T>> {
    match this {
        Some(this) => match unsafe { this.get::<Option<T>>(symbol) } {
            Ok(s) => s.lift_option(),
            Err(_) => None,
        },
        None => None,
    }
}

type FnGetDpiForSystem = unsafe extern "system" fn() -> u32;

lazy_static! {
    static ref USER32: Option<Library> =
        unsafe { Library::load_with_flags("User32.dll", LOAD_LIBRARY_SEARCH_SYSTEM32).ok() };
    static ref GET_DPI_FOR_SYSTEM: Option<Symbol<FnGetDpiForSystem>> =
        get(&USER32, b"GetDpiForSystem\0");
}

pub fn get_scale_factor() -> f64 {
    if let Some(ref f) = *GET_DPI_FOR_SYSTEM {
        return unsafe { f() } as f64 / 96.0;
    }
    1.0
}
