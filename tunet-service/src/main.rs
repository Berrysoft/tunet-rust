mod service;

use std::path::PathBuf;

use clap::Parser;
use enum_dispatch::enum_dispatch;
use once_cell::sync::OnceCell;
use tunet_helper::Result;
use tunet_settings::FileSettingsReader;
use tunet_settings_cli::{read_cred, save_cred};
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
        elevate()?;
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(save_cred(read_cred()?))?;
        let settings_path = FileSettingsReader::file_path()?;
        let manager =
            ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
        let service_info = ServiceInfo {
            name: SERVICE_NAME.into(),
            display_name: "TsinghuaNet Background Task".into(),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: std::env::current_exe()?,
            launch_arguments: vec!["start".into(), "--config".into(), settings_path.into()],
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };
        manager.create_service(&service_info, ServiceAccess::QUERY_STATUS)?;
        println!("Register successfully.");
        Ok(())
    }
}

#[derive(Debug, Parser)]
struct Unregister;

impl Command for Unregister {
    fn run(&self) -> Result<()> {
        elevate()?;
        let manager =
            ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
        manager
            .open_service(SERVICE_NAME, ServiceAccess::DELETE)?
            .delete()?;
        println!("Unregister successfully.");
        Ok(())
    }
}

static CONFIG_PATH: OnceCell<PathBuf> = OnceCell::new();

#[derive(Debug, Parser)]
struct Start {
    #[clap(long)]
    config: PathBuf,
}

impl Command for Start {
    fn run(&self) -> Result<()> {
        CONFIG_PATH.set(self.config.clone()).unwrap();
        service::start()
    }
}

fn elevate() -> Result<()> {
    if !is_elevated::is_elevated() {
        let status = std::process::Command::new("powershell.exe")
            .arg("-c")
            .arg("Start-Process")
            .arg(std::env::current_exe()?)
            .arg("-Verb")
            .arg("runas")
            .arg("-ArgumentList")
            .arg(
                std::env::args()
                    .skip(1)
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(","),
            )
            .arg("-Wait")
            .status()?;
        std::process::exit(status.code().unwrap_or_default());
    } else {
        Ok(())
    }
}
