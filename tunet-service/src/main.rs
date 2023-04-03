mod net_watcher;

use clap::Parser;
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
    let settings = FileSettingsReader::with_path(opt.config)?;
    let cred = Arc::new(settings.read()?);
    let client = create_http_client()?;
    let c = TUNetConnect::new_with_suggest(None, cred, client).await?;
    let events = net_watcher::watch()?;
    let mut events = pin!(events);
    loop {
        tokio::select! {
            _ = ctrl_c() => {
                break;
            }
            e = events.next() => {
                if let Some(()) = e {
                    match c.login().await {
                        Ok(res) => println!("{}", res),
                        Err(msg) => eprintln!("{}", msg),
                    }
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}
