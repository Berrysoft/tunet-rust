mod notify;

use crate::SERVICE_NAME;
use anyhow::Result;
use std::{ffi::OsString, sync::Mutex, time::Duration};
use tokio::{signal::windows::ctrl_c, sync::watch};
use tokio_stream::{wrappers::WatchStream, StreamExt};
use windows_service::{
    define_windows_service,
    service::{
        Service, ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl,
        ServiceExitCode, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

fn open_manager() -> Result<ServiceManager> {
    Ok(ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CREATE_SERVICE,
    )?)
}

fn open_service(manager: &ServiceManager) -> Result<Service> {
    Ok(manager.open_service(
        SERVICE_NAME,
        ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE,
    )?)
}

fn create_service(
    manager: &ServiceManager,
    interval: Option<humantime::Duration>,
) -> Result<Service> {
    let service_info = ServiceInfo {
        name: SERVICE_NAME.into(),
        display_name: "TsinghuaNet Background Task".into(),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: std::env::current_exe()?,
        launch_arguments: if let Some(d) = interval {
            vec!["start".into(), "--interval".into(), d.to_string().into()]
        } else {
            vec!["start".into()]
        },
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };
    Ok(manager.create_service(
        &service_info,
        ServiceAccess::QUERY_STATUS | ServiceAccess::START,
    )?)
}

fn delete_service(service: &Service) -> Result<()> {
    let status = service.query_status()?;
    if status.current_state != ServiceState::Stopped {
        service.stop()?;
    }
    loop {
        let status = service.query_status()?;
        if status.current_state == ServiceState::Stopped {
            break;
        }
    }
    service.delete()?;
    Ok(())
}

pub fn register(interval: Option<humantime::Duration>) -> Result<()> {
    winlog2::register(SERVICE_NAME)?;

    let manager = open_manager()?;

    if let Ok(service) = open_service(&manager) {
        delete_service(&service)?;
    }

    let service = create_service(&manager, interval)?;

    service.start::<&str>(&[])?;
    Ok(())
}

pub fn unregister() -> Result<()> {
    let manager = open_manager()?;
    let service = open_service(&manager)?;
    delete_service(&service)?;

    winlog2::deregister(SERVICE_NAME)?;
    Ok(())
}

static START_INTERVAL: Mutex<Option<humantime::Duration>> = Mutex::new(None);

pub fn start(interval: Option<humantime::Duration>) -> Result<()> {
    *START_INTERVAL.lock().unwrap() = interval;
    service_dispatcher::start(SERVICE_NAME, ffi_service_entry)?;
    Ok(())
}

define_windows_service!(ffi_service_entry, service_entry);

fn service_entry(_args: Vec<OsString>) {
    if let Err(e) = service_entry_impl() {
        notify::error(e.to_string()).ok();
    }
}

fn service_entry_impl() -> Result<()> {
    let interval = START_INTERVAL.lock().unwrap().as_ref().cloned();

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
        .block_on(service_main(interval, rx))?;

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

async fn service_main(
    interval: Option<humantime::Duration>,
    rx: watch::Receiver<()>,
) -> Result<()> {
    winlog2::init(SERVICE_NAME)?;
    let mut ctrlc = ctrl_c()?;
    let mut stopc = WatchStream::new(rx).skip(1);
    let mut timer = crate::create_timer(interval);
    let mut events = netstatus::NetStatus::watch();
    loop {
        tokio::select! {
            _ = ctrlc.recv() => {
                break;
            }
            _ = stopc.next() => {
                break;
            }
            _ = timer.next() => {
                log::info!("Timer triggered.");
                notify::notify(true).ok();
            }
            e = events.next() => {
                log::info!("Net status changed.");
                if let Some(()) = e {
                    if let Err(msg) = notify::notify(false) {
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
