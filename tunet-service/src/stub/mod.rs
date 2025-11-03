use anyhow::{anyhow, Result};
use compio::signal::ctrl_c;
use futures_util::{FutureExt, StreamExt};

pub fn register(_interval: Option<humantime::Duration>) -> Result<()> {
    Err(anyhow!("不支持的命令"))
}

pub fn unregister() -> Result<()> {
    Err(anyhow!("不支持的命令"))
}

pub fn start(interval: Option<humantime::Duration>) -> Result<()> {
    env_logger::try_init()?;
    compio::runtime::Runtime::new()?.block_on(start_impl(interval))
}

async fn start_impl(interval: Option<humantime::Duration>) -> Result<()> {
    let timer = crate::create_timer(interval).fuse();
    let mut timer = std::pin::pin!(timer);
    loop {
        let ctrlc = ctrl_c();
        let ctrlc = std::pin::pin!(ctrlc);
        futures_util::select! {
            _ = ctrlc.fuse() => {
                break;
            }
            _ = timer.next() => {
                log::info!("Timer triggered.");
                if let Err(msg) = crate::run_once(true).await {
                    log::error!("{msg}");
                }
            }
        }
    }
    Ok(())
}
