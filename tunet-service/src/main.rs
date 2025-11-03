cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[path = "windows/mod.rs"]
        mod service;
    } else if #[cfg(target_os = "macos")] {
        #[path = "mac/mod.rs"]
        mod service;
    } else {
        #[path ="stub/mod.rs"]
        mod service;
    }
}

mod elevator;
mod notification;

use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use enum_dispatch::enum_dispatch;
use futures_util::{future::Either, Stream};
use tunet_helper::{create_http_client, TUNetConnect, TUNetHelper};
use tunet_settings::SettingsReader;

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
        let mut reader = SettingsReader::new()?;
        let (u, p) = reader.read_ask_full()?;
        reader.save(&u, &p)?;
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
        compio::runtime::Runtime::new()?.block_on(run_once(self.quiet))
    }
}

pub async fn run_once(quiet: bool) -> Result<()> {
    match SettingsReader::new()?.read_full() {
        Ok((u, p)) => {
            let client = create_http_client().await?;
            let c = TUNetConnect::new_with_suggest(None, client).await?;
            c.login(&u, &p).await?;
            let flux = c.flux().await?;
            if !quiet {
                notification::succeeded(flux)?;
            }
        }
        Err(e) => {
            log::error!("{e}");
        }
    }
    Ok(())
}

pub fn create_timer(interval: Option<humantime::Duration>) -> impl Stream<Item = Instant> {
    if let Some(d) = interval {
        Either::Left(async_stream::stream! {
            let mut interval = compio::time::interval(*d);
            loop {
                yield interval.tick().await;
            }
        })
    } else {
        Either::Right(futures_util::stream::pending())
    }
}
