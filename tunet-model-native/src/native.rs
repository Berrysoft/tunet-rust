use std::{ffi::c_void, sync::Mutex};
use tunet_model::UpdateMsg;

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

pub type UpdateCallback = Option<extern "C" fn(UpdateMsg, *mut c_void)>;

pub fn wrap_callback(
    func: UpdateCallback,
    data: *mut c_void,
) -> Box<dyn Fn(UpdateMsg) + Send + Sync + 'static> {
    struct TempWrapper {
        func: UpdateCallback,
        data: *mut c_void,
    }

    unsafe impl Send for TempWrapper {}
    unsafe impl Sync for TempWrapper {}

    let wrapper = TempWrapper { func, data };

    Box::new(move |m| {
        if let Some(func) = wrapper.func {
            func(m, wrapper.data)
        }
    })
}

pub type Model = *const Mutex<tunet_model::Model>;
