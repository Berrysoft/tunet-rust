use chrono::Datelike;
use itertools::Itertools;
use mac_address::MacAddressIterator;
use std::cmp::Reverse;
use std::io::Write;
use std::net::Ipv4Addr;
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tunet_rust::{usereg::*, *};

mod settings;
mod strfmt;

use settings::*;
use strfmt::FmtColor;

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
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let mut labelspec = ColorSpec::new();
    labelspec.set_fg(Some(Color::Cyan));
    stdout.set_color(&labelspec)?;
    write!(&mut stdout, "用户 ")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
    writeln!(&mut stdout, "{}", f.username)?;
    stdout.set_color(&labelspec)?;
    write!(&mut stdout, "流量 ")?;
    f.flux.fmt_color(&mut stdout)?;
    writeln!(&mut stdout)?;
    stdout.set_color(&labelspec)?;
    write!(&mut stdout, "时长 ")?;
    f.online_time.fmt_color(&mut stdout)?;
    writeln!(&mut stdout)?;
    stdout.set_color(&labelspec)?;
    write!(&mut stdout, "余额 ")?;
    f.balance.fmt_color(&mut stdout)?;
    writeln!(&mut stdout)?;
    Ok(())
}

fn do_online() -> Result<()> {
    let client = create_http_client();
    let (u, p, ac_ids) = read_cred()?;
    {
        let mut c = UseregHelper::from_cred_client(&u, &p, &client);
        c.login()?;
        let us = c.users()?;
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        writeln!(
            &mut stdout,
            "    IP地址            登录时间            MAC地址"
        )?;
        let mut ipspec = ColorSpec::new();
        ipspec.set_fg(Some(Color::Yellow));
        let mut macspec = ColorSpec::new();
        macspec.set_fg(Some(Color::Cyan));
        for u in us {
            let is_self = MacAddressIterator::new()
                .map(|mut it| it.any(|self_addr| Some(self_addr) == u.mac_address))
                .unwrap_or(false);
            stdout.set_color(&ipspec)?;
            write!(&mut stdout, "{:15} ", u.address.to_string())?;
            u.login_time.fmt_color_aligned(&mut stdout)?;
            stdout.set_color(&macspec)?;
            write!(
                &mut stdout,
                " {}",
                u.mac_address.map(|a| a.to_string()).unwrap_or_default()
            )?;
            if is_self {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
                write!(&mut stdout, "*")?;
            }
            writeln!(&mut stdout)?;
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
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        writeln!(
            &mut stdout,
            "      登录时间             注销时间         流量"
        )?;
        let mut total_flux = Flux(0);
        for d in &mut details {
            let d = d?;
            d.login_time.fmt_color_aligned(&mut stdout)?;
            write!(&mut stdout, " ")?;
            d.logout_time.fmt_color_aligned(&mut stdout)?;
            write!(&mut stdout, " ")?;
            d.flux.fmt_color_aligned(&mut stdout)?;
            writeln!(&mut stdout)?;
            total_flux.0 += d.flux.0;
        }
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
        write!(&mut stdout, "总流量 ")?;
        total_flux.fmt_color(&mut stdout)?;
        writeln!(&mut stdout)?;
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
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        writeln!(&mut stdout, " 登录日期    流量")?;
        let mut total_flux = Flux(0);
        for (date, flux) in details {
            date.fmt_color_aligned(&mut stdout)?;
            write!(&mut stdout, " ")?;
            flux.fmt_color_aligned(&mut stdout)?;
            writeln!(&mut stdout)?;
            total_flux.0 += flux.0;
        }
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
        write!(&mut stdout, "总流量 ")?;
        total_flux.fmt_color(&mut stdout)?;
        writeln!(&mut stdout)?;
    }
    save_cred(u, p, ac_ids)
}
