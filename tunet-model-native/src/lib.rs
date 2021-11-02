#![allow(clippy::missing_safety_doc)]

use itertools::Itertools;
use netstatus::*;
use std::ffi::c_void;
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
    runtime.block_on(async move {
        if let Some(main) = main {
            let (tx, mut rx) = channel(32);
            let model = Arc::new(RwLock::new(Model::new(tx)?));
            {
                let model = model.clone();
                tokio::spawn(async move {
                    while let Some(a) = rx.recv().await {
                        let mut model = model.write().unwrap();
                        model.handle(a);
                    }
                });
            }
            let res = main(Arc::as_ptr(&model), data);
            Ok::<_, anyhow::Error>(res)
        } else {
            Ok(0)
        }
    })
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

#[no_mangle]
pub unsafe extern "C" fn tunet_model_set_update_callback(
    model: native::Model,
    update: native::UpdateCallback,
    data: *mut c_void,
) {
    let mut model = model.as_ref().unwrap().write().unwrap();
    model.set_callback(native::wrap_callback(update, data));
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

fn read_str(s: &str, f: extern "C" fn(*const u8, usize, *mut c_void), data: *mut c_void) {
    let bytes = s.as_bytes();
    f(bytes.as_ptr(), bytes.len(), data)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_status(
    model: native::Model,
    f: native::StringCallback,
    data: *mut c_void,
) -> native::Status {
    match &read_model(model).status {
        NetStatus::Unknown => native::Status::Unknown,
        NetStatus::Wwan => native::Status::Wwan,
        NetStatus::Wlan(ssid) => {
            if let Some(f) = f {
                read_str(ssid, f, data);
            }
            native::Status::Wlan
        }
        NetStatus::Lan => native::Status::Lan,
    }
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_cred_username(
    model: native::Model,
    f: native::StringCallback,
    data: *mut c_void,
) {
    if let Some(f) = f {
        read_str(&read_model(model).cred.username, f, data)
    }
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_cred_password(
    model: native::Model,
    f: native::StringCallback,
    data: *mut c_void,
) {
    if let Some(f) = f {
        read_str(&read_model(model).cred.password, f, data)
    }
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_state(model: native::Model) -> native::State {
    read_model(model).state.into()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_log(
    model: native::Model,
    f: native::StringCallback,
    data: *mut c_void,
) {
    if let Some(f) = f {
        read_str(&read_model(model).log, f, data)
    }
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_flux_username(
    model: native::Model,
    f: native::StringCallback,
    data: *mut c_void,
) {
    if let Some(f) = f {
        read_str(&read_model(model).flux.username, f, data)
    }
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
pub unsafe extern "C" fn tunet_model_onlines_foreach(
    model: native::Model,
    f: native::OnlinesForeachCallback,
    data: *mut c_void,
) {
    if let Some(f) = f {
        let model = read_model(model);
        for u in &model.users {
            let ou = native::OnlineUser::new(u, &model.mac_addrs);
            if !f(&ou, data) {
                break;
            }
        }
    }
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
