use mac_notification_sys::*;
use tunet_helper::{NetFlux, Result};

pub fn succeeded(flux: NetFlux) -> Result<()> {
    Notification::new()
        .title(&format!("登录成功：{}", flux.username))
        .message(&format!("流量：{}\n余额：{}", flux.flux, flux.balance))
        .send()?;
    Ok(())
}
