use chrono::Datelike;
use itertools::Itertools;
use mac_address::MacAddressIterator;
use std::cmp::Reverse;
use std::net::Ipv4Addr;
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, StandardStream};
use termcolor_output as tco;
use tunet_rust::{usereg::*, *};

mod settings;
mod strfmt;

use settings::*;
use strfmt::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "TsinghuaNetRust", about = "清华大学校园网客户端")]
enum TUNet {
    #[structopt(name = "login")]
    /// 登录
    Login {
        #[structopt(long, short = "s", default_value = "auto")]
        /// 连接方式
        host: NetState,
    },
    #[structopt(name = "logout")]
    /// 注销
    Logout {
        #[structopt(long, short = "s", default_value = "auto")]
        /// 连接方式
        host: NetState,
    },
    #[structopt(name = "status")]
    /// 查看在线状态
    Status {
        #[structopt(long, short = "s", default_value = "auto")]
        /// 连接方式
        host: NetState,
    },
    #[structopt(name = "online")]
    /// 查询在线IP
    Online,
    #[structopt(name = "connect")]
    /// 上线IP
    Connect {
        #[structopt(long, short)]
        /// IP地址
        address: Ipv4Addr,
    },
    #[structopt(name = "drop")]
    /// 下线IP
    Drop {
        #[structopt(long, short)]
        /// IP地址
        address: Ipv4Addr,
    },
    #[structopt(name = "detail")]
    /// 流量明细
    Detail {
        #[structopt(long, short, default_value = "logout")]
        /// 排序方式
        order: NetDetailOrder,
        #[structopt(long, short)]
        /// 倒序
        descending: bool,
        #[structopt(long, short)]
        /// 按日期分组
        grouping: bool,
    },
    #[structopt(name = "deletecred")]
    /// 删除用户名和密码
    DeleteCredential {},
}

fn main() -> Result<()> {
    let opt = TUNet::from_args();
    match opt {
        TUNet::Login { host } => do_login(host),
        TUNet::Logout { host } => do_logout(host),
        TUNet::Status { host } => do_status(host),
        TUNet::Online => do_online(),
        TUNet::Connect { address } => do_connect(address),
        TUNet::Drop { address } => do_drop(address),
        TUNet::Detail {
            order,
            descending,
            grouping,
        } => {
            if grouping {
                do_detail_grouping(order, descending)
            } else {
                do_detail(order, descending)
            }
        }
        TUNet::DeleteCredential {} => delete_cred(),
    }
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

fn do_login(s: NetState) -> Result<()> {
    let client = create_http_client();
    let (u, p, ac_ids) = read_cred()?;
    let ac_ids = {
        let mut c = TUNetConnect::from_state_cred_client(s, &u, &p, &client, ac_ids)?;
        let res = c.login()?;
        println!("{}", res);
        c.ac_ids().to_vec()
    };
    save_cred(u, p, ac_ids)
}

fn do_logout(s: NetState) -> Result<()> {
    let client = create_http_client();
    let (u, ac_ids) = read_username()?;
    let mut c = TUNetConnect::from_state_cred_client(s, &u, "", &client, ac_ids)?;
    let res = c.logout()?;
    println!("{}", res);
    Ok(())
}

fn do_status(s: NetState) -> Result<()> {
    let client = create_http_client();
    let c = TUNetConnect::from_state_cred_client(s, "", "", &client, vec![])?;
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

fn do_online() -> Result<()> {
    let client = create_http_client();
    let (u, p, ac_ids) = read_cred()?;
    {
        let mut c = UseregHelper::from_cred_client(&u, &p, &client);
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
    }
    save_cred(u, p, ac_ids)
}

fn do_connect(a: Ipv4Addr) -> Result<()> {
    let client = create_http_client();
    let (u, p, ac_ids) = read_cred()?;
    {
        let mut c = UseregHelper::from_cred_client(&u, &p, &client);
        c.login()?;
        let res = c.connect(a)?;
        println!("{}", res);
    }
    save_cred(u, p, ac_ids)
}

fn do_drop(a: Ipv4Addr) -> Result<()> {
    let client = create_http_client();
    let (u, p, ac_ids) = read_cred()?;
    {
        let mut c = UseregHelper::from_cred_client(&u, &p, &client);
        c.login()?;
        let res = c.drop(a)?;
        println!("{}", res);
    }
    save_cred(u, p, ac_ids)
}

fn do_detail(o: NetDetailOrder, d: bool) -> Result<()> {
    let client = create_http_client();
    let (u, p, ac_ids) = read_cred()?;
    {
        let mut c = UseregHelper::from_cred_client(&u, &p, &client);
        c.login()?;
        let mut details = c.details(o, d)?;
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
    }
    save_cred(u, p, ac_ids)
}

fn do_detail_grouping(o: NetDetailOrder, d: bool) -> Result<()> {
    let client = create_http_client();
    let (u, p, ac_ids) = read_cred()?;
    {
        let mut c = UseregHelper::from_cred_client(&u, &p, &client);
        c.login()?;
        let details = c
            .details(NetDetailOrder::LogoutTime, d)?
            .try_collect::<_, Vec<_>, _>()?;
        let mut details = details
            .into_iter()
            .group_by(|detail| detail.logout_time.date())
            .into_iter()
            .map(|(key, group)| (key, Flux(group.map(|detail| detail.flux.0).sum::<u64>())))
            .collect::<Vec<_>>();
        match o {
            NetDetailOrder::Flux => {
                if d {
                    details.sort_unstable_by_key(|(_, flux)| Reverse(*flux));
                } else {
                    details.sort_unstable_by_key(|(_, flux)| *flux);
                }
            }
            _ => {
                if d {
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
    }
    save_cred(u, p, ac_ids)
}
