use std::ffi::c_void;
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
            unsafe { func(m, wrapper.data) }
        }
    })
}
