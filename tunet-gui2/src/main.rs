use std::sync::Arc;
use tunet_helper::{anyhow, create_http_client, Result, TUNetConnect, TUNetHelper};
use tunet_settings::FileSettingsReader;
use tunet_suggest::TUNetHelperExt;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::new()?;

    let color = color_theme::Color::accent();
    let home_model = app.global::<HomeModel>();
    home_model.set_theme_color(slint::Color::from_argb_u8(255, color.r, color.g, color.b));
    home_model.set_theme_color_t1(slint::Color::from_argb_u8(191, color.r, color.g, color.b));
    home_model.set_theme_color_t2(slint::Color::from_argb_u8(140, color.r, color.g, color.b));

    app.global::<AboutModel>()
        .set_version(env!("CARGO_PKG_VERSION").into());

    let weak_app = app.as_weak();
    tokio::spawn(async move {
        let cred = Arc::new(FileSettingsReader::new()?.read()?);
        let client = create_http_client()?;
        let c = TUNetConnect::new_with_suggest(None, cred, client).await?;
        let flux = c.flux().await?;
        weak_app
            .upgrade_in_event_loop(move |app| {
                app.global::<HomeModel>().set_info(NetInfo {
                    username: flux.username.into(),
                    flux: flux.flux.to_string().into(),
                    online_time: flux.online_time.to_string().into(),
                    balance: flux.balance.to_string().into(),
                });
            })
            .map_err(|e| anyhow!("{:?}", e))?;
        Ok(()) as Result<()>
    });

    app.run()?;
    Ok(())
}
