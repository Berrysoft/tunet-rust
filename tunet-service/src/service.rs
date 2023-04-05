mod net_watcher;
mod notify;

use crate::SERVICE_NAME;
use clap::Parser;
use std::{
    ffi::OsString,
    pin::{pin, Pin},
    time::Duration,
};
use tokio::{
    signal::windows::ctrl_c,
    sync::watch,
    time::{interval, Instant},
};
use tokio_stream::{
    pending,
    wrappers::{IntervalStream, WatchStream},
    Stream, StreamExt,
};
use tunet_helper::Result;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

#[derive(Debug, Parser)]
struct ServiceOptions {
    #[clap(short, long)]
    interval: Option<humantime::Duration>,
}

pub fn start() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_entry)?;
    Ok(())
}

define_windows_service!(ffi_service_entry, service_entry);

fn service_entry(args: Vec<OsString>) {
    if let Err(e) = service_entry_impl(args) {
        notify::error(e.to_string()).ok();
    }
}

fn service_entry_impl(args: Vec<OsString>) -> Result<()> {
    let (tx, rx) = watch::channel(());

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                tx.send(()).ok();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(service_main(args, rx))?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

async fn service_main(args: Vec<OsString>, rx: watch::Receiver<()>) -> Result<()> {
    let options = ServiceOptions::try_parse_from(args)?;
    let mut ctrlc = ctrl_c()?;
    let mut stopc = WatchStream::new(rx).skip(1);
    let mut timer = if let Some(d) = options.interval {
        Box::pin(IntervalStream::new(interval(*d))) as Pin<Box<dyn Stream<Item = Instant>>>
    } else {
        Box::pin(pending())
    };
    let events = net_watcher::watch()?;
    let mut events = pin!(events);
    loop {
        tokio::select! {
            _ = ctrlc.recv() => {
                break;
            }
            _ = stopc.next() => {
                break;
            }
            _ = timer.next() => {
                notify::notify(true).ok();
            }
            e = events.next() => {
                if let Some(()) = e {
                    if let Err(msg) = notify::notify(false) {
                        notify::error(msg.to_string()).ok();
                    }
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}
