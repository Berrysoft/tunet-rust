use super::*;
// Section: wire functions

#[no_mangle]
pub extern "C" fn wire_new__static_method__Runtime(port_: i64) {
    wire_new__static_method__Runtime_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_start__method__Runtime(port_: i64, that: *mut wire_Runtime) {
    wire_start__method__Runtime_impl(port_, that)
}

// Section: allocate functions

#[no_mangle]
pub extern "C" fn new_MutexModel() -> wire_MutexModel {
    wire_MutexModel::new_with_null_ptr()
}

#[no_mangle]
pub extern "C" fn new_MutexOptionMpscReceiverAction() -> wire_MutexOptionMpscReceiverAction {
    wire_MutexOptionMpscReceiverAction::new_with_null_ptr()
}

#[no_mangle]
pub extern "C" fn new_box_autoadd_runtime_0() -> *mut wire_Runtime {
    support::new_leak_box_ptr(wire_Runtime::new_with_null_ptr())
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

// Section: impl Wire2Api

impl Wire2Api<RustOpaque<Mutex<Model>>> for wire_MutexModel {
    fn wire2api(self) -> RustOpaque<Mutex<Model>> {
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
impl Wire2Api<Runtime> for *mut wire_Runtime {
    fn wire2api(self) -> Runtime {
        let wrap = unsafe { support::box_from_leak_ptr(self) };
        Wire2Api::<Runtime>::wire2api(*wrap).into()
    }
}
impl Wire2Api<Runtime> for wire_Runtime {
    fn wire2api(self) -> Runtime {
        Runtime {
            rx: self.rx.wire2api(),
            model: self.model.wire2api(),
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
pub struct wire_MutexOptionMpscReceiverAction {
    ptr: *const core::ffi::c_void,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_Runtime {
    rx: wire_MutexOptionMpscReceiverAction,
    model: wire_MutexModel,
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
impl NewWithNullPtr for wire_MutexOptionMpscReceiverAction {
    fn new_with_null_ptr() -> Self {
        Self {
            ptr: core::ptr::null(),
        }
    }
}

impl NewWithNullPtr for wire_Runtime {
    fn new_with_null_ptr() -> Self {
        Self {
            rx: wire_MutexOptionMpscReceiverAction::new_with_null_ptr(),
            model: wire_MutexModel::new_with_null_ptr(),
        }
    }
}

impl Default for wire_Runtime {
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
