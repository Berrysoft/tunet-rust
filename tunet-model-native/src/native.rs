use std::{
    ffi::c_void,
    sync::{Arc, Mutex},
};
use tunet_model::UpdateMsg;
use tunet_rust::NetState;

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

pub type MainCallback = Option<extern "C" fn(*mut c_void) -> i32>;
pub type UpdateCallback = Option<extern "C" fn(UpdateMsg, *mut c_void)>;

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

pub type Model = *const Mutex<tunet_model::Model>;

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
