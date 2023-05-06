use anyhow::{anyhow, Result};
use serde::Serialize;
use std::path::PathBuf;
use tokio::signal::unix::{signal, SignalKind};
use tokio_stream::StreamExt;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Launchd {
    label: String,
    program_arguments: Vec<String>,
    run_at_load: bool,
}

fn launchd_path() -> Result<PathBuf> {
    let mut p = dirs::home_dir().ok_or_else(|| anyhow!("Cannot get home dir"))?;
    p.push("Library");
    p.push("LaunchAgents");
    p.push("tunet-service.plist");
    Ok(p)
}

pub fn register(interval: Option<humantime::Duration>) -> Result<()> {
    unregister()?;
    let mut args = vec![
        std::env::current_exe()?.to_string_lossy().into_owned(),
        "start".to_string(),
    ];
    if let Some(d) = interval {
        args.push("--interval".to_string());
        args.push(d.to_string());
    }
    let launchd = Launchd {
        label: crate::SERVICE_NAME.to_string(),
        program_arguments: args,
        run_at_load: true,
    };
    let launchd_path = launchd_path()?;
    plist::to_file_xml(&launchd_path, &launchd)?;
    println!("Wrote launchd to {}", launchd_path.display());
    std::process::Command::new("launchctl")
        .arg("load")
        .arg(&launchd_path)
        .status()?;
    Ok(())
}

pub fn unregister() -> Result<()> {
    std::process::Command::new("launchctl")
        .arg("stop")
        .arg(crate::SERVICE_NAME)
        .status()?;
    let launchd_path = launchd_path()?;
    if launchd_path.exists() {
        std::process::Command::new("launchctl")
            .arg("unload")
            .arg(&launchd_path)
            .status()?;
        std::fs::remove_file(launchd_path)?;
    }
    Ok(())
}

pub fn start(interval: Option<humantime::Duration>) -> Result<()> {
    env_logger::try_init()?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(start_impl(interval))
}

async fn start_impl(interval: Option<humantime::Duration>) -> Result<()> {
    let mut ctrlc = signal(SignalKind::interrupt())?;
    let mut kill = signal(SignalKind::terminate())?;
    let mut timer = crate::create_timer(interval);
    let mut events = netstatus::NetStatus::watch();
    loop {
        tokio::select! {
            _ = ctrlc.recv() => {
                break;
            }
            _ = kill.recv() => {
                break;
            }
            _ = timer.next() => {
                log::info!("Timer triggered.");
                if let Err(msg) = crate::run_once(true).await {
                    log::error!("{}", msg);
                }
            }
            e = events.next() => {
                log::info!("Net status changed.");
                if let Some(()) = e {
                    if let Err(msg) = crate::run_once(false).await {
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
