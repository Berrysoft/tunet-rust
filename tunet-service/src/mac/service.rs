use std::{ffi::CString, pin::pin};
use system_configuration::network_reachability::SCNetworkReachability;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::watch,
};
use tokio_stream::{wrappers::WatchStream, StreamExt};
use tunet_helper::{anyhow, Result};

pub fn register(_interval: Option<humantime::Duration>) -> Result<()> {
    Err(anyhow!("不支持的命令"))
}

pub fn unregister() -> Result<()> {
    Err(anyhow!("不支持的命令"))
}

pub fn start() -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(start_impl())
}

async fn start_impl() -> Result<()> {
    let mut ctrlc = signal(SignalKind::interrupt())?;
    let mut kill = signal(SignalKind::terminate())?;
    let (tx, rx) = watch::channel(());
    let mut sc = SCNetworkReachability::from_host(&CString::new("127.0.0.1")?)
        .ok_or_else(|| anyhow!("Cannot get network reachability"))?;
    sc.set_callback(move |_| {
        println!("Changed.");
        tx.send(()).ok();
    })?;
    let events = WatchStream::new(rx);
    let mut events = pin!(events);
    loop {
        tokio::select! {
            _ = ctrlc.recv() => {
                break;
            }
            _ = kill.recv() => {
                break;
            }
            e = events.next() => {
                if let Some(()) = e {
                    if let Err(msg) = crate::run_once(false).await {
                        eprintln!("{}", msg)
                    }
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}
