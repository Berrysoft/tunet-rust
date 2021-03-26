use ansi_term::Color;
use chrono::Datelike;
use itertools::Itertools;
use mac_address::MacAddressIterator;
use std::cmp::Reverse;
use std::net::Ipv4Addr;
use structopt::StructOpt;
use tunet_rust::{usereg::*, *};

mod settings;
mod strfmt;

use settings::*;

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
    #[cfg(not(target_os = "windows"))]
    let console_color_ok = true;
    #[cfg(target_os = "windows")]
    let console_color_ok = ansi_term::enable_ansi_support().is_ok();
    let opt = TUNet::from_args();
    match opt {
        TUNet::Login { host } => do_login(host),
        TUNet::Logout { host } => do_logout(host),
        TUNet::Status { host } => do_status(host, console_color_ok),
        TUNet::Online => do_online(console_color_ok),
        TUNet::Connect { address } => do_connect(address),
        TUNet::Drop { address } => do_drop(address),
        TUNet::Detail {
            order,
            descending,
            grouping,
        } => {
            if grouping {
                do_detail_grouping(order, descending, console_color_ok)
            } else {
                do_detail(order, descending, console_color_ok)
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

fn do_status(s: NetState, color: bool) -> Result<()> {
    let client = create_http_client();
    let c = TUNetConnect::from_state_cred_client(s, "", "", &client, vec![])?;
    let f = c.flux()?;
    if color {
        println!(
            "{} {}",
            Color::Cyan.normal().paint("用户"),
            Color::Yellow.normal().paint(f.username)
        );
        println!(
            "{} {}",
            Color::Cyan.normal().paint("流量"),
            strfmt::colored_flux(f.flux, true, false)
        );
        println!(
            "{} {}",
            Color::Cyan.normal().paint("时长"),
            strfmt::colored_duration(f.online_time)
        );
        println!(
            "{} {}",
            Color::Cyan.normal().paint("余额"),
            strfmt::colored_currency(f.balance)
        );
    } else {
        println!("{} {}", "用户", f.username);
        println!("{} {}", "流量", strfmt::format_flux(f.flux));
        println!("{} {}", "时长", strfmt::format_duration(f.online_time));
        println!("{} {}", "余额", strfmt::format_currency(f.balance));
    }
    Ok(())
}

fn do_online(color: bool) -> Result<()> {
    let client = create_http_client();
    let (u, p, ac_ids) = read_cred()?;
    {
        let mut c = UseregHelper::from_cred_client(&u, &p, &client);
        c.login()?;
        let us = c.users()?;
        println!("    IP地址            登录时间            MAC地址");
        for u in us {
            let is_self = MacAddressIterator::new()
                .map(|mut it| it.any(|self_addr| Some(self_addr) == u.mac_address))
                .unwrap_or(false);
            let is_self = if is_self { "*" } else { "" };
            if color {
                println!(
                    "{} {} {}{}",
                    Color::Yellow
                        .normal()
                        .paint(format!("{:15}", u.address.to_string())),
                    strfmt::colored_date_time(u.login_time),
                    Color::Cyan
                        .normal()
                        .paint(u.mac_address.map(|a| a.to_string()).unwrap_or_default()),
                    Color::Purple.normal().paint(is_self)
                );
            } else {
                println!(
                    "{:15} {:20} {}{}",
                    u.address.to_string(),
                    strfmt::format_date_time(u.login_time),
                    u.mac_address.map(|a| a.to_string()).unwrap_or_default(),
                    is_self
                );
            }
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

fn do_detail(o: NetDetailOrder, d: bool, color: bool) -> Result<()> {
    let client = create_http_client();
    let (u, p, ac_ids) = read_cred()?;
    {
        let mut c = UseregHelper::from_cred_client(&u, &p, &client);
        c.login()?;
        let mut details = c.details(o, d)?;
        println!("      登录时间             注销时间         流量");
        let mut total_flux = 0u64;
        for d in &mut details {
            let d = d?;
            if color {
                println!(
                    "{} {} {}",
                    strfmt::colored_date_time(d.login_time),
                    strfmt::colored_date_time(d.logout_time),
                    strfmt::colored_flux(d.flux, false, true)
                );
            } else {
                println!(
                    "{:20} {:20} {:>8}",
                    strfmt::format_date_time(d.login_time),
                    strfmt::format_date_time(d.logout_time),
                    strfmt::format_flux(d.flux)
                );
            }
            total_flux += d.flux;
        }
        if color {
            println!(
                "{} {}",
                Color::Cyan.normal().paint("总流量"),
                strfmt::colored_flux(total_flux, true, false)
            );
        } else {
            println!("{} {}", "总流量", strfmt::format_flux(total_flux));
        }
    }
    save_cred(u, p, ac_ids)
}

fn do_detail_grouping(o: NetDetailOrder, d: bool, color: bool) -> Result<()> {
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
            .map(|(key, group)| (key, group.map(|detail| detail.flux).sum::<u64>()))
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
        println!(" 登录日期    流量");
        let mut total_flux = 0;
        for (date, flux) in details {
            if color {
                println!(
                    "{} {}",
                    strfmt::colored_date(date),
                    strfmt::colored_flux(flux, false, true)
                );
            } else {
                println!(
                    "{:10} {:>8}",
                    strfmt::format_date(date),
                    strfmt::format_flux(flux)
                );
            }
            total_flux += flux;
        }
        if color {
            println!(
                "{} {}",
                Color::Cyan.normal().paint("总流量"),
                strfmt::colored_flux(total_flux, true, false)
            );
        } else {
            println!("{} {}", "总流量", strfmt::format_flux(total_flux));
        }
    }
    save_cred(u, p, ac_ids)
}
