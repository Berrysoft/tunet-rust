use crate::*;
use libiw_bindings::*;
use std::collections::VecDeque;
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
    let ssids = ((*args) as *mut VecDeque<String>).as_mut().unwrap();
    let mut info = MaybeUninit::uninit().assume_init();
    iw_get_basic_config(skfd, ifname, &mut info);
    if info.has_essid != 0 {
        ssids.push_back(
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                info.essid.as_ptr() as _,
                info.essid_len as _,
            ))
            .to_owned(),
        );
    }
    0
}

pub fn current() -> NetStatus {
    unsafe {
        let skfd = Socket(iw_sockets_open());
        if skfd.0 < 0 {
            NetStatus::Unknown
        } else {
            let mut ssids = VecDeque::new();
            let mut args = [&mut ssids as *mut _ as *mut c_char];
            iw_enum_devices(skfd.0, Some(enum_handler), args.as_mut_ptr(), 1);
            match ssids.pop_front() {
                Some(ssid) => NetStatus::Wlan(ssid),
                None => NetStatus::Unknown,
            }
        }
    }
}
