use lazy_static::*;
use reqwest::Client;
use std::convert::From;
use std::ffi::CStr;
use std::mem::size_of;
use std::net::Ipv4Addr;
use std::os::raw::c_char;
use std::ptr::{copy_nonoverlapping, null_mut};
use std::string::String;
use std::vec::Vec;
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
    username_len: i32,
    flux: i64,
    online_time: i64,
    balance: f64,
}

#[repr(C)]
pub struct User {
    address: i64,
    login_time: i64,
    client: *mut c_char,
    client_len: i32,
}

#[repr(C)]
pub struct Detail {
    login_time: i64,
    logout_time: i64,
    flux: i64,
}

static mut ERROR_MSG: String = String::new();

fn write_string(msg: &str, ptr: *mut c_char, len: i32) -> i32 {
    if ptr != null_mut() {
        unsafe {
            copy_nonoverlapping(msg.as_ptr(), ptr as *mut u8, len as usize * size_of::<u8>());
        }
        msg.len() as i32
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn tunet_last_err(message: *mut c_char, len: i32) -> i32 {
    unsafe { write_string(&ERROR_MSG, message, len) }
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
    flux.online_time = f.online_time.as_secs() as i64;
    flux.flux = f.flux as i64;
    flux.balance = f.balance;
    Ok(write_string(&f.username, flux.username, flux.username_len))
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

static mut USEREG_USERS: Vec<NetUser> = Vec::new();

#[no_mangle]
pub extern "C" fn tunet_usereg_users(cred: &Credential) -> i32 {
    unwrap_res(tunet_usereg_users_impl(cred))
}

fn tunet_usereg_users_impl(cred: &Credential) -> Result<i32> {
    let helper = get_usereg_helper(cred)?;
    unsafe {
        USEREG_USERS = helper.users()?;
        Ok(USEREG_USERS.len() as i32)
    }
}

#[no_mangle]
pub extern "C" fn tunet_usereg_users_destory() -> i32 {
    unsafe {
        USEREG_USERS.clear();
    }
    0
}

#[no_mangle]
pub extern "C" fn tunet_usereg_users_fetch(index: i32, user: &mut User) -> i32 {
    unwrap_res(tunet_usereg_users_fetch_impl(index, user))
}

fn tunet_usereg_users_fetch_impl(index: i32, user: &mut User) -> Result<i32> {
    let index = index as usize;
    unsafe {
        if index < USEREG_USERS.len() {
            let u = &USEREG_USERS[index];
            user.address = u32::from(u.address) as i64;
            user.login_time = u.login_time.timestamp();
            Ok(write_string(&u.client, user.client, user.client_len))
        } else {
            Ok(0)
        }
    }
}

static mut USEREG_DETAILS: Vec<NetDetail> = Vec::new();

#[no_mangle]
pub extern "C" fn tunet_usereg_details(cred: &Credential, order: DetailOrder, desc: i32) -> i32 {
    unwrap_res(tunet_usereg_details_impl(cred, order, desc))
}

fn tunet_usereg_details_impl(cred: &Credential, order: DetailOrder, desc: i32) -> Result<i32> {
    let helper = get_usereg_helper(cred)?;
    unsafe {
        let o = match order {
            DetailOrder::LoginTime => NetDetailOrder::LoginTime,
            DetailOrder::LogoutTime => NetDetailOrder::LogoutTime,
            DetailOrder::Flux => NetDetailOrder::Flux,
        };
        USEREG_DETAILS = helper.details(o, desc != 0)?;
        Ok(USEREG_DETAILS.len() as i32)
    }
}

#[no_mangle]
pub extern "C" fn tunet_usereg_details_destory() -> i32 {
    unsafe {
        USEREG_DETAILS.clear();
    }
    0
}

#[no_mangle]
pub extern "C" fn tunet_usereg_details_fetch(index: i32, detail: &mut Detail) -> i32 {
    unwrap_res(tunet_usereg_details_fetch_impl(index, detail))
}

fn tunet_usereg_details_fetch_impl(index: i32, detail: &mut Detail) -> Result<i32> {
    let index = index as usize;
    unsafe {
        if index < USEREG_DETAILS.len() {
            let d = &USEREG_DETAILS[index];
            detail.login_time = d.login_time.timestamp();
            detail.logout_time = d.logout_time.timestamp();
            detail.flux = d.flux as i64;
        }
    }
    Ok(0)
}
