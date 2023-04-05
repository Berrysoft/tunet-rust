mod service;
mod toast;

use clap::Parser;
use enum_dispatch::enum_dispatch;
use std::sync::Arc;
use tunet_helper::{create_http_client, Result, TUNetConnect, TUNetHelper};
use tunet_settings::FileSettingsReader;
use tunet_settings_cli::{read_cred, save_cred};
use tunet_suggest::TUNetHelperExt;
use windows_service::{
    service::{
        ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceState,
        ServiceType,
    },
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
    RunOnce,
}

#[derive(Debug, Parser)]
struct Register {
    #[clap(short, long)]
    interval: Option<humantime::Duration>,
}

impl Command for Register {
    fn run(&self) -> Result<()> {
        elevate()?;
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(save_cred(read_cred()?))?;
        let manager =
            ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
        let service = if let Ok(service) = manager.open_service(
            SERVICE_NAME,
            ServiceAccess::QUERY_STATUS | ServiceAccess::START | ServiceAccess::STOP,
        ) {
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
            service
        } else {
            let service_info = ServiceInfo {
                name: SERVICE_NAME.into(),
                display_name: "TsinghuaNet Background Task".into(),
                service_type: ServiceType::OWN_PROCESS,
                start_type: ServiceStartType::AutoStart,
                error_control: ServiceErrorControl::Normal,
                executable_path: std::env::current_exe()?,
                launch_arguments: vec!["start".into()],
                dependencies: vec![],
                account_name: None,
                account_password: None,
            };
            manager.create_service(
                &service_info,
                ServiceAccess::QUERY_STATUS | ServiceAccess::START,
            )?
        };
        let service_args = if let Some(d) = self.interval {
            vec!["--interval".to_string(), d.to_string()]
        } else {
            vec![]
        };
        service.start(&service_args)?;
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
        let service = manager.open_service(
            SERVICE_NAME,
            ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE,
        )?;
        let status = service.query_status()?;
        if status.current_state != ServiceState::Stopped {
            service.stop()?;
        }
        service.delete()?;
        println!("Unregister successfully.");
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

#[derive(Debug, Parser)]
struct RunOnce {
    #[clap(short, long, default_value = "false")]
    quiet: bool,
}

impl Command for RunOnce {
    fn run(&self) -> Result<()> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(async {
                let cred = Arc::new(FileSettingsReader::new()?.read()?);
                let client = create_http_client()?;
                let c = TUNetConnect::new_with_suggest(None, cred, client).await?;
                c.login().await?;
                let flux = c.flux().await?;
                if !self.quiet {
                    toast::succeeded(flux)?;
                }
                Ok(())
            })
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
