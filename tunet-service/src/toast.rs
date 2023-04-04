use tunet_helper::NetFlux;
use windows::{
    core::*,
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::{ToastNotification, ToastNotificationManager},
};

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
