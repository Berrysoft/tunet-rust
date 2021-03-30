#![allow(non_camel_case_types)]

use crate::*;
use libc::{close, IFNAMSIZ};
use libloading::Library;
use std::collections::VecDeque;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_double, c_int, c_uchar};

pub type iw_enum_handler =
    unsafe extern "C" fn(c_int, *mut c_char, *mut *mut c_char, c_int) -> c_int;

pub type iw_sockets_open_fn = unsafe extern "C" fn() -> c_int;
pub type iw_enum_devices_fn =
    unsafe extern "C" fn(skfd: c_int, fn_: iw_enum_handler, args: *mut *mut c_char, count: c_int);
pub type iw_get_basic_config_fn =
    unsafe extern "C" fn(skfd: c_int, ifname: *const c_char, info: *mut wireless_config) -> c_int;

const IW_ENCODING_TOKEN_MAX: usize = 64;
const IW_ESSID_MAX_SIZE: usize = 32;

#[repr(C)]
pub struct wireless_config {
    pub name: [c_char; IFNAMSIZ + 1],
    pub has_nwid: c_int,
    pub nwid: iw_param,
    pub has_freq: c_int,
    pub freq: c_double,
    pub freq_flags: c_int,
    pub has_key: c_int,
    pub key: [c_uchar; IW_ENCODING_TOKEN_MAX],
    pub key_size: c_int,
    pub key_flags: c_int,
    pub has_essid: c_int,
    pub essid_on: c_int,
    pub essid: [c_char; IW_ESSID_MAX_SIZE + 2],
    pub essid_len: c_int,
    pub has_mode: c_int,
    pub mode: c_int,
}

#[repr(C)]
pub struct iw_param {
    pub value: i32,
    pub fixed: u8,
    pub disabled: u8,
    pub flags: u16,
}

#[repr(transparent)]
struct Socket(c_int);

impl Drop for Socket {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe {
                close(self.0);
            }
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
    assert_eq!(_count, 2);
    let lib = ((*args) as *const Library).as_ref().unwrap();
    if let Ok(fnconf) = lib.get::<iw_get_basic_config_fn>(b"iw_get_basic_config\0") {
        let ssids = ((*args.add(1)) as *mut VecDeque<String>).as_mut().unwrap();
        let mut info = MaybeUninit::uninit().assume_init();
        fnconf(skfd, ifname, &mut info);
        if info.has_essid != 0 {
            ssids.push_back(
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    info.essid.as_ptr() as _,
                    info.essid_len as _,
                ))
                .to_owned(),
            );
        }
    }
    0
}

pub fn current() -> NetStatus {
    unsafe {
        if let Ok(lib) = Library::new("libiw.so") {
            if let Ok(fnopen) = lib.get::<iw_sockets_open_fn>(b"iw_sockets_open\0") {
                let skfd = Socket(fnopen());
                if skfd.0 >= 0 {
                    let mut ssids = VecDeque::new();
                    if let Ok(fnenum) = lib.get::<iw_enum_devices_fn>(b"iw_enum_devices\0") {
                        let mut args = [
                            &lib as *const _ as *mut c_char,
                            &mut ssids as *mut _ as *mut c_char,
                        ];
                        fnenum(skfd.0, enum_handler, args.as_mut_ptr(), args.len() as _);
                    }
                    if let Some(ssid) = ssids.pop_front() {
                        return NetStatus::Wlan(ssid);
                    }
                }
            }
        }
        NetStatus::Unknown
    }
}
