mod platform_helper;

use clap::Parser;
use flexi_logger::{LogSpecification, Logger};
use std::{path::PathBuf, pin::pin, sync::Arc};
use tokio::signal::ctrl_c;
use tokio_stream::StreamExt;
use tunet_helper::{create_http_client, Result, TUNetConnect, TUNetHelper};
use tunet_settings::FileSettingsReader;
use tunet_suggest::TUNetHelperExt;

#[derive(Debug, Parser)]
struct Options {
    #[clap(long)]
    config: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opt = Options::parse();
    let spec = LogSpecification::env_or_parse("debug")?;
    let _log_handle = Logger::with(spec)
        .log_to_stdout()
        .set_palette("b1;3;2;4;6".to_string())
        .use_utc()
        .start()?;
    let settings = FileSettingsReader::with_path(opt.config)?;
    let cred = Arc::new(settings.read()?);
    let client = create_http_client()?;
    let c = TUNetConnect::new_with_suggest(None, cred, client).await?;
    let events = platform_helper::watch()?;
    let mut events = pin!(events);
    loop {
        tokio::select! {
            e = ctrl_c() => {
                match e {
                    Ok(()) => log::info!("Ctrl-C received"),
                    Err(e) => log::error!("{}", e),
                }
                break;
            }
            e = events.next() => {
                if let Some(()) = e {
                    if let Err(msg) = login_and_flux(&c).await {
                        log::error!("{}", msg);
                    }
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}

async fn login_and_flux(c: &TUNetConnect) -> Result<()> {
    let res = c.login().await?;
    log::info!("{}", res);
    let flux = c.flux().await?;
    platform_helper::succeeded(flux)?;
    Ok(())
}
