use tokio::signal::ctrl_c;
use tokio_stream::StreamExt;
use tunet_helper::{anyhow, Result};

pub fn register(_interval: Option<humantime::Duration>) -> Result<()> {
    Err(anyhow!("不支持的命令"))
}

pub fn unregister() -> Result<()> {
    Err(anyhow!("不支持的命令"))
}

pub fn start(interval: Option<humantime::Duration>) -> Result<()> {
    env_logger::try_init()?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(start_impl(interval))
}

async fn start_impl(interval: Option<humantime::Duration>) -> Result<()> {
    let mut timer = crate::create_timer(interval);
    loop {
        tokio::select! {
            _ = ctrl_c() => {
                break;
            }
            _ = timer.next() => {
                log::info!("Timer triggered.");
                if let Err(msg) = crate::run_once(true).await {
                    log::error!("{}", msg);
                }
            }
        }
    }
    Ok(())
}
