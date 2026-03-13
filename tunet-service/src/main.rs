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
use argh::FromArgs;
use enum_dispatch::enum_dispatch;
use futures_util::{Stream, future::Either};
use tunet_helper::{TUNetConnect, create_http_client};
use tunet_settings::SettingsReader;

pub const SERVICE_NAME: &str = "tunet-service";

fn main() -> Result<()> {
    let commands: Commands = argh::from_env();
    commands.run()
}

#[enum_dispatch(CommandsImpl)]
trait Command {
    fn run(&self) -> Result<()>;
}

#[derive(Debug, FromArgs)]
#[argh(description = "清华校园网后台服务")]
struct Commands {
    #[argh(subcommand)]
    cmd: CommandsImpl,
}

impl Command for Commands {
    fn run(&self) -> Result<()> {
        self.cmd.run()
    }
}

#[enum_dispatch]
#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum CommandsImpl {
    Register,
    Unregister,
    Start,
    RunOnce,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "register")]
/// 注册服务
struct Register {
    #[argh(option, short = 'i')]
    /// 定时登录的时间间隔，默认为网络状态改变时登录
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

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "unregister")]
/// 注销服务
struct Unregister {}

impl Command for Unregister {
    fn run(&self) -> Result<()> {
        elevator::elevate()?;
        service::unregister()?;
        println!("服务注销成功");
        Ok(())
    }
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "start")]
/// 启动服务
struct Start {
    #[argh(option, short = 'i')]
    /// 定时登录的时间间隔，默认为网络状态改变时登录。在 Windows 上此选项无效。
    interval: Option<humantime::Duration>,
}

impl Command for Start {
    fn run(&self) -> Result<()> {
        service::start(self.interval)
    }
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "run-once")]
/// 运行一次
struct RunOnce {
    #[argh(switch, short = 'q')]
    /// 不显示系统通知
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
