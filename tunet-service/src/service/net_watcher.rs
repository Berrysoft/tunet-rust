use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::watch;
use tokio_stream::{wrappers::WatchStream, Stream};
use windows::{core::*, Foundation::EventRegistrationToken, Networking::Connectivity::*};

pub fn watch() -> Result<impl Stream<Item = ()>> {
    let (tx, rx) = watch::channel(());
    let token = NetworkInformation::NetworkStatusChanged(&NetworkStatusChangedEventHandler::new(
        move |_| {
            log::debug!("NetworkStatusChanged");
            tx.send(()).ok();
            Ok(())
        },
    ))?;
    Ok(PickFilesStream {
        s: WatchStream::new(rx),
        token: NetworkStatusChangedToken(token),
    })
}

#[pin_project]
struct PickFilesStream<S: Stream<Item = ()> + Send + Sync> {
    #[pin]
    s: S,
    token: NetworkStatusChangedToken,
}

impl<S: Stream<Item = ()> + Send + Sync> Stream for PickFilesStream<S> {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().s.poll_next(cx)
    }
}

struct NetworkStatusChangedToken(EventRegistrationToken);

impl Drop for NetworkStatusChangedToken {
    fn drop(&mut self) {
        log::debug!("RemoveNetworkStatusChanged");
        NetworkInformation::RemoveNetworkStatusChanged(self.0).unwrap()
    }
}
