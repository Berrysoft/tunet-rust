use core_foundation::{
    base::TCFType,
    runloop::{kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopRef},
};
use serde::Serialize;
use std::{
    ffi::CStr,
    os::unix::thread::{JoinHandleExt, RawPthread},
    path::PathBuf,
    pin::pin,
};
use system_configuration::network_reachability::SCNetworkReachability;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::watch,
};
use tokio_stream::StreamExt;
use tunet_helper::{anyhow, Result};

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
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(start_impl(interval))
}

async fn start_impl(interval: Option<humantime::Duration>) -> Result<()> {
    let mut ctrlc = signal(SignalKind::interrupt())?;
    let mut kill = signal(SignalKind::terminate())?;
    let mut timer = crate::create_timer(interval);
    let (tx, rx) = watch::channel(());
    let loop_thread = std::thread::spawn(move || -> Result<()> {
        let host = unsafe { CStr::from_bytes_with_nul_unchecked(b"0.0.0.0\0") };
        let mut sc = SCNetworkReachability::from_host(host)
            .ok_or_else(|| anyhow!("Cannot get network reachability"))?;
        sc.set_callback(move |_| {
            tx.send(()).ok();
        })?;
        unsafe {
            sc.schedule_with_runloop(&CFRunLoop::get_current(), kCFRunLoopDefaultMode)?;
        }
        CFRunLoop::run_current();
        Ok(())
    });
    let mut events = pin!(rx);
    loop {
        tokio::select! {
            _ = ctrlc.recv() => {
                break;
            }
            _ = kill.recv() => {
                break;
            }
            _ = timer.next() => {
                if let Err(msg) = crate::run_once(true).await {
                    eprintln!("{}", msg)
                }
            }
            e = events.changed() => {
                if let Ok(()) = e {
                    if let Err(msg) = crate::run_once(false).await {
                        eprintln!("{}", msg)
                    }
                } else {
                    break;
                }
            }
        }
    }
    unsafe { CFRunLoop::wrap_under_get_rule(_CFRunLoopGet0(loop_thread.as_pthread_t())) }.stop();
    match loop_thread.join() {
        Ok(res) => res,
        Err(e) => std::panic::resume_unwind(e),
    }
}

extern "C" {
    fn _CFRunLoopGet0(thread: RawPthread) -> CFRunLoopRef;
}
