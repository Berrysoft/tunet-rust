use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::watch;
use tokio_stream::{wrappers::WatchStream, Stream};
use tunet_helper::NetFlux;
use windows::{
    core::*,
    Data::Xml::Dom::XmlDocument,
    Foundation::EventRegistrationToken,
    Networking::Connectivity::*,
    UI::Notifications::{ToastNotification, ToastNotificationManager},
};

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

pub fn succeeded(flux: NetFlux) -> Result<()> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
    <toast>
        <visual>
            <binding template="ToastGeneric">
                <text hint-maxLines="1">登录成功：{0}</text>
                <text>流量：{1}</text>
                <text>余额：{2}</text>
            </binding>
        </visual>
    </toast>"#,
        flux.username, flux.flux, flux.balance
    );
    let dom = XmlDocument::new()?;
    dom.LoadXml(&HSTRING::from(xml))?;
    let notification = ToastNotification::CreateToastNotification(&dom)?;
    ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(
        "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe",
    ))?
    .Show(&notification)?;
    Ok(())
}
