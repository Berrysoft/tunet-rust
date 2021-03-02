#![feature(option_result_unwrap_unchecked)]
#![feature(thread_local)]

use lazy_static::*;
use std::borrow::Cow;
use std::convert::From;
use std::ffi::{CStr, CString};
use std::net::Ipv4Addr;
use std::os::raw::{c_char, c_void};
use std::ptr::null;
use tunet_rust::{usereg::*, *};

#[repr(i32)]
pub enum State {
    Unknown,
    Net,
    Auth4,
    Auth6,
}

#[repr(i32)]
pub enum DetailOrder {
    LoginTime,
    LogoutTime,
    Flux,
}

#[repr(C)]
pub struct Credential {
    username: *const c_char,
    password: *const c_char,
    state: State,
    use_proxy: bool,
    ac_id_hints: AcIdHints,
}

#[repr(C)]
pub struct AcIdHints {
    data: *const i32,
    size: usize,
}

#[repr(C)]
pub struct Flux {
    username: *const c_char,
    flux: u64,
    online_time: u64,
    balance: f64,
}

#[repr(C)]
pub struct User {
    address: u32,
    login_time: i64,
    mac_address: [u8; 6],
}

#[repr(C)]
pub struct Detail {
    login_time: i64,
    logout_time: i64,
    flux: u64,
}

#[thread_local]
static mut ERROR_MSG: Option<CString> = None;

fn write_string<'a, S: Into<Cow<'a, str>>>(msg: S, storage: &mut Option<CString>) -> *const c_char {
    unsafe {
        *storage = Some(CString::from_vec_unchecked(
            msg.into().into_owned().into_bytes(),
        ));
        storage.as_ref().unwrap_unchecked().as_ptr()
    }
}

#[no_mangle]
pub extern "C" fn tunet_last_err() -> *const c_char {
    unsafe {
        match &ERROR_MSG {
            Some(str) => str.as_ptr(),
            None => null(),
        }
    }
}

unsafe fn exact_str<'a>(cstr: *const c_char) -> Cow<'a, str> {
    CStr::from_ptr(cstr).to_string_lossy()
}

lazy_static! {
    static ref CLIENT: Option<HttpClient> = create_http_client(true).ok();
    static ref NO_PROXY_CLIENT: Option<HttpClient> = create_http_client(false).ok();
}

fn get_client(proxy: bool) -> Result<&'static HttpClient> {
    match if proxy {
        CLIENT.as_ref()
    } else {
        NO_PROXY_CLIENT.as_ref()
    } {
        Some(c) => Ok(c),
        None => Err(NetHelperError::InitErr),
    }
}

fn get_helper(cred: &Credential) -> Result<TUNetConnect> {
    unsafe {
        let u = exact_str(cred.username);
        let p = exact_str(cred.password);
        let state = match &cred.state {
            State::Net => NetState::Net,
            State::Auth4 => NetState::Auth4,
            State::Auth6 => NetState::Auth6,
            _ => NetState::Unknown,
        };
        TUNetConnect::from_state_cred_client(
            state,
            u,
            p,
            get_client(cred.use_proxy)?,
            std::slice::from_raw_parts(cred.ac_id_hints.data, cred.ac_id_hints.size).to_vec(),
        )
    }
}

fn get_usereg_helper(cred: &Credential) -> Result<UseregHelper> {
    unsafe {
        let u = exact_str(cred.username);
        let p = exact_str(cred.password);
        Ok(UseregHelper::from_cred_client(
            u,
            p,
            get_client(cred.use_proxy)?,
        ))
    }
}

fn unwrap_res(res: Result<i32>) -> i32 {
    unsafe {
        match res {
            Ok(r) => {
                ERROR_MSG = None;
                r
            }
            Err(e) => {
                write_string(format!("{}", e), &mut ERROR_MSG);
                -1
            }
        }
    }
}

fn unwrap_ptr<'a, T>(ptr: *const T) -> Result<&'a T> {
    if let Some(r) = unsafe { ptr.as_ref() } {
        Ok(r)
    } else {
        Err(NetHelperError::NullPtrErr)
    }
}

#[thread_local]
static mut AC_ID_HINTS: Vec<i32> = Vec::new();

#[no_mangle]
pub extern "C" fn tunet_login(cred: *const Credential) -> i32 {
    unwrap_res(tunet_login_impl(cred))
}

fn tunet_login_impl(cred: *const Credential) -> Result<i32> {
    let cred = unwrap_ptr(cred)?;
    let mut helper = get_helper(cred)?;
    helper.login()?;
    unsafe {
        AC_ID_HINTS = helper.ac_ids().to_vec();
    }
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_hints() -> AcIdHints {
    unsafe {
        AcIdHints {
            data: AC_ID_HINTS.as_ptr(),
            size: AC_ID_HINTS.len(),
        }
    }
}

#[no_mangle]
pub extern "C" fn tunet_logout(cred: *const Credential) -> i32 {
    unwrap_res(tunet_logout_impl(cred))
}

fn tunet_logout_impl(cred: *const Credential) -> Result<i32> {
    let cred = unwrap_ptr(cred)?;
    let mut helper = get_helper(cred)?;
    helper.logout()?;
    Ok(0)
}

#[thread_local]
static mut FLUX_USERNAME: Option<CString> = None;

#[no_mangle]
pub extern "C" fn tunet_status(cred: *const Credential, flux: &mut Flux) -> i32 {
    unwrap_res(tunet_status_impl(cred, flux))
}

fn tunet_status_impl(cred: *const Credential, flux: &mut Flux) -> Result<i32> {
    let cred = unwrap_ptr(cred)?;
    let helper = get_helper(cred)?;
    let f = helper.flux()?;
    unsafe {
        flux.username = write_string(f.username, &mut FLUX_USERNAME);
    }
    flux.online_time = f.online_time.as_secs();
    flux.flux = f.flux;
    flux.balance = f.balance;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_usereg_login(cred: *const Credential) -> i32 {
    unwrap_res(tunet_usereg_login_impl(cred))
}

fn tunet_usereg_login_impl(cred: *const Credential) -> Result<i32> {
    let cred = unwrap_ptr(cred)?;
    let mut helper = get_usereg_helper(cred)?;
    helper.login()?;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_usereg_logout(cred: *const Credential) -> i32 {
    unwrap_res(tunet_usereg_logout_impl(cred))
}

fn tunet_usereg_logout_impl(cred: *const Credential) -> Result<i32> {
    let cred = unwrap_ptr(cred)?;
    let mut helper = get_usereg_helper(cred)?;
    helper.logout()?;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_usereg_drop(cred: *const Credential, addr: u32) -> i32 {
    unwrap_res(tunet_usereg_drop_impl(cred, addr))
}

fn tunet_usereg_drop_impl(cred: *const Credential, addr: u32) -> Result<i32> {
    let cred = unwrap_ptr(cred)?;
    let helper = get_usereg_helper(cred)?;
    let a = Ipv4Addr::from(addr);
    helper.drop(a)?;
    Ok(0)
}

pub type UseregUsersCallback = extern "C" fn(user: &User, data: *mut c_void) -> bool;

#[no_mangle]
pub extern "C" fn tunet_usereg_users(
    cred: *const Credential,
    callback: Option<UseregUsersCallback>,
    data: *mut c_void,
) -> i32 {
    unwrap_res(tunet_usereg_users_impl(cred, callback, data))
}

fn tunet_usereg_users_impl(
    cred: *const Credential,
    callback: Option<UseregUsersCallback>,
    data: *mut c_void,
) -> Result<i32> {
    let cred = unwrap_ptr(cred)?;
    let helper = get_usereg_helper(cred)?;
    let users = helper.users()?;
    let mut len = 0;
    if let Some(callback) = callback {
        for u in users {
            let user = User {
                address: u.address.into(),
                login_time: u.login_time.timestamp(),
                mac_address: u.mac_address.bytes(),
            };
            len += 1;
            if !callback(&user, data) {
                break;
            }
        }
    }
    Ok(len)
}

pub type UseregDetailsCallback = extern "C" fn(detail: &Detail, data: *mut c_void) -> bool;

#[no_mangle]
pub extern "C" fn tunet_usereg_details(
    cred: *const Credential,
    order: DetailOrder,
    desc: bool,
    callback: Option<UseregDetailsCallback>,
    data: *mut c_void,
) -> i32 {
    unwrap_res(tunet_usereg_details_impl(cred, order, desc, callback, data))
}

fn tunet_usereg_details_impl(
    cred: *const Credential,
    order: DetailOrder,
    desc: bool,
    callback: Option<UseregDetailsCallback>,
    data: *mut c_void,
) -> Result<i32> {
    let cred = unwrap_ptr(cred)?;
    let helper = get_usereg_helper(cred)?;
    let o = match order {
        DetailOrder::LoginTime => NetDetailOrder::LoginTime,
        DetailOrder::LogoutTime => NetDetailOrder::LogoutTime,
        DetailOrder::Flux => NetDetailOrder::Flux,
    };
    let mut details = helper.details(o, desc)?;
    let mut len = 0;
    if let Some(callback) = callback {
        for d in &mut details {
            let detail = Detail {
                login_time: d.login_time.timestamp(),
                logout_time: d.logout_time.timestamp(),
                flux: d.flux,
            };
            len += 1;
            if !callback(&detail, data) {
                break;
            }
        }
    }
    details.into_ret().unwrap_or(Ok(()))?;
    Ok(len)
}
