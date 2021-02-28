use lazy_static::*;
use std::ffi::{CStr, CString};
use std::net::Ipv4Addr;
use std::os::raw::{c_char, c_void};
use std::ptr::null_mut;
use std::sync::Mutex;
use std::{borrow::Cow, convert::From};
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
}

#[repr(C)]
pub struct Flux {
    username: *mut c_char,
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

lazy_static! {
    static ref ERROR_MSG: Mutex<String> = Mutex::new(String::new());
}

fn write_string(msg: &str) -> *mut c_char {
    CString::new(msg)
        .map(|s| s.into_raw())
        .unwrap_or(null_mut())
}

#[no_mangle]
pub extern "C" fn tunet_last_err() -> *mut c_char {
    ERROR_MSG
        .lock()
        .map(|s| write_string(&s))
        .unwrap_or(null_mut())
}

#[no_mangle]
pub extern "C" fn tunet_string_free(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            let _s = CString::from_raw(ptr);
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
        from_state_cred_client(state, u, p, get_client(cred.use_proxy)?, vec![])
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
    match res {
        Ok(r) => r,
        Err(e) => match ERROR_MSG.lock().map(|mut s| *s = format!("{}", e)) {
            Ok(_) => -1,
            Err(_) => -2,
        },
    }
}

#[no_mangle]
pub extern "C" fn tunet_login(cred: &Credential) -> i32 {
    unwrap_res(tunet_login_impl(cred))
}

fn tunet_login_impl(cred: &Credential) -> Result<i32> {
    let mut helper = get_helper(cred)?;
    helper.login()?;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_logout(cred: &Credential) -> i32 {
    unwrap_res(tunet_logout_impl(cred))
}

fn tunet_logout_impl(cred: &Credential) -> Result<i32> {
    let mut helper = get_helper(cred)?;
    helper.logout()?;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_status(cred: &Credential, flux: &mut Flux) -> i32 {
    unwrap_res(tunet_status_impl(cred, flux))
}

fn tunet_status_impl(cred: &Credential, flux: &mut Flux) -> Result<i32> {
    let helper = get_helper(cred)?;
    let f = helper.flux()?;
    flux.username = write_string(&f.username);
    flux.online_time = f.online_time.as_secs();
    flux.flux = f.flux;
    flux.balance = f.balance;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_usereg_login(cred: &Credential) -> i32 {
    unwrap_res(tunet_usereg_login_impl(cred))
}

fn tunet_usereg_login_impl(cred: &Credential) -> Result<i32> {
    let mut helper = get_usereg_helper(cred)?;
    helper.login()?;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_usereg_logout(cred: &Credential) -> i32 {
    unwrap_res(tunet_usereg_logout_impl(cred))
}

fn tunet_usereg_logout_impl(cred: &Credential) -> Result<i32> {
    let mut helper = get_usereg_helper(cred)?;
    helper.logout()?;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_usereg_drop(cred: &Credential, addr: u32) -> i32 {
    unwrap_res(tunet_usereg_drop_impl(cred, addr))
}

fn tunet_usereg_drop_impl(cred: &Credential, addr: u32) -> Result<i32> {
    let helper = get_usereg_helper(cred)?;
    let a = Ipv4Addr::from(addr);
    helper.drop(a)?;
    Ok(0)
}

pub type UseregUsersCallback = extern "C" fn(user: &User, data: *mut c_void) -> bool;

#[no_mangle]
pub extern "C" fn tunet_usereg_users(
    cred: &Credential,
    callback: Option<UseregUsersCallback>,
    data: *mut c_void,
) -> i32 {
    unwrap_res(tunet_usereg_users_impl(cred, callback, data))
}

fn tunet_usereg_users_impl(
    cred: &Credential,
    callback: Option<UseregUsersCallback>,
    data: *mut c_void,
) -> Result<i32> {
    let helper = get_usereg_helper(cred)?;
    let users = helper.users()?;
    let mut len = 0;
    if let Some(callback) = callback {
        for u in users {
            let user = User {
                address: u.address.into(),
                login_time: u.login_time.timestamp(),
                mac_address: u.mac_address.octets(),
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
    cred: &Credential,
    order: DetailOrder,
    desc: bool,
    callback: Option<UseregDetailsCallback>,
    data: *mut c_void,
) -> i32 {
    unwrap_res(tunet_usereg_details_impl(cred, order, desc, callback, data))
}

fn tunet_usereg_details_impl(
    cred: &Credential,
    order: DetailOrder,
    desc: bool,
    callback: Option<UseregDetailsCallback>,
    data: *mut c_void,
) -> Result<i32> {
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
    if let Some(ret) = details.into_ret() {
        ret?;
    }
    Ok(len)
}
