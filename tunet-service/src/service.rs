mod net_watcher;
mod notify;

use crate::SERVICE_NAME;
use clap::Parser;
use flexi_logger::{LogSpecification, Logger};
use std::{ffi::OsString, path::PathBuf, pin::pin, sync::Arc, time::Duration};
use tokio::{signal::windows::ctrl_c, sync::watch};
use tokio_stream::{wrappers::WatchStream, StreamExt};
use tunet_helper::{create_http_client, Result, TUNetConnect, TUNetHelper};
use tunet_settings::FileSettingsReader;
use tunet_suggest::TUNetHelperExt;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

pub fn start() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_entry)?;
    Ok(())
}

define_windows_service!(ffi_service_entry, service_entry);

fn service_entry(args: Vec<OsString>) {
    if let Err(e) = service_entry_impl(args) {
        log::error!("{}", e);
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

    let next_status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    };

    status_handle.set_service_status(next_status)?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(service_main(args, rx))
}

#[derive(Debug, Parser)]
struct Options {
    #[clap(long)]
    config: PathBuf,
}

async fn service_main(args: Vec<OsString>, rx: watch::Receiver<()>) -> Result<()> {
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
