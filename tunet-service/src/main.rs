mod service;

use clap::Parser;
use enum_dispatch::enum_dispatch;
use tunet_helper::Result;
use windows_service::{
    service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType},
    service_manager::{ServiceManager, ServiceManagerAccess},
};

const SERVICE_NAME: &str = "tunet-service";

fn main() -> Result<()> {
    let commands = Commands::parse();
    commands.run()
}

#[enum_dispatch(Commands)]
trait Command {
    fn run(&self) -> Result<()>;
}

#[enum_dispatch]
#[derive(Debug, Parser)]
#[clap(about, version, author)]
enum Commands {
    Register,
    Unregister,
    Start,
}

#[derive(Debug, Parser)]
struct Register;

impl Command for Register {
    fn run(&self) -> Result<()> {
        let manager =
            ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
        let service_info = ServiceInfo {
            name: SERVICE_NAME.into(),
            display_name: "TsinghuaNet Background Task".into(),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: std::env::current_exe()?,
            launch_arguments: vec![],
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };
        manager.create_service(&service_info, ServiceAccess::QUERY_STATUS)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
struct Unregister;

impl Command for Unregister {
    fn run(&self) -> Result<()> {
        let manager =
            ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
        manager
            .open_service(SERVICE_NAME, ServiceAccess::DELETE)?
            .delete()?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
struct Start;

impl Command for Start {
    fn run(&self) -> Result<()> {
        service::start()
    }
}
