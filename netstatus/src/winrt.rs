use crate::*;
use flume::{r#async::RecvStream, unbounded};
use futures_util::{future::Either, stream::pending};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use windows::{core::*, Networking::Connectivity::*};

fn current_impl() -> Result<NetStatus> {
    let profile = NetworkInformation::GetInternetConnectionProfile()?;
    let cl = profile.GetNetworkConnectivityLevel()?;
    if cl == NetworkConnectivityLevel::None {
        Ok(NetStatus::Unknown)
    } else if profile.IsWlanConnectionProfile()? {
        let ssid = profile.WlanConnectionProfileDetails()?.GetConnectedSsid()?;
        Ok(NetStatus::Wlan(ssid.to_string_lossy()))
    } else if profile.IsWwanConnectionProfile()? {
        Ok(NetStatus::Wwan)
    } else {
        Ok(NetStatus::Lan)
    }
}

pub fn current() -> NetStatus {
    current_impl().unwrap_or_else(|e| {
        log::warn!("{}", e.message());
        NetStatus::Unknown
    })
}

fn watch_impl() -> Result<impl Stream<Item = ()>> {
    let (tx, rx) = unbounded();
    let token = NetworkInformation::NetworkStatusChanged(&NetworkStatusChangedEventHandler::new(
        move |_| {
            tx.send(()).ok();
            Ok(())
        },
    ))?;
    Ok(StatusWatchStream {
        rx: rx.into_stream(),
        token: NetworkStatusChangedToken(token),
    })
}

pub fn watch() -> impl Stream<Item = ()> {
    watch_impl().map(Either::Left).unwrap_or_else(|e| {
        log::warn!("{}", e.message());
        Either::Right(pending())
    })
}

#[pin_project]
struct StatusWatchStream {
    #[pin]
    rx: RecvStream<'static, ()>,
    token: NetworkStatusChangedToken,
}

impl Stream for StatusWatchStream {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().rx.poll_next(cx)
    }
}

struct NetworkStatusChangedToken(i64);

impl Drop for NetworkStatusChangedToken {
    fn drop(&mut self) {
        NetworkInformation::RemoveNetworkStatusChanged(self.0).unwrap()
    }
}
