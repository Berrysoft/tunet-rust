cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[path = "windows/mod.rs"]
        mod platform;
    } else if #[cfg(target_os = "macos")] {
        #[path = "mac/mod.rs"]
        mod platform;
    } else {
        #[path ="stub/mod.rs"]
        mod platform;
    }
}

use platform::*;

use clap::Parser;
use enum_dispatch::enum_dispatch;
use std::sync::Arc;
use tunet_helper::{create_http_client, Result, TUNetConnect, TUNetHelper};
use tunet_settings::FileSettingsReader;
use tunet_settings_cli::{read_cred, save_cred};
use tunet_suggest::TUNetHelperExt;

pub const SERVICE_NAME: &str = "tunet-service";

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
        elevator::elevate()?;
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(save_cred(read_cred()?))?;
        service::register(self.interval)?;
        println!("服务注册成功");
        Ok(())
    }
}

#[derive(Debug, Parser)]
struct Unregister;

impl Command for Unregister {
    fn run(&self) -> Result<()> {
        elevator::elevate()?;
        service::unregister()?;
        println!("服务注销成功");
        Ok(())
    }
}

#[derive(Debug, Parser)]
struct Start {
    #[clap(short, long, help = "Ignored on Windows.")]
    interval: Option<humantime::Duration>,
}

impl Command for Start {
    fn run(&self) -> Result<()> {
        service::start(self.interval)
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
            .block_on(run_once(self.quiet))
    }
}

pub async fn run_once(quiet: bool) -> Result<()> {
    let cred = Arc::new(FileSettingsReader::new()?.read_with_password()?);
    let client = create_http_client()?;
    let c = TUNetConnect::new_with_suggest(None, cred, client).await?;
    c.login().await?;
    let flux = c.flux().await?;
    if !quiet {
        notification::succeeded(flux)?;
    }
    Ok(())
}
