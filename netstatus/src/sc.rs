use crate::*;
use anyhow::{anyhow, Result};
use core_foundation::{
    base::TCFType,
    runloop::{kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopRef},
};
use objc::{
    runtime::{Class, Object},
    *,
};
use pin_project::pin_project;
use std::{
    ffi::CStr,
    os::unix::thread::{JoinHandleExt, RawPthread},
    pin::Pin,
    task::{Context, Poll},
    thread::JoinHandle,
};
use system_configuration::network_reachability::{ReachabilityFlags, SCNetworkReachability};
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

#[link(name = "CoreWLAN", kind = "framework")]
extern "C" {
    #[link_name = "OBJC_CLASS_$_CWWiFiClient"]
    static OBJC_CLASS__CWWiFiClient: Class;
}

unsafe fn get_ssid() -> Option<String> {
    let client: *mut Object = msg_send![&OBJC_CLASS__CWWiFiClient, sharedWiFiClient];
    let interface: *mut Object = msg_send![client, interface];
    let name: *mut Object = msg_send![interface, ssid];
    if !name.is_null() {
        let name = std::ffi::CStr::from_ptr(msg_send![name, UTF8String])
            .to_string_lossy()
            .into_owned();
        Some(name)
    } else {
        None
    }
}

pub fn current() -> NetStatus {
    let host = unsafe { CStr::from_bytes_with_nul_unchecked(b"0.0.0.0\0") };
    if let Some(sc) = SCNetworkReachability::from_host(host) {
        if let Ok(flag) = sc.reachability() {
            if !flag.contains(ReachabilityFlags::REACHABLE) {
                return NetStatus::Unknown;
            }
            if !flag.contains(ReachabilityFlags::CONNECTION_REQUIRED)
                || ((flag.contains(ReachabilityFlags::CONNECTION_ON_DEMAND)
                    || flag.contains(ReachabilityFlags::CONNECTION_ON_TRAFFIC))
                    && !flag.contains(ReachabilityFlags::INTERVENTION_REQUIRED))
            {
                return match unsafe { get_ssid() } {
                    Some(ssid) => NetStatus::Wlan(ssid),
                    None => NetStatus::Unknown,
                };
            }
            if flag == ReachabilityFlags::IS_WWAN {
                return NetStatus::Wwan;
            }
        }
    }
    NetStatus::Unknown
}

pub fn watch() -> impl Stream<Item = ()> {
    let (tx, rx) = watch::channel(());
    let loop_thread = std::thread::spawn(move || -> Result<()> {
        let host = unsafe { CStr::from_bytes_with_nul_unchecked(b"0.0.0.0\0") };
        let mut sc = SCNetworkReachability::from_host(host)
            .ok_or_else(|| anyhow!("Cannot get network reachability"))?;
        sc.set_callback(move |_| {
            tx.send(()).ok();
        })?;
        unsafe {
            sc.schedule_with_runloop(&CFRunLoop::get_current(), kCFRunLoopDefaultMode)?;
        }
        CFRunLoop::run_current();
        Ok(())
    });
    StatusWatchStream {
        s: WatchStream::new(rx),
        thread: CFJThread {
            handle: Some(loop_thread),
        },
    }
}

#[pin_project]
struct StatusWatchStream {
    #[pin]
    s: WatchStream<()>,
    thread: CFJThread,
}

impl Stream for StatusWatchStream {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().s.poll_next(cx)
    }
}

struct CFJThread {
    handle: Option<JoinHandle<Result<()>>>,
}

impl Drop for CFJThread {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            unsafe { CFRunLoop::wrap_under_get_rule(_CFRunLoopGet0(handle.as_pthread_t())) }.stop();
            match handle.join() {
                Ok(res) => res.unwrap(),
                Err(e) => std::panic::resume_unwind(e),
            }
        }
    }
}

extern "C" {
    fn _CFRunLoopGet0(thread: RawPthread) -> CFRunLoopRef;
}
