use crate::*;
use core_foundation::{
    base::TCFType,
    runloop::{kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopRef},
};
use flume::{r#async::RecvStream, unbounded};
use objc2_core_wlan::CWWiFiClient;
use pin_project::pin_project;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    os::unix::thread::{JoinHandleExt, RawPthread},
    pin::Pin,
    task::{Context, Poll},
    thread::JoinHandle,
};
use system_configuration::network_reachability::{ReachabilityFlags, SCNetworkReachability};

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

pub fn current() -> NetStatus {
    let sc = SCNetworkReachability::from(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0));
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
    NetStatus::Unknown
}

pub fn watch() -> impl Stream<Item = ()> {
    let (tx, rx) = unbounded();
    let loop_thread = std::thread::spawn(move || {
        let mut sc =
            SCNetworkReachability::from(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0));
        sc.set_callback(move |_| {
            tx.send(()).ok();
        })
        .unwrap_or_else(|e| log::warn!("{}", e));
        unsafe { sc.schedule_with_runloop(&CFRunLoop::get_current(), kCFRunLoopDefaultMode) }
            .unwrap_or_else(|e| log::warn!("{}", e));
        CFRunLoop::run_current();
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
            unsafe { CFRunLoop::wrap_under_get_rule(_CFRunLoopGet0(handle.as_pthread_t())) }.stop();
            if let Err(e) = handle.join() {
                std::panic::resume_unwind(e);
            }
        }
    }
}

extern "C" {
    fn _CFRunLoopGet0(thread: RawPthread) -> CFRunLoopRef;
}
