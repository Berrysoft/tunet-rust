#![forbid(unsafe_code)]

use itertools::Itertools;
use mac_address::MacAddressIterator;
use std::cmp::Reverse;
use std::net::Ipv4Addr;
use std::ops::Deref;
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, StandardStream};
use termcolor_output as tco;
use tunet_rust::{usereg::*, *};

mod settings;
mod strfmt;

use settings::*;
use strfmt::*;

trait TUNetCommand {
    fn run(&self) -> Result<()>;
}

#[derive(Debug, StructOpt)]
#[structopt(name = "TsinghuaNetRust", about = "清华大学校园网客户端")]
enum TUNet {
    #[structopt(name = "login")]
    /// 登录
    Login(TUNetLogin),
    #[structopt(name = "logout")]
    /// 注销
    Logout(TUNetLogout),
    #[structopt(name = "status")]
    /// 查看在线状态
    Status(TUNetStatus),
    #[structopt(name = "online")]
    /// 查询在线IP
    Online(TUNetOnline),
    #[structopt(name = "connect")]
    /// 上线IP
    Connect(TUNetUseregConnect),
    #[structopt(name = "drop")]
    /// 下线IP
    Drop(TUNetUseregDrop),
    #[structopt(name = "detail")]
    /// 流量明细
    Detail(TUNetDetail),
    #[structopt(name = "deletecred")]
    /// 删除用户名和密码
    DeleteCredential(TUNetDeleteCred),
}

impl Deref for TUNet {
    type Target = dyn TUNetCommand;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Login(c) => c,
            Self::Logout(c) => c,
            Self::Status(c) => c,
            Self::Online(c) => c,
            Self::Connect(c) => c,
            Self::Drop(c) => c,
            Self::Detail(c) => c,
            Self::DeleteCredential(c) => c,
        }
    }
}

fn main() -> Result<()> {
    let opt = TUNet::from_args();
    opt.run()
}

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

#[derive(Debug, StructOpt)]
struct TUNetLogin {
    #[structopt(long, short = "s", default_value = "auto")]
    /// 连接方式
    host: NetState,
}

impl TUNetCommand for TUNetLogin {
    fn run(&self) -> Result<()> {
        let client = create_http_client();
        let cred = read_cred()?;
        let mut c = TUNetConnect::new(self.host, cred, &client)?;
        let res = c.login()?;
        println!("{}", res);
        save_cred(c.cred())
    }
}

#[derive(Debug, StructOpt)]
struct TUNetLogout {
    #[structopt(long, short = "s", default_value = "auto")]
    /// 连接方式
    host: NetState,
}

impl TUNetCommand for TUNetLogout {
    fn run(&self) -> Result<()> {
        let client = create_http_client();
        let cred = read_username()?;
        let mut c = TUNetConnect::new(self.host, cred, &client)?;
        let res = c.logout()?;
        println!("{}", res);
        Ok(())
    }
}
#[derive(Debug, StructOpt)]
struct TUNetStatus {
    #[structopt(long, short = "s", default_value = "auto")]
    /// 连接方式
    host: NetState,
}

impl TUNetCommand for TUNetStatus {
    fn run(&self) -> Result<()> {
        let client = create_http_client();
        let c = TUNetConnect::new(self.host, NetCredential::default(), &client)?;
        let f = c.flux()?;
        let stdout = StandardStream::stdout(ColorChoice::Auto);
        let mut stdout = tco::ResetGuard::Owned(stdout);
        tco::writeln!(
            stdout,
            "{}用户 {}{}",
            fg!(Some(Color::Cyan)),
            reset!(),
            f.username
        )?;
        tco::writeln!(
            stdout,
            "{}流量 {}{}{}",
            fg!(Some(Color::Cyan)),
            fg!(Some(get_flux_color(&f.flux, true))),
            bold!(true),
            f.flux
        )?;
        tco::writeln!(
            stdout,
            "{}时长 {}{}",
            fg!(Some(Color::Cyan)),
            fg!(Some(Color::Green)),
            FmtDuration(f.online_time)
        )?;
        tco::writeln!(
            stdout,
            "{}余额 {}{}",
            fg!(Some(Color::Cyan)),
            fg!(Some(Color::Yellow)),
            f.balance
        )?;
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct TUNetOnline {}

impl TUNetCommand for TUNetOnline {
    fn run(&self) -> Result<()> {
        let client = create_http_client();
        let cred = read_cred()?;
        let mut c = UseregHelper::new(cred, &client);
        c.login()?;
        let us = c.users()?;
        let mac_addrs = MacAddressIterator::new()
            .map(|it| it.collect::<Vec<_>>())
            .unwrap_or_default();
        let stdout = StandardStream::stdout(ColorChoice::Auto);
        let mut stdout = tco::ResetGuard::Owned(stdout);
        tco::writeln!(stdout, "    IP地址            登录时间            MAC地址")?;
        for u in us {
            let is_self = mac_addrs
                .iter()
                .any(|it| Some(it) == u.mac_address.as_ref());
            tco::writeln!(
                stdout,
                "{}{:15} {}{:20} {}{}{}{}",
                fg!(Some(Color::Yellow)),
                u.address,
                fg!(Some(Color::Green)),
                FmtDateTime(u.login_time),
                fg!(Some(Color::Cyan)),
                u.mac_address.map(|a| a.to_string()).unwrap_or_default(),
                fg!(Some(Color::Magenta)),
                if is_self { "*" } else { "" }
            )?;
        }
        save_cred(c.cred())
    }
}

#[derive(Debug, StructOpt)]
struct TUNetUseregConnect {
    #[structopt(long, short)]
    /// IP地址
    address: Ipv4Addr,
}

impl TUNetCommand for TUNetUseregConnect {
    fn run(&self) -> Result<()> {
        let client = create_http_client();
        let cred = read_cred()?;
        let mut c = UseregHelper::new(cred, &client);
        c.login()?;
        let res = c.connect(self.address)?;
        println!("{}", res);
        save_cred(c.cred())
    }
}

#[derive(Debug, StructOpt)]
struct TUNetUseregDrop {
    #[structopt(long, short)]
    /// IP地址
    address: Ipv4Addr,
}

impl TUNetCommand for TUNetUseregDrop {
    fn run(&self) -> Result<()> {
        let client = create_http_client();
        let cred = read_cred()?;
        let mut c = UseregHelper::new(cred, &client);
        c.login()?;
        let res = c.drop(self.address)?;
        println!("{}", res);
        save_cred(c.cred())
    }
}

#[derive(Debug, StructOpt)]
struct TUNetDetail {
    #[structopt(long, short, default_value = "logout")]
    /// 排序方式
    order: NetDetailOrder,
    #[structopt(long, short)]
    /// 倒序
    descending: bool,
    #[structopt(long, short)]
    /// 按日期分组
    grouping: bool,
}

impl TUNetDetail {
    fn run_detail(&self) -> Result<()> {
        let client = create_http_client();
        let cred = read_cred()?;
        let mut c = UseregHelper::new(cred, &client);
        c.login()?;
        let mut details = c.details(self.order, self.descending)?;
        let stdout = StandardStream::stdout(ColorChoice::Auto);
        let mut stdout = tco::ResetGuard::Owned(stdout);
        tco::writeln!(stdout, "      登录时间             注销时间         流量")?;
        let mut total_flux = Flux(0);
        for d in &mut details {
            let d = d?;
            tco::writeln!(
                stdout,
                "{}{:20} {:20} {}{:>8}",
                fg!(Some(Color::Green)),
                FmtDateTime(d.login_time),
                FmtDateTime(d.logout_time),
                fg!(Some(get_flux_color(&d.flux, false))),
                d.flux
            )?;
            total_flux.0 += d.flux.0;
        }
        tco::writeln!(
            stdout,
            "{}总流量 {}{}{}",
            fg!(Some(Color::Cyan)),
            fg!(Some(get_flux_color(&total_flux, true))),
            bold!(true),
            total_flux
        )?;
        save_cred(c.cred())
    }

    fn run_detail_grouping(&self) -> Result<()> {
        let client = create_http_client();
        let cred = read_cred()?;
        let mut c = UseregHelper::new(cred, &client);
        c.login()?;
        let details = c
            .details(NetDetailOrder::LogoutTime, self.descending)?
            .try_collect::<_, Vec<_>, _>()?;
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
        let stdout = StandardStream::stdout(ColorChoice::Auto);
        let mut stdout = tco::ResetGuard::Owned(stdout);
        tco::writeln!(stdout, " 登录日期    流量")?;
        let mut total_flux = Flux(0);
        for (date, flux) in details {
            tco::writeln!(
                stdout,
                "{}{:10} {}{:>8}",
                fg!(Some(Color::Green)),
                date,
                fg!(Some(get_flux_color(&flux, true))),
                flux
            )?;
            total_flux.0 += flux.0;
        }
        tco::writeln!(
            stdout,
            "{}总流量 {}{}{}",
            fg!(Some(Color::Cyan)),
            fg!(Some(get_flux_color(&total_flux, true))),
            bold!(true),
            total_flux
        )?;
        save_cred(c.cred())
    }
}

impl TUNetCommand for TUNetDetail {
    fn run(&self) -> Result<()> {
        if self.grouping {
            self.run_detail_grouping()
        } else {
            self.run_detail()
        }
    }
}

#[derive(Debug, StructOpt)]
struct TUNetDeleteCred {}

impl TUNetCommand for TUNetDeleteCred {
    fn run(&self) -> Result<()> {
        delete_cred()
    }
}
