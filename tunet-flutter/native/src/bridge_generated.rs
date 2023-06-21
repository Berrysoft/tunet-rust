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
// Generated by `flutter_rust_bridge`@ 1.78.0.

use crate::api::*;
use core::panic::UnwindSafe;
use flutter_rust_bridge::*;
use std::ffi::c_void;
use std::sync::Arc;

// Section: imports

// Section: wire functions

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
fn wire_start__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
    config: impl Wire2Api<RuntimeStartConfig> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "start__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Stream,
        },
        move || {
            let api_that = that.wire2api();
            let api_config = config.wire2api();
            move |task_callback| {
                Ok(Runtime::start(
                    &api_that,
                    task_callback.stream_sink(),
                    api_config,
                ))
            }
        },
    )
}
fn wire_current_status__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "current_status__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Ok(mirror_NetStatus(Runtime::current_status(&api_that)))
        },
    )
}
fn wire_load_credential__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "load_credential__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Runtime::load_credential(&api_that)
        },
    )
}
fn wire_save_credential__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
    u: impl Wire2Api<String> + UnwindSafe,
    p: impl Wire2Api<String> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "save_credential__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            let api_u = u.wire2api();
            let api_p = p.wire2api();
            move |task_callback| Runtime::save_credential(&api_that, api_u, api_p)
        },
    )
}
fn wire_queue_credential__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
    u: impl Wire2Api<String> + UnwindSafe,
    p: impl Wire2Api<String> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "queue_credential__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            let api_u = u.wire2api();
            let api_p = p.wire2api();
            move |task_callback| Ok(Runtime::queue_credential(&api_that, api_u, api_p))
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
fn wire_queue_state__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
    s: impl Wire2Api<Option<NetState>> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "queue_state__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            let api_s = s.wire2api();
            move |task_callback| Ok(Runtime::queue_state(&api_that, api_s))
        },
    )
}
fn wire_queue_details__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "queue_details__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Ok(Runtime::queue_details(&api_that))
        },
    )
}
fn wire_queue_onlines__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "queue_onlines__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            move |task_callback| Ok(Runtime::queue_onlines(&api_that))
        },
    )
}
fn wire_queue_connect__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
    ip: impl Wire2Api<Ipv4AddrWrap> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "queue_connect__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            let api_ip = ip.wire2api();
            move |task_callback| Ok(Runtime::queue_connect(&api_that, api_ip))
        },
    )
}
fn wire_queue_drop__method__Runtime_impl(
    port_: MessagePort,
    that: impl Wire2Api<Runtime> + UnwindSafe,
    ips: impl Wire2Api<Vec<Ipv4AddrWrap>> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "queue_drop__method__Runtime",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_that = that.wire2api();
            let api_ips = ips.wire2api();
            move |task_callback| Ok(Runtime::queue_drop(&api_that, api_ips))
        },
    )
}
// Section: wrapper structs

#[derive(Clone)]
struct mirror_Balance(Balance);

#[derive(Clone)]
struct mirror_Flux(Flux);

#[derive(Clone)]
struct mirror_NetDateTime(NetDateTime);

#[derive(Clone)]
struct mirror_NetDetail(NetDetail);

#[derive(Clone)]
struct mirror_NetFlux(NetFlux);

#[derive(Clone)]
struct mirror_NetState(NetState);

#[derive(Clone)]
struct mirror_NetStatus(NetStatus);

#[derive(Clone)]
struct mirror_NewDuration(NewDuration);

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
        let NetDateTime_ = None::<NetDateTime>.unwrap();
        let _: chrono::NaiveDateTime = NetDateTime_.0;
    }
    {
        let NetDetail = None::<NetDetail>.unwrap();
        let _: NetDateTime = NetDetail.login_time;
        let _: NetDateTime = NetDetail.logout_time;
        let _: Flux = NetDetail.flux;
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
    match None::<NetStatus>.unwrap() {
        NetStatus::Unknown => {}
        NetStatus::Wwan => {}
        NetStatus::Wlan(field0) => {
            let _: String = field0;
        }
        NetStatus::Lan => {}
    }
    {
        let NewDuration_ = None::<NewDuration>.unwrap();
        let _: chrono::Duration = NewDuration_.0;
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

impl Wire2Api<i32> for i32 {
    fn wire2api(self) -> i32 {
        self
    }
}

impl Wire2Api<NetState> for i32 {
    fn wire2api(self) -> NetState {
        match self {
            0 => NetState::Unknown,
            1 => NetState::Net,
            2 => NetState::Auth4,
            3 => NetState::Auth6,
            _ => unreachable!("Invalid variant for NetState: {}", self),
        }
    }
}

impl Wire2Api<u8> for u8 {
    fn wire2api(self) -> u8 {
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

impl support::IntoDart for DetailDailyPoint {
    fn into_dart(self) -> support::DartAbi {
        vec![self.day.into_dart(), mirror_Flux(self.flux).into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for DetailDailyPoint {}

impl support::IntoDart for DetailDailyWrap {
    fn into_dart(self) -> support::DartAbi {
        vec![
            self.details.into_dart(),
            self.now_month.into_dart(),
            self.now_day.into_dart(),
            mirror_Flux(self.max_flux).into_dart(),
        ]
        .into_dart()
    }
}
impl support::IntoDartExceptPrimitive for DetailDailyWrap {}

impl support::IntoDart for mirror_Flux {
    fn into_dart(self) -> support::DartAbi {
        vec![self.0 .0.into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_Flux {}

impl support::IntoDart for Ipv4AddrWrap {
    fn into_dart(self) -> support::DartAbi {
        vec![self.octets.into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for Ipv4AddrWrap {}

impl support::IntoDart for mirror_NetDateTime {
    fn into_dart(self) -> support::DartAbi {
        vec![self.0 .0.into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_NetDateTime {}

impl support::IntoDart for mirror_NetDetail {
    fn into_dart(self) -> support::DartAbi {
        vec![
            mirror_NetDateTime(self.0.login_time).into_dart(),
            mirror_NetDateTime(self.0.logout_time).into_dart(),
            mirror_Flux(self.0.flux).into_dart(),
        ]
        .into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_NetDetail {}

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
impl support::IntoDart for mirror_NetStatus {
    fn into_dart(self) -> support::DartAbi {
        match self.0 {
            NetStatus::Unknown => vec![0.into_dart()],
            NetStatus::Wwan => vec![1.into_dart()],
            NetStatus::Wlan(field0) => vec![2.into_dart(), field0.into_dart()],
            NetStatus::Lan => vec![3.into_dart()],
        }
        .into_dart()
    }
}
impl support::IntoDartExceptPrimitive for mirror_NetStatus {}
impl support::IntoDart for NetUserWrap {
    fn into_dart(self) -> support::DartAbi {
        vec![
            self.address.into_dart(),
            mirror_NetDateTime(self.login_time).into_dart(),
            self.mac_address.into_dart(),
            mirror_Flux(self.flux).into_dart(),
            self.is_local.into_dart(),
        ]
        .into_dart()
    }
}
impl support::IntoDartExceptPrimitive for NetUserWrap {}

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

impl support::IntoDart for UpdateMsgWrap {
    fn into_dart(self) -> support::DartAbi {
        match self {
            Self::Credential(field0) => vec![0.into_dart(), field0.into_dart()],
            Self::State(field0) => vec![1.into_dart(), field0.into_dart()],
            Self::Status(field0) => vec![2.into_dart(), field0.into_dart()],
            Self::Log(field0) => vec![3.into_dart(), field0.into_dart()],
            Self::Flux(field0) => vec![4.into_dart(), field0.into_dart()],
            Self::Online(field0) => vec![5.into_dart(), field0.into_dart()],
            Self::Details(field0, field1) => {
                vec![6.into_dart(), field0.into_dart(), field1.into_dart()]
            }
            Self::LogBusy(field0) => vec![7.into_dart(), field0.into_dart()],
            Self::OnlineBusy(field0) => vec![8.into_dart(), field0.into_dart()],
            Self::DetailBusy(field0) => vec![9.into_dart(), field0.into_dart()],
        }
        .into_dart()
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
