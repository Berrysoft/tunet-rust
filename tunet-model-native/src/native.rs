use std::{
    ffi::c_void,
    sync::{Arc, RwLock},
};
use tunet_model::UpdateMsg;
use tunet_rust::{usereg::NetDetail, NetState};

#[repr(i32)]
pub enum Action {
    Timer,
    Tick,
    Login,
    Logout,
    Flux,
    Online,
    Details,
}

impl From<Action> for tunet_model::Action {
    fn from(a: Action) -> Self {
        match a {
            Action::Timer => Self::Timer,
            Action::Tick => Self::Tick,
            Action::Login => Self::Login,
            Action::Logout => Self::Logout,
            Action::Flux => Self::Flux,
            Action::Online => Self::Online,
            Action::Details => Self::Details,
        }
    }
}

#[repr(i32)]
pub enum State {
    Auto,
    Net,
    Auth4,
    Auth6,
}

impl From<State> for NetState {
    fn from(s: State) -> Self {
        match s {
            State::Auto => Self::Auto,
            State::Net => Self::Net,
            State::Auth4 => Self::Auth4,
            State::Auth6 => Self::Auth6,
        }
    }
}

impl From<NetState> for State {
    fn from(s: NetState) -> Self {
        match s {
            NetState::Net => Self::Net,
            NetState::Auth4 => Self::Auth4,
            NetState::Auth6 => Self::Auth6,
            _ => Self::Auto,
        }
    }
}

#[repr(C)]
pub struct Detail {
    pub login_time: i64,
    pub logout_time: i64,
    pub flux: u64,
}

impl From<&NetDetail> for Detail {
    fn from(d: &NetDetail) -> Self {
        Self {
            login_time: d.login_time.timestamp(),
            logout_time: d.logout_time.timestamp(),
            flux: d.flux.0,
        }
    }
}

#[repr(C)]
pub struct DetailGroup {
    pub logout_date: i64,
    pub flux: u64,
}

#[repr(C)]
pub struct DetailGroupByTime {
    pub logout_start_time: u32,
    pub flux: u64,
}

pub type MainCallback = Option<extern "C" fn(*mut c_void) -> i32>;
pub type UpdateCallback = Option<extern "C" fn(UpdateMsg, *mut c_void)>;
pub type DetailsForeachCallback = Option<extern "C" fn(*const Detail, *mut c_void) -> bool>;
pub type DetailsGroupedForeachCallback =
    Option<extern "C" fn(*const DetailGroup, *mut c_void) -> bool>;
pub type DetailsGroupedByTimeForeachCallback =
    Option<extern "C" fn(*const DetailGroupByTime, *mut c_void) -> bool>;

pub fn wrap_callback(
    func: UpdateCallback,
    data: *mut c_void,
) -> Arc<dyn Fn(UpdateMsg) + Send + Sync + 'static> {
    struct TempWrapper {
        func: UpdateCallback,
        data: *mut c_void,
    }

    unsafe impl Send for TempWrapper {}
    unsafe impl Sync for TempWrapper {}

    let wrapper = TempWrapper { func, data };

    Arc::new(move |m| {
        if let Some(func) = wrapper.func {
            func(m, wrapper.data)
        }
    })
}

pub type Model = *const RwLock<tunet_model::Model>;

#[repr(C)]
pub struct StringView {
    data: *const u8,
    size: usize,
}

impl StringView {
    pub fn new(s: &str) -> Self {
        let span = s.as_bytes();
        Self {
            data: span.as_ptr(),
            size: span.len(),
        }
    }
}
