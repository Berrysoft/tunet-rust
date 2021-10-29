use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::runtime::Builder;
use tokio::sync::mpsc::*;
use tunet_model::*;
use tunet_rust::*;

mod native;

fn tunet_runtime_init_impl(
    val: usize,
    main: native::MainCallback,
    data: *mut c_void,
) -> Result<i32> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(val)
        .enable_all()
        .build()?;
    let res = runtime.block_on(async move {
        if let Some(main) = main {
            main(data)
        } else {
            1
        }
    });
    Ok(res)
}

#[no_mangle]
pub extern "C" fn tunet_runtime_init(
    val: usize,
    main: native::MainCallback,
    data: *mut c_void,
) -> i32 {
    tunet_runtime_init_impl(val, main, data).unwrap_or(1)
}

#[no_mangle]
pub extern "C" fn tunet_color_accent() -> color_theme::Color {
    color_theme::Color::accent()
}

fn tunet_model_new_impl(
    update: native::UpdateCallback,
    data: *mut c_void,
) -> Result<native::Model> {
    let (tx, mut rx) = channel(32);
    let model = Arc::new(Mutex::new({
        let mut model = Model::new(tx)?;
        model.set_callback(Some(native::wrap_callback(update, data)));
        model
    }));
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
    let model = Arc::from_raw(model);
    model.lock().unwrap().set_callback(None);
}

unsafe fn lock_model<'a>(model: native::Model) -> MutexGuard<'a, Model> {
    model.as_ref().unwrap().lock().unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue(model: native::Model, action: native::Action) {
    lock_model(model).queue(action.into());
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue_state(model: native::Model, state: native::State) {
    lock_model(model).queue(Action::State(Some(state.into())));
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_log(model: native::Model) -> native::StringView {
    native::StringView::new(&lock_model(model).log)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_username(model: native::Model) -> native::StringView {
    native::StringView::new(&lock_model(model).flux.username)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_flux(model: native::Model) -> u64 {
    lock_model(model).flux.flux.0
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_online_time(model: native::Model) -> i64 {
    lock_model(model).flux.online_time.0.num_seconds()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_balance(model: native::Model) -> f64 {
    lock_model(model).flux.balance.0
}
