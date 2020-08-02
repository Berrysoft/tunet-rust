use lazy_static::*;
use reqwest::blocking::Client;
use std::convert::From;
use std::ffi::CStr;
use std::mem::size_of;
use std::net::Ipv4Addr;
use std::os::raw::{c_char, c_void};
use std::ptr::copy_nonoverlapping;
use std::string::String;
use tunet_rust::usereg::*;
use tunet_rust::*;

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
    use_proxy: i32,
}

#[repr(C)]
pub struct Flux {
    username: *mut c_char,
    flux: i64,
    online_time: i64,
    balance: f64,
}

#[repr(C)]
pub struct User {
    address: i64,
    login_time: i64,
    client: *mut c_char,
}

#[repr(C)]
pub struct Detail {
    login_time: i64,
    logout_time: i64,
    flux: i64,
}

static mut ERROR_MSG: String = String::new();

fn write_string(msg: &str) -> *mut c_char {
    unsafe {
        let ptr = libc::malloc(msg.len() + 1) as *mut c_char;
        if !ptr.is_null() {
            copy_nonoverlapping(msg.as_ptr(), ptr as *mut u8, msg.len() * size_of::<u8>());
            *(ptr.add(msg.len())) = 0;
        }
        ptr
    }
}

#[no_mangle]
pub extern "C" fn tunet_last_err() -> *mut c_char {
    unsafe { write_string(&ERROR_MSG) }
}

#[no_mangle]
pub extern "C" fn tunet_string_free(ptr: *const c_char) {
    unsafe { libc::free(ptr as *mut c_void) }
}

unsafe fn exact_str<'a>(cstr: *const c_char) -> &'a str {
    CStr::from_ptr(cstr).to_str().expect("")
}

lazy_static! {
    static ref CLIENT: Client = Client::builder().cookie_store(true).build().unwrap();
    static ref NO_PROXY_CLIENT: Client = Client::builder().cookie_store(true).no_proxy().build().unwrap();
}

fn get_client(proxy: bool) -> &'static Client {
    if proxy {
        &*CLIENT
    } else {
        &*NO_PROXY_CLIENT
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
        from_state_cred_client(state, u.to_owned(), p.to_owned(), get_client(cred.use_proxy != 0))
    }
}

fn get_usereg_helper(cred: &Credential) -> Result<UseregHelper> {
    unsafe {
        let u = exact_str(cred.username);
        let p = exact_str(cred.password);
        Ok(UseregHelper::from_cred_client(u.to_owned(), p.to_owned(), get_client(cred.use_proxy != 0)))
    }
}

fn unwrap_res(res: Result<i32>) -> i32 {
    match res {
        Ok(r) => r,
        Err(e) => {
            unsafe {
                ERROR_MSG = format!("{:?}", e);
            }
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn tunet_login(cred: &Credential) -> i32 {
    unwrap_res(tunet_login_impl(cred))
}

fn tunet_login_impl(cred: &Credential) -> Result<i32> {
    let helper = get_helper(cred)?;
    helper.login()?;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_logout(cred: &Credential) -> i32 {
    unwrap_res(tunet_logout_impl(cred))
}

fn tunet_logout_impl(cred: &Credential) -> Result<i32> {
    let helper = get_helper(cred)?;
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
    flux.online_time = f.online_time.as_secs() as i64;
    flux.flux = f.flux as i64;
    flux.balance = f.balance;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_usereg_login(cred: &Credential) -> i32 {
    unwrap_res(tunet_usereg_login_impl(cred))
}

fn tunet_usereg_login_impl(cred: &Credential) -> Result<i32> {
    let helper = get_usereg_helper(cred)?;
    helper.login()?;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_usereg_logout(cred: &Credential) -> i32 {
    unwrap_res(tunet_usereg_logout_impl(cred))
}

fn tunet_usereg_logout_impl(cred: &Credential) -> Result<i32> {
    let helper = get_usereg_helper(cred)?;
    helper.logout()?;
    Ok(0)
}

#[no_mangle]
pub extern "C" fn tunet_usereg_drop(cred: &Credential, addr: i64) -> i32 {
    unwrap_res(tunet_usereg_drop_impl(cred, addr))
}

fn tunet_usereg_drop_impl(cred: &Credential, addr: i64) -> Result<i32> {
    let helper = get_usereg_helper(cred)?;
    let a = Ipv4Addr::from(addr as u32);
    helper.drop(a)?;
    Ok(0)
}

pub type UseregUsersCallback = extern "C" fn(user: &User, data: *mut c_void) -> i32;

#[no_mangle]
pub extern "C" fn tunet_usereg_users(cred: &Credential, user: &mut User, callback: Option<UseregUsersCallback>, data: *mut c_void) -> i32 {
    unwrap_res(tunet_usereg_users_impl(cred, user, callback, data))
}

fn tunet_usereg_users_impl(cred: &Credential, user: &mut User, callback: Option<UseregUsersCallback>, data: *mut c_void) -> Result<i32> {
    let helper = get_usereg_helper(cred)?;
    let users = helper.users()?;
    if let Some(callback) = callback {
        for u in &users {
            user.address = u32::from(u.address) as i64;
            user.login_time = u.login_time.timestamp();
            user.client = write_string(&u.client);
            if callback(user, data) == 0 {
                break;
            }
        }
    }
    Ok(users.len() as i32)
}

pub type UseregDetailsCallback = extern "C" fn(detail: &Detail, data: *mut c_void) -> i32;

#[no_mangle]
pub extern "C" fn tunet_usereg_details(cred: &Credential, order: DetailOrder, desc: i32, callback: Option<UseregDetailsCallback>, data: *mut c_void) -> i32 {
    unwrap_res(tunet_usereg_details_impl(cred, order, desc, callback, data))
}

fn tunet_usereg_details_impl(cred: &Credential, order: DetailOrder, desc: i32, callback: Option<UseregDetailsCallback>, data: *mut c_void) -> Result<i32> {
    let helper = get_usereg_helper(cred)?;
    let o = match order {
        DetailOrder::LoginTime => NetDetailOrder::LoginTime,
        DetailOrder::LogoutTime => NetDetailOrder::LogoutTime,
        DetailOrder::Flux => NetDetailOrder::Flux,
    };
    let details = helper.details(o, desc != 0)?;
    if let Some(callback) = callback {
        for d in &details {
            let detail = Detail { login_time: d.login_time.timestamp(), logout_time: d.logout_time.timestamp(), flux: d.flux as i64 };
            if callback(&detail, data) == 0 {
                break;
            }
        }
    }
    Ok(details.len() as i32)
}
