use super::SUGGEST_SSID_MAP;
use crate::*;
use libc::{sockaddr, sockaddr_in, AF_INET};
use objc::{rc::StrongPtr, runtime::Class, *};
use std::ffi::{c_void, CStr};
use std::mem::{size_of, MaybeUninit};

type CFTypeRef = *const c_void;
type CFAllocatorRef = *mut c_void;
type SCNetworkReachabilityRef = *mut c_void;
type Boolean = u8;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum SCNetworkReachabilityFlags {
    InvalidValue = 0,
    Reachable = 1 << 1,
    ConnectionRequired = 1 << 2,
    ConnectionOnTraffic = 1 << 3,
    InterventionRequired = 1 << 4,
    ConnectionOnDemand = 1 << 5,
}

impl SCNetworkReachabilityFlags {
    pub const fn has_flag(&self, f: Self) -> bool {
        (*self as u32 & f as u32) != 0
    }
}

#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "CoreWLAN", kind = "framework")]
#[link(name = "SystemConfiguration", kind = "framework")]
extern "C" {
    #[link_name = "OBJC_CLASS_$_CWWiFiClient"]
    static OBJC_CLASS__CWWiFiClient: Class;

    static kCFAllocatorDefault: CFAllocatorRef;

    fn CFRelease(cf: CFTypeRef);

    fn SCNetworkReachabilityCreateWithAddress(
        allocator: CFAllocatorRef,
        address: *const sockaddr,
    ) -> SCNetworkReachabilityRef;

    fn SCNetworkReachabilityGetFlags(
        target: SCNetworkReachabilityRef,
        flags: *mut SCNetworkReachabilityFlags,
    ) -> Boolean;
}

#[repr(transparent)]
#[derive(Debug)]
struct SCNetworkReachability(SCNetworkReachabilityRef);

impl Drop for SCNetworkReachability {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0) }
        }
    }
}

const fn is_wlan(flag: SCNetworkReachabilityFlags) -> bool {
    if !flag.has_flag(SCNetworkReachabilityFlags::Reachable) {
        return false;
    }
    if !flag.has_flag(SCNetworkReachabilityFlags::ConnectionRequired) {
        return true;
    }
    if flag.has_flag(SCNetworkReachabilityFlags::ConnectionOnDemand)
        || flag.has_flag(SCNetworkReachabilityFlags::ConnectionOnTraffic)
    {
        if !flag.has_flag(SCNetworkReachabilityFlags::InterventionRequired) {
            return true;
        }
    }
    false
}

unsafe fn get_ssid() -> String {
    let client = StrongPtr::new(msg_send![&OBJC_CLASS__CWWiFiClient, sharedWiFiClient]);
    let interface = StrongPtr::new(msg_send![*client, interface]);
    let name = StrongPtr::new(msg_send![*interface, ssid]);
    CStr::from_ptr(msg_send![*name, UTF8String])
        .to_string_lossy()
        .into_owned()
}

pub fn suggest() -> NetState {
    unsafe {
        let mut addr = MaybeUninit::<sockaddr_in>::zeroed().assume_init();
        addr.sin_len = size_of::<sockaddr_in>() as _;
        addr.sin_family = AF_INET as _;
        let reach = SCNetworkReachability(SCNetworkReachabilityCreateWithAddress(
            kCFAllocatorDefault,
            &addr as *const sockaddr_in as _,
        ));
        if reach.0.is_null() {
            NetState::Unknown
        } else {
            let mut flag = SCNetworkReachabilityFlags::InvalidValue;
            if SCNetworkReachabilityGetFlags(reach.0, &mut flag) == 0 {
                NetState::Unknown
            } else {
                if is_wlan(flag) {
                    let ssid = get_ssid();
                    SUGGEST_SSID_MAP
                        .get(ssid.as_str())
                        .copied()
                        .unwrap_or(NetState::Unknown)
                } else {
                    NetState::Unknown
                }
            }
        }
    }
}
