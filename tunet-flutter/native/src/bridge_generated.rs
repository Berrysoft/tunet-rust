#![allow(
    non_camel_case_types,
    unused,
    clippy::redundant_closure,
    clippy::useless_conversion,
    clippy::unit_arg,
    clippy::double_parens,
    non_snake_case,
    clippy::too_many_arguments
)]
// AUTO GENERATED FILE, DO NOT EDIT.
// Generated by `flutter_rust_bridge`@ 1.77.1.

use crate::api::*;
use core::panic::UnwindSafe;
use flutter_rust_bridge::*;
use std::ffi::c_void;
use std::sync::Arc;

// Section: imports

// Section: wire functions

fn wire_flux_to_string_impl(port_: MessagePort, f: impl Wire2Api<u64> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "flux_to_string",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_f = f.wire2api();
            move |task_callback| Ok(flux_to_string(api_f))
        },
    )
}
fn wire_new__static_method__Runtime_impl(port_: MessagePort) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "new__static_method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || move |task_callback| Runtime::new(),
    )
}
fn wire_start__method__Runtime_impl(port_: MessagePort, that: impl Wire2Api<Runtime> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "start__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Stream,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Ok(Runtime::start(&api_that, task_callback.stream_sink()))
        },
    )
}
fn wire_queue_login__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "queue_login__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Ok(Runtime::queue_login(&api_that))
        },
    )
}
fn wire_queue_logout__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "queue_logout__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Ok(Runtime::queue_logout(&api_that))
        },
    )
}
fn wire_queue_flux__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "queue_flux__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Ok(Runtime::queue_flux(&api_that))
        },
    )
}
fn wire_flux__method__Runtime_impl(port_: MessagePort, that: impl Wire2Api<Runtime> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "flux__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Ok(mirror_NetFlux(Runtime::flux(&api_that)))
        },
    )
}
fn wire_state__method__Runtime_impl(port_: MessagePort, that: impl Wire2Api<Runtime> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "state__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Ok(Runtime::state(&api_that))
        },
    )
}
// Section: wrapper structs

#[derive(Clone)]
struct mirror_Balance(Balance);

#[derive(Clone)]
struct mirror_Flux(Flux);

#[derive(Clone)]
struct mirror_NetFlux(NetFlux);

#[derive(Clone)]
struct mirror_NetState(NetState);

#[derive(Clone)]
struct mirror_NewDuration(NewDuration);

#[derive(Clone)]
struct mirror_UpdateMsg(UpdateMsg);

// Section: static checks

const _: fn() = || {
    {
        let Balance_ = None::<Balance>.unwrap();
        let _: f64 = Balance_.0;
    }
    {
        let Flux_ = None::<Flux>.unwrap();
        let _: u64 = Flux_.0;
    }
    {
        let NetFlux = None::<NetFlux>.unwrap();
        let _: String = NetFlux.username;
        let _: Flux = NetFlux.flux;
        let _: NewDuration = NetFlux.online_time;
        let _: Balance = NetFlux.balance;
    }
    match None::<NetState>.unwrap() {
        NetState::Unknown => {}
        NetState::Net => {}
        NetState::Auth4 => {}
        NetState::Auth6 => {}
    }
    {
        let NewDuration_ = None::<NewDuration>.unwrap();
        let _: chrono::Duration = NewDuration_.0;
    }
    match None::<UpdateMsg>.unwrap() {
        UpdateMsg::Credential => {}
        UpdateMsg::State => {}
        UpdateMsg::Status => {}
        UpdateMsg::Log => {}
        UpdateMsg::Flux => {}
        UpdateMsg::Online => {}
        UpdateMsg::Details => {}
        UpdateMsg::LogBusy => {}
        UpdateMsg::OnlineBusy => {}
        UpdateMsg::DetailBusy => {}
    }
};
// Section: allocate functions

// Section: related functions

// Section: impl Wire2Api

pub trait Wire2Api<T> {
    fn wire2api(self) -> T;
}

impl<T, S> Wire2Api<Option<T>> for *mut S
where
    *mut S: Wire2Api<T>,
{
    fn wire2api(self) -> Option<T> {
        (!self.is_null()).then(|| self.wire2api())
    }
}

impl Wire2Api<u64> for u64 {
    fn wire2api(self) -> u64 {
        self
    }
}
// Section: impl IntoDart

impl support::IntoDart for mirror_Balance {
    fn into_dart(self) -> support::DartAbi {
        vec![self.0 .0.into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_Balance {}

impl support::IntoDart for mirror_Flux {
    fn into_dart(self) -> support::DartAbi {
        vec![self.0 .0.into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_Flux {}

impl support::IntoDart for mirror_NetFlux {
    fn into_dart(self) -> support::DartAbi {
        vec![
            self.0.username.into_dart(),
            mirror_Flux(self.0.flux).into_dart(),
            mirror_NewDuration(self.0.online_time).into_dart(),
            mirror_Balance(self.0.balance).into_dart(),
        ]
        .into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_NetFlux {}

impl support::IntoDart for mirror_NetState {
    fn into_dart(self) -> support::DartAbi {
        match self.0 {
            NetState::Unknown => 0,
            NetState::Net => 1,
            NetState::Auth4 => 2,
            NetState::Auth6 => 3,
        }
        .into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_NetState {}
impl support::IntoDart for NetStateWrap {
    fn into_dart(self) -> support::DartAbi {
        vec![mirror_NetState(self.0).into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for NetStateWrap {}

impl support::IntoDart for mirror_NewDuration {
    fn into_dart(self) -> support::DartAbi {
        vec![self.0 .0.into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_NewDuration {}

impl support::IntoDart for Runtime {
    fn into_dart(self) -> support::DartAbi {
        vec![
            self.rx.into_dart(),
            self.model.into_dart(),
            self.handle.into_dart(),
        ]
        .into_dart()
    }
}
impl support::IntoDartExceptPrimitive for Runtime {}

impl support::IntoDart for mirror_UpdateMsg {
    fn into_dart(self) -> support::DartAbi {
        match self.0 {
            UpdateMsg::Credential => 0,
            UpdateMsg::State => 1,
            UpdateMsg::Status => 2,
            UpdateMsg::Log => 3,
            UpdateMsg::Flux => 4,
            UpdateMsg::Online => 5,
            UpdateMsg::Details => 6,
            UpdateMsg::LogBusy => 7,
            UpdateMsg::OnlineBusy => 8,
            UpdateMsg::DetailBusy => 9,
        }
        .into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_UpdateMsg {}
impl support::IntoDart for UpdateMsgWrap {
    fn into_dart(self) -> support::DartAbi {
        vec![mirror_UpdateMsg(self.0).into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for UpdateMsgWrap {}

// Section: executor

support::lazy_static! {
    pub static ref FLUTTER_RUST_BRIDGE_HANDLER: support::DefaultHandler = Default::default();
}

#[cfg(not(target_family = "wasm"))]
#[path = "bridge_generated.io.rs"]
mod io;
#[cfg(not(target_family = "wasm"))]
pub use io::*;
