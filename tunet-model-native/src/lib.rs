use std::ffi::c_void;
use std::ptr::null_mut;
use tunet_model::*;

mod native;

fn tunet_model_new_impl(update: native::UpdateCallback, data: *mut c_void) -> Result<Box<Model>> {
    Ok(Box::new(Model::with_callback(
        native::wrap_callback(update, data),
        tx,
    )?))
}

#[no_mangle]
pub extern "C" fn tunet_model_new(update: native::UpdateCallback, data: *mut c_void) -> *mut Model {
    match tunet_model_new_impl(update, data) {
        Ok(m) => Box::leak(m),
        Err(e) => null_mut(),
    }
}
