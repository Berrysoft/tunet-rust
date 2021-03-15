use crate::*;
use libc::{sockaddr, sockaddr_in, AF_INET};
use std::ffi::c_void;
use std::mem::{size_of, MaybeUninit};

#[cfg(target_os = "macos")]
use objc::{
    rc::StrongPtr,
    runtime::{Class, Object},
    *,
};

type CFAllocatorRef = *mut c_void;
type CFTypeRef = *const c_void;
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
    IsWWAN = 1 << 18,
}

impl SCNetworkReachabilityFlags {
    pub const fn has(&self, f: Self) -> bool {
        (*self as u32 & f as u32) != 0
    }

    pub const fn has_only(&self, f: Self) -> bool {
        (*self as u32 & f as u32) == f as u32
    }
}

#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "SystemConfiguration", kind = "framework")]
extern "C" {
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
struct CFObject(*mut c_void);

impl Drop for CFObject {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0) }
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn get_ssid() -> Option<String> {
    #[link(name = "CoreWLAN", kind = "framework")]
    extern "C" {
        #[link_name = "OBJC_CLASS_$_CWWiFiClient"]
        static OBJC_CLASS__CWWiFiClient: Class;
    }

    let client = StrongPtr::new(msg_send![&OBJC_CLASS__CWWiFiClient, sharedWiFiClient]);
    let interface = StrongPtr::new(msg_send![*client, interface]);
    let name: *mut Object = msg_send![*interface, ssid];
    if !name.is_null() {
        let name = StrongPtr::new(name);
        let name = std::ffi::CStr::from_ptr(msg_send![*name, UTF8String])
            .to_string_lossy()
            .into_owned();
        Some(name)
    } else {
        None
    }
}

#[cfg(target_os = "ios")]
unsafe fn get_ssid() -> Option<String> {
    type CFArrayRef = *mut c_void;
    type CFDictionaryRef = *mut c_void;
    type CFStringRef = *mut c_void;
    type CFIndex = std::os::raw::c_long;
    type CFStringEncoding = u32;

    extern "C" {
        static kCNNetworkInfoKeySSID: CFStringRef;

        fn CNCopySupportedInterfaces() -> CFArrayRef;
        fn CNCopyCurrentNetworkInfo(interface: CFStringRef) -> CFDictionaryRef;

        fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: CFIndex) -> *const c_void;

        fn CFDictionaryGetValue(dict: CFDictionaryRef, key: *const c_void) -> *const c_void;

        fn CFStringGetLength(std: CFStringRef) -> CFIndex;
        fn CFStringGetCString(
            str: CFStringRef,
            buffer: *mut std::os::raw::c_char,
            size: CFIndex,
            enc: CFStringEncoding,
        ) -> Boolean;
    }

    #[allow(non_upper_case_globals)]
    const kCFStringEncodingUTF8: CFStringEncoding = 0x08000100;

    let arr = CNCopySupportedInterfaces();
    if arr.is_null() {
        None
    } else {
        let interface = CFObject(CFArrayGetValueAtIndex(arr, 0) as _);
        let dict = CFObject(CNCopyCurrentNetworkInfo(interface.0));
        let ssid = CFObject(CFDictionaryGetValue(dict.0, kCNNetworkInfoKeySSID) as _);
        let len = CFStringGetLength(ssid.0);
        if len == 0 {
            Some(String::new())
        } else {
            let mut buffer = vec![0u8; (len + 1) as usize];
            if CFStringGetCString(
                ssid.0,
                buffer.as_mut_ptr() as _,
                len + 1,
                kCFStringEncodingUTF8,
            ) != 0
            {
                buffer.pop();
                Some(String::from_utf8_unchecked(buffer))
            } else {
                None
            }
        }
    }
}

pub fn current() -> NetStatus {
    unsafe {
        let mut addr = MaybeUninit::<sockaddr_in>::zeroed().assume_init();
        addr.sin_len = size_of::<sockaddr_in>() as _;
        addr.sin_family = AF_INET as _;
        let reach = CFObject(SCNetworkReachabilityCreateWithAddress(
            kCFAllocatorDefault,
            &addr as *const sockaddr_in as _,
        ));
        if reach.0.is_null() {
            NetStatus::Unknown
        } else {
            let mut flag = SCNetworkReachabilityFlags::InvalidValue;
            if SCNetworkReachabilityGetFlags(reach.0, &mut flag) == 0 {
                NetStatus::Unknown
            } else {
                if !flag.has(SCNetworkReachabilityFlags::Reachable) {
                    return NetStatus::Unknown;
                }
                if !flag.has(SCNetworkReachabilityFlags::ConnectionRequired) {
                    return match get_ssid() {
                        Some(ssid) => NetStatus::Wlan(ssid),
                        None => NetStatus::Unknown,
                    };
                }
                if flag.has(SCNetworkReachabilityFlags::ConnectionOnDemand)
                    || flag.has(SCNetworkReachabilityFlags::ConnectionOnTraffic)
                {
                    if !flag.has(SCNetworkReachabilityFlags::InterventionRequired) {
                        return match get_ssid() {
                            Some(ssid) => NetStatus::Wlan(ssid),
                            None => NetStatus::Unknown,
                        };
                    }
                }
                if flag.has_only(SCNetworkReachabilityFlags::IsWWAN) {
                    return NetStatus::Wwan;
                }
                return NetStatus::Unknown;
            }
        }
    }
}
