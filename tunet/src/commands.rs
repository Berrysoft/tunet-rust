use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use console::{style, Color};
use enum_dispatch::enum_dispatch;
use futures_util::{pin_mut, stream::TryStreamExt};
use itertools::Itertools;
use mac_address::{MacAddress, MacAddressIterator};
use std::cmp::Reverse;
use std::io::{stdin, stdout, Write};
use std::net::Ipv4Addr;
use tunet_helper::{usereg::*, *};
use tunet_settings::*;

fn get_flux_color(f: &Flux, total: bool) -> Color {
    let flux = f.0;
    if flux == 0 {
        Color::Cyan
    } else if flux < if total { 20_000_000_000 } else { 2_000_000_000 } {
        Color::Yellow
    } else {
        Color::Magenta
    }
}

#[async_trait]
#[enum_dispatch(TUNet)]
pub trait TUNetCommand {
    async fn run(&self) -> Result<()>;
}

#[enum_dispatch]
#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub enum TUNet {
    #[clap(name = "login", about = "登录")]
    Login,
    #[clap(name = "logout", about = "注销")]
    Logout,
    #[clap(name = "status", about = "查看在线状态")]
    Status,
    #[clap(name = "online", about = "查询在线IP")]
    Online,
    #[clap(name = "connect", about = "上线IP")]
    UseregConnect,
    #[clap(name = "drop", about = "下线IP")]
    UseregDrop,
    #[clap(name = "detail", about = "流量明细")]
    Detail,
    #[clap(name = "deletecred", about = "删除用户名和密码")]
    DeleteCred,
}

#[derive(Debug, Parser)]
pub struct Login {
    #[clap(long, short = 's')]
    /// 连接方式
    host: Option<NetState>,
}

#[async_trait]
impl TUNetCommand for Login {
    async fn run(&self) -> Result<()> {
        let client = create_http_client()?;
        let mut reader = SettingsReader::new()?;
        let (u, p) = reader.read_ask_full()?;
        let c = TUNetConnect::new_with_suggest(self.host, client).await?;
        let res = c.login(&u, &p).await?;
        println!("{}", res);
        reader.save(&u, &p)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct Logout {
    #[clap(long, short = 's')]
    /// 连接方式
    host: Option<NetState>,
}

#[async_trait]
impl TUNetCommand for Logout {
    async fn run(&self) -> Result<()> {
        let client = create_http_client()?;
        let reader = SettingsReader::new()?;
        let u = reader.read_ask_username()?;
        let c = TUNetConnect::new_with_suggest(self.host, client).await?;
        let res = c.logout(&u).await?;
        println!("{}", res);
        Ok(())
    }
}
#[derive(Debug, Parser)]
pub struct Status {
    #[clap(long, short = 's')]
    /// 连接方式
    host: Option<NetState>,
    /// 输出 NUON
    #[clap(long, default_value = "false")]
    nuon: bool,
}

#[async_trait]
impl TUNetCommand for Status {
    async fn run(&self) -> Result<()> {
        let client = create_http_client()?;
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

#[derive(Debug, Parser)]
pub struct Online {
    /// 输出 NUON
    #[clap(long, default_value = "false")]
    nuon: bool,
}

fn is_self(mac_addrs: &[MacAddress], u: &NetUser) -> bool {
    mac_addrs
        .iter()
        .any(|it| Some(it) == u.mac_address.as_ref())
}

#[async_trait]
impl TUNetCommand for Online {
    async fn run(&self) -> Result<()> {
        let client = create_http_client()?;
        let mut reader = SettingsReader::new()?;
        let (u, p) = reader.read_ask_full()?;
        let c = UseregHelper::new(client);
        c.login(&u, &p).await?;
        let us = c.users();
        let mac_addrs = MacAddressIterator::new()
            .map(|it| it.collect::<Vec<_>>())
            .unwrap_or_default();
        pin_mut!(us);
        if self.nuon {
            print!("[[address login_time flux mac_address is_self]; ");
            while let Some(u) = us.try_next().await? {
                print!(
                    "[\"{}\" {} {}b \"{}\" {}] ",
                    u.address,
                    naive_rfc3339(u.login_time),
                    u.flux.0,
                    u.mac_address.map(|a| a.to_string()).unwrap_or_default(),
                    is_self(&mac_addrs, &u),
                );
            }
            println!("]");
        } else {
            println!("    IP地址            登录时间         流量        MAC地址");
            while let Some(u) = us.try_next().await? {
                let is_self = is_self(&mac_addrs, &u);
                println!(
                    "{:15} {:20} {:>8} {} {}",
                    style(u.address).yellow(),
                    style(u.login_time).green(),
                    style(u.flux).fg(get_flux_color(&u.flux, true)),
                    style(u.mac_address.map(|a| a.to_string()).unwrap_or_default()).cyan(),
                    style(if is_self { "本机" } else { "" }).magenta()
                );
            }
        }
        reader.save(&u, &p)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct UseregConnect {
    #[clap(long, short)]
    /// IP地址
    address: Ipv4Addr,
}

#[async_trait]
impl TUNetCommand for UseregConnect {
    async fn run(&self) -> Result<()> {
        let client = create_http_client()?;
        let mut reader = SettingsReader::new()?;
        let (u, p) = reader.read_ask_full()?;
        let c = UseregHelper::new(client);
        c.login(&u, &p).await?;
        let res = c.connect(self.address).await?;
        println!("{}", res);
        reader.save(&u, &p)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct UseregDrop {
    #[clap(long, short)]
    /// IP地址
    address: Ipv4Addr,
}

#[async_trait]
impl TUNetCommand for UseregDrop {
    async fn run(&self) -> Result<()> {
        let client = create_http_client()?;
        let mut reader = SettingsReader::new()?;
        let (u, p) = reader.read_ask_full()?;
        let c = UseregHelper::new(client);
        c.login(&u, &p).await?;
        let res = c.drop(self.address).await?;
        println!("{}", res);
        reader.save(&u, &p)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct Detail {
    #[clap(long, short, default_value = "logout")]
    /// 排序方式
    order: NetDetailOrder,
    #[clap(long, short)]
    /// 倒序
    descending: bool,
    #[clap(long, short)]
    /// 按日期分组
    grouping: bool,
    /// 输出 NUON
    #[clap(long, default_value = "false")]
    nuon: bool,
}

impl Detail {
    async fn run_detail(&self) -> Result<()> {
        let client = create_http_client()?;
        let mut reader = SettingsReader::new()?;
        let (u, p) = reader.read_ask_full()?;
        let c = UseregHelper::new(client);
        c.login(&u, &p).await?;
        let details = c.details(self.order, self.descending);
        pin_mut!(details);
        if self.nuon {
            print!("[[login_time logout_time flux]; ");
            while let Some(d) = details.try_next().await? {
                print!(
                    "[{} {} {}b] ",
                    naive_rfc3339(d.login_time),
                    naive_rfc3339(d.logout_time),
                    d.flux.0
                );
            }
            println!("]");
        } else {
            println!("      登录时间             注销时间         流量");
            let mut total_flux = Flux(0);
            while let Some(d) = details.try_next().await? {
                println!(
                    "{:20} {:20} {:>8}",
                    style(d.login_time).green(),
                    style(d.logout_time).green(),
                    style(d.flux).fg(get_flux_color(&d.flux, false))
                );
                total_flux.0 += d.flux.0;
            }
            println!(
                "{} {}",
                style("总流量").cyan(),
                style(total_flux)
                    .bold()
                    .fg(get_flux_color(&total_flux, true))
            );
        }
        reader.save(&u, &p)?;
        Ok(())
    }

    async fn run_detail_grouping(&self) -> Result<()> {
        let client = create_http_client()?;
        let mut reader = SettingsReader::new()?;
        let (u, p) = reader.read_ask_full()?;
        let c = UseregHelper::new(client);
        c.login(&u, &p).await?;
        let details = c
            .details(NetDetailOrder::LogoutTime, self.descending)
            .try_collect::<Vec<_>>()
            .await?;
        let mut details = details
            .into_iter()
            .group_by(|detail| detail.logout_time.date())
            .into_iter()
            .map(|(key, group)| (key, Flux(group.map(|detail| detail.flux.0).sum::<u64>())))
            .collect::<Vec<_>>();
        match self.order {
            NetDetailOrder::Flux => {
                if self.descending {
                    details.sort_unstable_by_key(|(_, flux)| Reverse(*flux));
                } else {
                    details.sort_unstable_by_key(|(_, flux)| *flux);
                }
            }
            _ => {
                if self.descending {
                    details.sort_unstable_by_key(|(date, _)| Reverse(date.day()));
                }
            }
        }
        if self.nuon {
            print!("[[login_time flux]; ");
            for (date, flux) in details {
                print!("[{} {}b] ", date, flux.0);
            }
            println!("]");
        } else {
            println!(" 登录日期    流量");
            let mut total_flux = Flux(0);
            for (date, flux) in details {
                println!(
                    "{:10} {:>8}",
                    style(date).green(),
                    style(flux).fg(get_flux_color(&flux, true))
                );
                total_flux.0 += flux.0;
            }
            println!(
                "{} {}",
                style("总流量").cyan(),
                style(total_flux)
                    .bold()
                    .fg(get_flux_color(&total_flux, true))
            );
        }
        reader.save(&u, &p)?;
        Ok(())
    }
}

#[async_trait]
impl TUNetCommand for Detail {
    async fn run(&self) -> Result<()> {
        if self.grouping {
            self.run_detail_grouping().await
        } else {
            self.run_detail().await
        }
    }
}

#[derive(Debug, Parser)]
pub struct DeleteCred {}

#[async_trait]
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

fn naive_rfc3339(datetime: NetDateTime) -> String {
    DateTime::<FixedOffset>::from_local(datetime.0, FixedOffset::east_opt(8 * 3600).unwrap())
        .to_rfc3339()
}
