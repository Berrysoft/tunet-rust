mod net_watcher;
mod notify;

use clap::Parser;
use flexi_logger::{LogSpecification, Logger};
use std::{ffi::OsString, path::PathBuf, pin::pin, sync::Arc};
use tokio::{signal::windows::ctrl_c, sync::watch};
use tokio_stream::{wrappers::WatchStream, StreamExt};
use tunet_helper::{create_http_client, Result, TUNetConnect, TUNetHelper};
use tunet_settings::FileSettingsReader;
use tunet_suggest::TUNetHelperExt;

#[derive(Debug, Parser)]
struct Options {
    #[clap(long)]
    config: PathBuf,
}

pub async fn service_main(args: Vec<OsString>, rx: watch::Receiver<()>) -> Result<()> {
    let opt = Options::parse_from(args);
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
    let mut ctrlc = ctrl_c()?;
    let mut stopc = WatchStream::new(rx);
    let events = net_watcher::watch()?;
    let mut events = pin!(events);
    loop {
        tokio::select! {
            _ = ctrlc.recv() => {
                log::info!("Ctrl-C received");
                break;
            }
            _ = stopc.next() => {
                log::info!("SCM stop received");
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
    notify::succeeded(flux)?;
    Ok(())
}
