mod service;

use std::{ffi::OsString, time::Duration};
use tokio::sync::watch;
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

const SERVICE_NAME: &str = "tunet-service";

fn main() -> Result<()> {
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
        service_type: ServiceType::USER_OWN_PROCESS,
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
        .block_on(service::service_main(args, rx))
}
