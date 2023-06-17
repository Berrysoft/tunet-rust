use super::*;
// Section: wire functions

#[no_mangle]
pub extern "C" fn wire_flux_to_string(port_: i64, f: u64) {
    wire_flux_to_string_impl(port_, f)
}

#[no_mangle]
pub extern "C" fn wire_new__static_method__RuntimeStartConfig(
    port_: i64,
    status: i32,
    ssid: *mut wire_uint_8_list,
    username: *mut wire_uint_8_list,
    password: *mut wire_uint_8_list,
) {
    wire_new__static_method__RuntimeStartConfig_impl(port_, status, ssid, username, password)
}

#[no_mangle]
pub extern "C" fn wire_new__static_method__Runtime(port_: i64) {
    wire_new__static_method__Runtime_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_start__method__Runtime(
    port_: i64,
    that: *mut wire_Runtime,
    config: *mut wire_RuntimeStartConfig,
) {
    wire_start__method__Runtime_impl(port_, that, config)
}

#[no_mangle]
pub extern "C" fn wire_queue_credential__method__Runtime(
    port_: i64,
    that: *mut wire_Runtime,
    u: *mut wire_uint_8_list,
    p: *mut wire_uint_8_list,
) {
    wire_queue_credential__method__Runtime_impl(port_, that, u, p)
}

#[no_mangle]
pub extern "C" fn wire_queue_login__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_queue_login__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_queue_logout__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_queue_logout__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_queue_flux__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_queue_flux__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_queue_state__method__Runtime(
    port_: i64,
    that: *mut wire_Runtime,
    s: *mut wire_NetStateWrap,
) {
    wire_queue_state__method__Runtime_impl(port_, that, s)
}

#[no_mangle]
pub extern "C" fn wire_queue_details__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_queue_details__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_queue_onlines__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_queue_onlines__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_log_busy__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_log_busy__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_log_text__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_log_text__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_flux__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_flux__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_state__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_state__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_status__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_status__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_detail_busy__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_detail_busy__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_details__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_details__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_detail_daily__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_detail_daily__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_username__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_username__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_online_busy__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_online_busy__method__Runtime_impl(port_, that)
}

#[no_mangle]
pub extern "C" fn wire_onlines__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_onlines__method__Runtime_impl(port_, that)
}

// Section: allocate functions

#[no_mangle]
pub extern "C" fn new_MutexModel() -> wire_MutexModel {
    wire_MutexModel::new_with_null_ptr()
}

#[no_mangle]
pub extern "C" fn new_MutexOptionHandle() -> wire_MutexOptionHandle {
    wire_MutexOptionHandle::new_with_null_ptr()
}

#[no_mangle]
pub extern "C" fn new_MutexOptionMpscReceiverAction() -> wire_MutexOptionMpscReceiverAction {
    wire_MutexOptionMpscReceiverAction::new_with_null_ptr()
}

#[no_mangle]
pub extern "C" fn new_NetStatus() -> wire_NetStatus {
    wire_NetStatus::new_with_null_ptr()
}

#[no_mangle]
pub extern "C" fn new_box_autoadd_net_state_wrap_0() -> *mut wire_NetStateWrap {
    support::new_leak_box_ptr(wire_NetStateWrap::new_with_null_ptr())
}

#[no_mangle]
pub extern "C" fn new_box_autoadd_runtime_0() -> *mut wire_Runtime {
    support::new_leak_box_ptr(wire_Runtime::new_with_null_ptr())
}

#[no_mangle]
pub extern "C" fn new_box_autoadd_runtime_start_config_0() -> *mut wire_RuntimeStartConfig {
    support::new_leak_box_ptr(wire_RuntimeStartConfig::new_with_null_ptr())
}

#[no_mangle]
pub extern "C" fn new_uint_8_list_0(len: i32) -> *mut wire_uint_8_list {
    let ans = wire_uint_8_list {
        ptr: support::new_leak_vec_ptr(Default::default(), len),
        len,
    };
    support::new_leak_box_ptr(ans)
}

// Section: related functions

#[no_mangle]
pub extern "C" fn drop_opaque_MutexModel(ptr: *const c_void) {
    unsafe {
        Arc::<Mutex<Model>>::decrement_strong_count(ptr as _);
    }
}

#[no_mangle]
pub extern "C" fn share_opaque_MutexModel(ptr: *const c_void) -> *const c_void {
    unsafe {
        Arc::<Mutex<Model>>::increment_strong_count(ptr as _);
        ptr
    }
}

#[no_mangle]
pub extern "C" fn drop_opaque_MutexOptionHandle(ptr: *const c_void) {
    unsafe {
        Arc::<Mutex<Option<Handle>>>::decrement_strong_count(ptr as _);
    }
}

#[no_mangle]
pub extern "C" fn share_opaque_MutexOptionHandle(ptr: *const c_void) -> *const c_void {
    unsafe {
        Arc::<Mutex<Option<Handle>>>::increment_strong_count(ptr as _);
        ptr
    }
}

#[no_mangle]
pub extern "C" fn drop_opaque_MutexOptionMpscReceiverAction(ptr: *const c_void) {
    unsafe {
        Arc::<Mutex<Option<mpsc::Receiver<Action>>>>::decrement_strong_count(ptr as _);
    }
}

#[no_mangle]
pub extern "C" fn share_opaque_MutexOptionMpscReceiverAction(ptr: *const c_void) -> *const c_void {
    unsafe {
        Arc::<Mutex<Option<mpsc::Receiver<Action>>>>::increment_strong_count(ptr as _);
        ptr
    }
}

#[no_mangle]
pub extern "C" fn drop_opaque_NetStatus(ptr: *const c_void) {
    unsafe {
        Arc::<NetStatus>::decrement_strong_count(ptr as _);
    }
}

#[no_mangle]
pub extern "C" fn share_opaque_NetStatus(ptr: *const c_void) -> *const c_void {
    unsafe {
        Arc::<NetStatus>::increment_strong_count(ptr as _);
        ptr
    }
}

// Section: impl Wire2Api

impl Wire2Api<RustOpaque<Mutex<Model>>> for wire_MutexModel {
    fn wire2api(self) -> RustOpaque<Mutex<Model>> {
        unsafe { support::opaque_from_dart(self.ptr as _) }
    }
}
impl Wire2Api<RustOpaque<Mutex<Option<Handle>>>> for wire_MutexOptionHandle {
    fn wire2api(self) -> RustOpaque<Mutex<Option<Handle>>> {
        unsafe { support::opaque_from_dart(self.ptr as _) }
    }
}
impl Wire2Api<RustOpaque<Mutex<Option<mpsc::Receiver<Action>>>>>
    for wire_MutexOptionMpscReceiverAction
{
    fn wire2api(self) -> RustOpaque<Mutex<Option<mpsc::Receiver<Action>>>> {
        unsafe { support::opaque_from_dart(self.ptr as _) }
    }
}
impl Wire2Api<RustOpaque<NetStatus>> for wire_NetStatus {
    fn wire2api(self) -> RustOpaque<NetStatus> {
        unsafe { support::opaque_from_dart(self.ptr as _) }
    }
}
impl Wire2Api<String> for *mut wire_uint_8_list {
    fn wire2api(self) -> String {
        let vec: Vec<u8> = self.wire2api();
        String::from_utf8_lossy(&vec).into_owned()
    }
}
impl Wire2Api<NetStateWrap> for *mut wire_NetStateWrap {
    fn wire2api(self) -> NetStateWrap {
        let wrap = unsafe { support::box_from_leak_ptr(self) };
        Wire2Api::<NetStateWrap>::wire2api(*wrap).into()
    }
}
impl Wire2Api<Runtime> for *mut wire_Runtime {
    fn wire2api(self) -> Runtime {
        let wrap = unsafe { support::box_from_leak_ptr(self) };
        Wire2Api::<Runtime>::wire2api(*wrap).into()
    }
}
impl Wire2Api<RuntimeStartConfig> for *mut wire_RuntimeStartConfig {
    fn wire2api(self) -> RuntimeStartConfig {
        let wrap = unsafe { support::box_from_leak_ptr(self) };
        Wire2Api::<RuntimeStartConfig>::wire2api(*wrap).into()
    }
}

impl Wire2Api<NetStateWrap> for wire_NetStateWrap {
    fn wire2api(self) -> NetStateWrap {
        NetStateWrap(self.field0.wire2api())
    }
}

impl Wire2Api<Runtime> for wire_Runtime {
    fn wire2api(self) -> Runtime {
        Runtime {
            rx: self.rx.wire2api(),
            model: self.model.wire2api(),
            handle: self.handle.wire2api(),
        }
    }
}
impl Wire2Api<RuntimeStartConfig> for wire_RuntimeStartConfig {
    fn wire2api(self) -> RuntimeStartConfig {
        RuntimeStartConfig {
            status: self.status.wire2api(),
            username: self.username.wire2api(),
            password: self.password.wire2api(),
        }
    }
}

impl Wire2Api<Vec<u8>> for *mut wire_uint_8_list {
    fn wire2api(self) -> Vec<u8> {
        unsafe {
            let wrap = support::box_from_leak_ptr(self);
            support::vec_from_leak_ptr(wrap.ptr, wrap.len)
        }
    }
}
// Section: wire structs

#[repr(C)]
#[derive(Clone)]
pub struct wire_MutexModel {
    ptr: *const core::ffi::c_void,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_MutexOptionHandle {
    ptr: *const core::ffi::c_void,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_MutexOptionMpscReceiverAction {
    ptr: *const core::ffi::c_void,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_NetStatus {
    ptr: *const core::ffi::c_void,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_NetStateWrap {
    field0: i32,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_Runtime {
    rx: wire_MutexOptionMpscReceiverAction,
    model: wire_MutexModel,
    handle: wire_MutexOptionHandle,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_RuntimeStartConfig {
    status: wire_NetStatus,
    username: *mut wire_uint_8_list,
    password: *mut wire_uint_8_list,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_uint_8_list {
    ptr: *mut u8,
    len: i32,
}

// Section: impl NewWithNullPtr

pub trait NewWithNullPtr {
    fn new_with_null_ptr() -> Self;
}

impl<T> NewWithNullPtr for *mut T {
    fn new_with_null_ptr() -> Self {
        std::ptr::null_mut()
    }
}

impl NewWithNullPtr for wire_MutexModel {
    fn new_with_null_ptr() -> Self {
        Self {
            ptr: core::ptr::null(),
        }
    }
}
impl NewWithNullPtr for wire_MutexOptionHandle {
    fn new_with_null_ptr() -> Self {
        Self {
            ptr: core::ptr::null(),
        }
    }
}
impl NewWithNullPtr for wire_MutexOptionMpscReceiverAction {
    fn new_with_null_ptr() -> Self {
        Self {
            ptr: core::ptr::null(),
        }
    }
}
impl NewWithNullPtr for wire_NetStatus {
    fn new_with_null_ptr() -> Self {
        Self {
            ptr: core::ptr::null(),
        }
    }
}

impl NewWithNullPtr for wire_NetStateWrap {
    fn new_with_null_ptr() -> Self {
        Self {
            field0: Default::default(),
        }
    }
}

impl Default for wire_NetStateWrap {
    fn default() -> Self {
        Self::new_with_null_ptr()
    }
}

impl NewWithNullPtr for wire_Runtime {
    fn new_with_null_ptr() -> Self {
        Self {
            rx: wire_MutexOptionMpscReceiverAction::new_with_null_ptr(),
            model: wire_MutexModel::new_with_null_ptr(),
            handle: wire_MutexOptionHandle::new_with_null_ptr(),
        }
    }
}

impl Default for wire_Runtime {
    fn default() -> Self {
        Self::new_with_null_ptr()
    }
}

impl NewWithNullPtr for wire_RuntimeStartConfig {
    fn new_with_null_ptr() -> Self {
        Self {
            status: wire_NetStatus::new_with_null_ptr(),
            username: core::ptr::null_mut(),
            password: core::ptr::null_mut(),
        }
    }
}

impl Default for wire_RuntimeStartConfig {
    fn default() -> Self {
        Self::new_with_null_ptr()
    }
}

// Section: sync execution mode utility

#[no_mangle]
pub extern "C" fn free_WireSyncReturn(ptr: support::WireSyncReturn) {
    unsafe {
        let _ = support::box_from_leak_ptr(ptr);
    };
}
