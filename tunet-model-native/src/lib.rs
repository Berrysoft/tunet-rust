use once_cell::sync::OnceCell;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::*;
use tunet_model::*;
use tunet_rust::*;

mod native;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn tunet_runtime_init_impl(val: usize) -> Result<()> {
    RUNTIME
        .set(
            Builder::new_multi_thread()
                .worker_threads(val)
                .enable_all()
                .build()?,
        )
        .map_err(|_| anyhow::anyhow!("Failed to set RUNTIME"))?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn tunet_runtime_init(val: usize) -> bool {
    tunet_runtime_init_impl(val).is_ok()
}

fn tunet_model_new_impl(
    update: native::UpdateCallback,
    data: *mut c_void,
) -> Result<native::Model> {
    let (tx, mut rx) = channel(32);
    let model = Arc::new(Mutex::new(Model::with_callback(
        native::wrap_callback(update, data),
        tx,
    )?));
    {
        let model = model.clone();
        tokio::spawn(async move {
            while let Some(a) = rx.recv().await {
                let mut model = model.lock().unwrap();
                model.handle(a);
            }
        });
    }
    Ok(Arc::into_raw(model))
}

#[no_mangle]
pub extern "C" fn tunet_model_new(
    update: native::UpdateCallback,
    data: *mut c_void,
) -> native::Model {
    tunet_model_new_impl(update, data).unwrap_or(null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_unref(model: native::Model) {
    let _ = Arc::from_raw(model);
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue(model: native::Model, action: native::Action) {
    let model = model.as_ref().unwrap().lock().unwrap();
    model.queue(action.into());
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_flux(model: native::Model) -> u64 {
    let model = model.as_ref().unwrap().lock().unwrap();
    model.flux.flux.0
}
