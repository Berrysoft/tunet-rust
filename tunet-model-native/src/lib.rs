#![allow(clippy::missing_safety_doc)]

use itertools::Itertools;
use mac_address::MacAddress;
use std::ffi::c_void;
use std::net::Ipv4Addr;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::runtime::Builder;
use tokio::sync::mpsc::*;
use tunet_helper::*;
use tunet_model::*;
use tunet_settings::*;
use widestring::{U16CStr, U16CString};

mod native;

unsafe fn write_str(p: *const u16) -> String {
    if !p.is_null() {
        U16CStr::from_ptr_str(p).to_string_lossy()
    } else {
        String::new()
    }
}

unsafe fn read_str(s: &str, f: extern "C" fn(*const u16, *mut c_void), data: *mut c_void) {
    let u16str = U16CString::from_str_unchecked(s);
    f(u16str.as_ptr(), data)
}

unsafe fn tunet_format<T: ToString>(value: T, f: native::StringCallback, data: *mut c_void) {
    if let Some(f) = f {
        read_str(&value.to_string(), f, data)
    }
}

#[no_mangle]
pub unsafe extern "C" fn tunet_format_flux(
    flux: u64,
    f: native::StringCallback,
    data: *mut c_void,
) {
    tunet_format(Flux(flux), f, data)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_format_duration(
    sec: i64,
    f: native::StringCallback,
    data: *mut c_void,
) {
    tunet_format(Duration(NaiveDuration::seconds(sec)), f, data)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_format_ip(addr: u32, f: native::StringCallback, data: *mut c_void) {
    tunet_format(Ipv4Addr::from(addr), f, data)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_parse_ip(str: *const u16) -> u32 {
    write_str(str).parse::<Ipv4Addr>().unwrap().into()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_format_mac_address(
    addr: &[u8; 6],
    f: native::StringCallback,
    data: *mut c_void,
) {
    tunet_format(MacAddress::from(*addr), f, data)
}

unsafe fn write_model<'a>(model: native::Model) -> RwLockWriteGuard<'a, Model> {
    model.as_ref().unwrap().write().unwrap()
}

unsafe fn read_model<'a>(model: native::Model) -> RwLockReadGuard<'a, Model> {
    model.as_ref().unwrap().read().unwrap()
}

fn tunet_model_start_impl(
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
            let mut reader = FileSettingsReader::new()?;
            if model.read().unwrap().del_at_exit() {
                reader.delete()?;
            } else {
                reader.save(model.read().unwrap().cred.clone()).await?;
            }
            Ok::<_, anyhow::Error>(res)
        } else {
            Ok(0)
        }
    })
}

#[no_mangle]
pub extern "C" fn tunet_model_start(
    val: usize,
    main: native::MainCallback,
    data: *mut c_void,
) -> i32 {
    tunet_model_start_impl(val, main, data).unwrap_or(1)
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_set_update_callback(
    model: native::Model,
    update: native::UpdateCallback,
    data: *mut c_void,
) {
    write_model(model).update = native::wrap_callback(update, data);
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue(model: native::Model, action: native::Action) {
    read_model(model).queue(action.into());
}

unsafe fn tunet_model_queue_cred_load_impl(model: native::Model) -> Result<()> {
    let reader = FileSettingsReader::new()?;
    let cred = reader.read_with_password()?;
    read_model(model).queue(Action::Credential(Arc::new(cred)));
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue_cred_load(model: native::Model) -> bool {
    tunet_model_queue_cred_load_impl(model).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue_cred(
    model: native::Model,
    u: *const u16,
    p: *const u16,
) {
    read_model(model).queue(Action::UpdateCredential(write_str(u), write_str(p)));
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue_state(model: native::Model, state: native::State) {
    read_model(model).queue(Action::State(Some(state.into())));
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue_connect(model: native::Model, addr: u32) {
    read_model(model).queue(Action::Connect(Ipv4Addr::from(addr)));
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_queue_drop(model: native::Model, addr: u32) {
    read_model(model).queue(Action::Drop(Ipv4Addr::from(addr)));
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_status(
    model: native::Model,
    f: native::StringCallback,
    data: *mut c_void,
) {
    if let Some(f) = f {
        read_str(&read_model(model).status.to_string(), f, data)
    }
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_accent_color(model: native::Model) -> color_theme::Color {
    read_model(model).accent
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

#[no_mangle]
pub unsafe extern "C" fn tunet_model_log_busy(model: native::Model) -> bool {
    read_model(model).log_busy()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_online_busy(model: native::Model) -> bool {
    read_model(model).online_busy()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_detail_busy(model: native::Model) -> bool {
    read_model(model).detail_busy()
}

#[no_mangle]
pub unsafe extern "C" fn tunet_model_set_del_at_exit(model: native::Model, v: bool) {
    read_model(model).set_del_at_exit(v)
}
