#![allow(non_camel_case_types)]

use libc::{close, IFNAMSIZ};
use std::os::raw::{c_char, c_double, c_int, c_uchar};

pub type iw_enum_handler =
    Option<unsafe extern "C" fn(c_int, *mut c_char, *mut *mut c_char, c_int) -> c_int>;

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

#[link(name = "iw")]
extern "C" {
    pub fn iw_sockets_open() -> c_int;
    pub fn iw_enum_devices(skfd: c_int, fn_: iw_enum_handler, args: *mut *mut c_char, count: c_int);
    pub fn iw_get_basic_config(
        skfd: c_int,
        ifname: *const c_char,
        info: *mut wireless_config,
    ) -> c_int;
    pub fn iw_essid_escape(dest: *mut c_char, src: *const c_char, slen: c_int);
}

#[inline]
pub unsafe fn iw_sockets_close(skfd: c_int) {
    close(skfd);
}
