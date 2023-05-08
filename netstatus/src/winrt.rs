use crate::*;
use futures_util::future::Either;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::watch;
use tokio_stream::{pending, wrappers::WatchStream};
use windows::{core::*, Foundation::EventRegistrationToken, Networking::Connectivity::*};

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
    let (tx, rx) = watch::channel(());
    let token = NetworkInformation::NetworkStatusChanged(&NetworkStatusChangedEventHandler::new(
        move |_| {
            tx.send(()).ok();
            Ok(())
        },
    ))?;
    Ok(StatusWatchStream {
        s: WatchStream::new(rx),
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
    s: WatchStream<()>,
    token: NetworkStatusChangedToken,
}

impl Stream for StatusWatchStream {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().s.poll_next(cx)
    }
}

struct NetworkStatusChangedToken(EventRegistrationToken);

impl Drop for NetworkStatusChangedToken {
    fn drop(&mut self) {
        NetworkInformation::RemoveNetworkStatusChanged(self.0).unwrap()
    }
}
