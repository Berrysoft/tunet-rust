use itertools::Itertools;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use tokio::runtime::Builder;
use tokio::sync::mpsc::*;
use tunet_model::*;
use tunet_rust::*;
use tunet_settings::*;

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
    let model = Arc::new(RwLock::new({
        let mut model = Model::new(tx)?;
        model.set_callback(Some(native::wrap_callback(update, data)));
        model
    }));
    {
        let model = model.clone();
        tokio::spawn(async move {
            while let Some(a) = rx.recv().await {
                let mut model = model.write().unwrap();
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
    model.write().unwrap().set_callback(None);
}

unsafe fn read_model<'a>(model: native::Model) -> RwLockReadGuard<'a, Model> {
    model.as_ref().unwrap().read().unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue(model: native::Model, action: native::Action) {
    read_model(model).queue(action.into());
}

unsafe fn tunet_model_queue_read_cred_impl(model: native::Model) -> Result<()> {
    let reader = FileSettingsReader::new()?;
    let cred = reader.read_with_password()?;
    read_model(model).queue(Action::Credential(Arc::new(cred)));
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue_read_cred(model: native::Model) -> bool {
    tunet_model_queue_read_cred_impl(model).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue_state(model: native::Model, state: native::State) {
    read_model(model).queue(Action::State(Some(state.into())));
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_cred_username(model: native::Model) -> native::StringView {
    native::StringView::new(&read_model(model).cred.username)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_cred_password(model: native::Model) -> native::StringView {
    native::StringView::new(&read_model(model).cred.password)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_state(model: native::Model) -> native::State {
    read_model(model).state.into()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_log(model: native::Model) -> native::StringView {
    native::StringView::new(&read_model(model).log)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_username(model: native::Model) -> native::StringView {
    native::StringView::new(&read_model(model).flux.username)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_flux(model: native::Model) -> u64 {
    read_model(model).flux.flux.0
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_online_time(model: native::Model) -> i64 {
    read_model(model).flux.online_time.0.num_seconds()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_balance(model: native::Model) -> f64 {
    read_model(model).flux.balance.0
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_details_foreach(
    model: native::Model,
    f: native::DetailsForeachCallback,
    data: *mut c_void,
) {
    if let Some(f) = f {
        for d in &read_model(model).details {
            let nd = d.into();
            if !f(&nd, data) {
                break;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_details_grouped_foreach(
    model: native::Model,
    f: native::DetailsGroupedForeachCallback,
    data: *mut c_void,
) {
    if let Some(f) = f {
        for (date, flux) in read_model(model)
            .details
            .iter()
            .group_by(|detail| detail.logout_time.date())
            .into_iter()
            .map(|(key, group)| (key, group.map(|detail| detail.flux.0).sum::<u64>()))
            .into_iter()
        {
            let g = native::DetailGroup {
                logout_date: date.and_hms(0, 0, 0).timestamp(),
                flux,
            };
            if !f(&g, data) {
                break;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_details_grouped_by_time_foreach(
    model: native::Model,
    groups: u32,
    f: native::DetailsGroupedByTimeForeachCallback,
    data: *mut c_void,
) {
    if let Some(f) = f {
        let interval = 24 / groups;
        for (t, flux) in read_model(model)
            .details
            .iter()
            .into_group_map_by(|detail| detail.logout_time.hour() / interval)
            .into_iter()
            .map(|(key, group)| {
                (
                    key * interval,
                    group.into_iter().map(|detail| detail.flux.0).sum::<u64>(),
                )
            })
            .into_iter()
        {
            let g = native::DetailGroupByTime {
                logout_start_time: t,
                flux,
            };
            if !f(&g, data) {
                break;
            }
        }
    }
}
