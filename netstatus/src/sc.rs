#![allow(deprecated)]

use crate::*;
use flume::{r#async::RecvStream, unbounded};
use objc2_core_foundation::{
    kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopGetCurrent, CFRunLoopRun, CFRunLoopStop,
    CFString,
};
use objc2_core_wlan::CWWiFiClient;
use objc2_system_configuration::{
    SCNetworkReachability, SCNetworkReachabilityContext, SCNetworkReachabilityCreateWithAddress,
    SCNetworkReachabilityFlags, SCNetworkReachabilityGetFlags,
    SCNetworkReachabilityScheduleWithRunLoop, SCNetworkReachabilitySetCallback,
};
use pin_project::pin_project;
use std::{
    ffi::c_void,
    mem::MaybeUninit,
    os::unix::thread::{JoinHandleExt, RawPthread},
    pin::Pin,
    ptr::NonNull,
    sync::Arc,
    task::{Context, Poll},
    thread::JoinHandle,
};

unsafe fn get_ssid() -> Option<String> {
    CWWiFiClient::sharedWiFiClient()
        .interface()
        .and_then(|interface| interface.ssid())
        .map(|name| {
            std::ffi::CStr::from_ptr(name.UTF8String())
                .to_string_lossy()
                .into_owned()
        })
}

fn create_reachability() -> CFRetained<SCNetworkReachability> {
    let mut addr = libc::sockaddr_in {
        sin_len: size_of::<libc::sockaddr_in>() as _,
        sin_family: libc::AF_INET as _,
        sin_port: 0,
        sin_addr: libc::in_addr { s_addr: 0 },
        sin_zero: Default::default(),
    };
    unsafe { SCNetworkReachabilityCreateWithAddress(None, NonNull::from(&mut addr).cast()) }
        .unwrap()
}

pub fn current() -> NetStatus {
    let sc = create_reachability();
    let mut flag = MaybeUninit::uninit();
    if unsafe { SCNetworkReachabilityGetFlags(&sc, NonNull::new_unchecked(flag.as_mut_ptr())) } {
        let flag = unsafe { flag.assume_init() };
        if !flag.contains(SCNetworkReachabilityFlags::Reachable) {
            return NetStatus::Unknown;
        }
        if !flag.contains(SCNetworkReachabilityFlags::ConnectionRequired)
            || ((flag.contains(SCNetworkReachabilityFlags::ConnectionOnDemand)
                || flag.contains(SCNetworkReachabilityFlags::ConnectionOnTraffic))
                && !flag.contains(SCNetworkReachabilityFlags::InterventionRequired))
        {
            return match unsafe { get_ssid() } {
                Some(ssid) => NetStatus::Wlan(ssid),
                None => NetStatus::Unknown,
            };
        }
        if flag == SCNetworkReachabilityFlags::IsWWAN {
            return NetStatus::Wwan;
        }
    }
    NetStatus::Unknown
}

pub fn watch() -> impl Stream<Item = ()> {
    let (tx, rx) = unbounded();
    let loop_thread = std::thread::spawn(move || {
        let sc = create_reachability();
        set_callback(sc.clone(), move |_| {
            tx.send(()).ok();
        });
        let run_loop = unsafe { CFRunLoopGetCurrent() }.unwrap();
        unsafe {
            SCNetworkReachabilityScheduleWithRunLoop(
                &sc,
                &run_loop,
                kCFRunLoopDefaultMode.unwrap_unchecked(),
            )
        };
        unsafe { CFRunLoopRun() };
    });
    StatusWatchStream {
        rx: rx.into_stream(),
        thread: CFJThread {
            handle: Some(loop_thread),
        },
    }
}

#[pin_project]
struct StatusWatchStream {
    #[pin]
    rx: RecvStream<'static, ()>,
    thread: CFJThread,
}

impl Stream for StatusWatchStream {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().rx.poll_next(cx)
    }
}

struct CFJThread {
    handle: Option<JoinHandle<()>>,
}

impl Drop for CFJThread {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let run_loop =
                unsafe { _CFRunLoopGet0(handle.as_pthread_t()).map(|ret| CFRetained::retain(ret)) }
                    .unwrap();
            unsafe { CFRunLoopStop(&run_loop) };
            if let Err(e) = handle.join() {
                std::panic::resume_unwind(e);
            }
        }
    }
}

extern "C-unwind" {
    fn _CFRunLoopGet0(thread: RawPthread) -> Option<NonNull<CFRunLoop>>;
}

fn set_callback<F: Fn(SCNetworkReachabilityFlags) + Sync + Send>(
    sc: CFRetained<SCNetworkReachability>,
    callback: F,
) -> bool {
    let callback = Arc::new(NetworkReachabilityCallbackContext::new(
        sc.clone(),
        callback,
    ));
    let mut callback_context = SCNetworkReachabilityContext {
        version: 0,
        info: Arc::into_raw(callback) as *mut _,
        retain: Some(NetworkReachabilityCallbackContext::<F>::retain_context),
        release: Some(NetworkReachabilityCallbackContext::<F>::release_context),
        copyDescription: Some(NetworkReachabilityCallbackContext::<F>::copy_ctx_description),
    };
    unsafe {
        SCNetworkReachabilitySetCallback(
            &sc,
            Some(NetworkReachabilityCallbackContext::<F>::callback),
            &mut callback_context,
        )
    }
}

struct NetworkReachabilityCallbackContext<T: Fn(SCNetworkReachabilityFlags) + Sync + Send> {
    _host: CFRetained<SCNetworkReachability>,
    callback: T,
}

impl<T: Fn(SCNetworkReachabilityFlags) + Sync + Send> NetworkReachabilityCallbackContext<T> {
    fn new(host: CFRetained<SCNetworkReachability>, callback: T) -> Self {
        Self {
            _host: host,
            callback,
        }
    }

    extern "C-unwind" fn callback(
        _target: NonNull<SCNetworkReachability>,
        flags: SCNetworkReachabilityFlags,
        context: *mut c_void,
    ) {
        let context: &mut Self = unsafe { &mut (*(context as *mut _)) };
        (context.callback)(flags);
    }

    extern "C-unwind" fn copy_ctx_description(_ctx: NonNull<c_void>) -> NonNull<CFString> {
        let description = CFString::from_static_str("NetworkRechability's callback context");
        CFRetained::into_raw(description)
    }

    extern "C-unwind" fn release_context(ctx: NonNull<c_void>) {
        unsafe {
            Arc::decrement_strong_count(ctx.as_ptr() as *mut Self);
        }
    }

    extern "C-unwind" fn retain_context(ctx_ptr: NonNull<c_void>) -> NonNull<c_void> {
        unsafe {
            Arc::increment_strong_count(ctx_ptr.as_ptr() as *mut Self);
        }
        ctx_ptr
    }
}
