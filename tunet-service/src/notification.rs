use notify_rust::Notification;
use tunet_helper::{NetFlux, Result};

pub fn succeeded(flux: NetFlux) -> Result<()> {
    Notification::new()
        .summary(&format!("登录成功：{}", flux.username))
        .body(&format!("流量：{}\n余额：{}", flux.flux, flux.balance))
        .show()?;
    Ok(())
}
