use anyhow::Result;
use argh::FromArgs;
use console::{style, Color};
use enum_dispatch::enum_dispatch;
use std::io::{stdin, stdout, Write};
use tunet_helper::*;
use tunet_settings::*;

fn get_flux_color(&Flux(flux): &Flux, total: bool) -> Color {
    if flux == 0 {
        Color::Cyan
    } else if flux < if total { 20_000_000_000 } else { 2_000_000_000 } {
        Color::Yellow
    } else {
        Color::Magenta
    }
}

#[enum_dispatch(TUNetImpl)]
pub trait TUNetCommand {
    async fn run(&self) -> Result<()>;
}

#[derive(Debug, FromArgs)]
#[argh(description = "清华大学校园网客户端")]
pub struct TUNet {
    #[argh(subcommand)]
    cmd: TUNetImpl,
}

impl TUNetCommand for TUNet {
    async fn run(&self) -> Result<()> {
        self.cmd.run().await
    }
}

#[enum_dispatch]
#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum TUNetImpl {
    Login,
    Logout,
    Status,
    DeleteCred,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "login")]
/// 登录
pub struct Login {
    #[argh(option, short = 's')]
    /// 连接方式
    host: Option<NetState>,
}

impl TUNetCommand for Login {
    async fn run(&self) -> Result<()> {
        let client = create_http_client().await?;
        let mut reader = SettingsReader::new()?;
        let (u, p) = reader.read_ask_full()?;
        let c = TUNetConnect::new_with_suggest(self.host, client).await?;
        let res = c.login(&u, &p).await?;
        println!("{res}");
        reader.save(&u, &p)?;
        Ok(())
    }
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "logout")]
/// 注销
pub struct Logout {
    #[argh(option, short = 's')]
    /// 连接方式
    host: Option<NetState>,
}

impl TUNetCommand for Logout {
    async fn run(&self) -> Result<()> {
        let client = create_http_client().await?;
        let reader = SettingsReader::new()?;
        let u = reader.read_ask_username()?;
        let c = TUNetConnect::new_with_suggest(self.host, client).await?;
        let res = c.logout(&u).await?;
        println!("{res}");
        Ok(())
    }
}
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "status")]
/// 查看在线状态
pub struct Status {
    #[argh(option, short = 's')]
    /// 连接方式
    host: Option<NetState>,
    /// 输出 NUON
    #[argh(switch)]
    nuon: bool,
}

impl TUNetCommand for Status {
    async fn run(&self) -> Result<()> {
        let client = create_http_client().await?;
        let c = TUNetConnect::new_with_suggest(self.host, client).await?;
        let f = c.flux().await?;
        if self.nuon {
            println!(
                "{{username:{},flux:{}b,online_time:{}ms,balance:{}}}",
                f.username,
                f.flux.0,
                f.online_time.0.num_milliseconds(),
                f.balance.0
            );
        } else {
            println!("{} {}", style("用户").cyan(), f.username);
            println!(
                "{} {}",
                style("流量").cyan(),
                style(f.flux).bold().fg(get_flux_color(&f.flux, true))
            );
            println!("{} {}", style("时长").cyan(), style(f.online_time).green());
            println!("{} {}", style("余额").cyan(), style(f.balance).yellow());
        }
        Ok(())
    }
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "deletecred")]
/// 删除用户名和密码
pub struct DeleteCred {}

impl TUNetCommand for DeleteCred {
    async fn run(&self) -> Result<()> {
        let mut reader = SettingsReader::new()?;
        let u = reader.read_username()?;
        print!("是否删除设置文件？[y/N]");
        stdout().flush()?;
        let mut s = String::new();
        stdin().read_line(&mut s)?;
        if s.trim().eq_ignore_ascii_case("y") {
            reader.delete(&u)?;
            println!("已删除");
        }
        Ok(())
    }
}
