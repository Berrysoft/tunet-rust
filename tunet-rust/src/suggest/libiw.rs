use super::SUGGEST_SSID_MAP;
use crate::*;
use libiw_bindings::*;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int};

#[repr(transparent)]
struct Socket(c_int);

impl Drop for Socket {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe { iw_sockets_close(self.0) }
        }
    }
}

unsafe extern "C" fn enum_handler(
    skfd: c_int,
    ifname: *mut c_char,
    args: *mut *mut c_char,
    _count: c_int,
) -> c_int {
    #[cfg(debug_assertions)]
    assert_eq!(_count, 1);
    let ssids = ((*args) as *mut Vec<String>).as_mut().unwrap_unchecked();
    let mut info = MaybeUninit::uninit().assume_init();
    iw_get_basic_config(skfd, ifname, &mut info);
    if info.has_essid != 0 {
        let mut dest = [0; 129];
        iw_essid_escape(dest.as_mut_ptr(), info.essid.as_ptr(), info.essid_len);
        ssids.push(CStr::from_ptr(dest.as_ptr()).to_string_lossy().into_owned());
    }
    0
}

pub fn suggest() -> NetState {
    unsafe {
        let skfd = Socket(iw_sockets_open());
        if skfd.0 < 0 {
            return NetState::Unknown;
        }
        let mut ssids: Vec<String> = vec![];
        let mut args = [&mut ssids as *mut _ as *mut c_char];
        iw_enum_devices(skfd.0, Some(enum_handler), args.as_mut_ptr(), 1);
        for ssid in ssids {
            if let Some(s) = SUGGEST_SSID_MAP.get(ssid.as_str()) {
                return *s;
            }
        }
    }
    NetState::Unknown
}
