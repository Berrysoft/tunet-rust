use ansi_term;
use ansi_term::Color;
use chrono::Datelike;
use itertools::Itertools;
use rpassword::read_password;
use std::io::{stdin, stdout, Write};
use std::net::Ipv4Addr;
use std::option::Option;
use std::string::String;
use structopt::StructOpt;
use tunet_rust::usereg::*;
use tunet_rust::*;

mod strfmt {
    use ansi_term::{ANSIString, Color};
    use chrono::{NaiveDate, NaiveDateTime};
    use std::time;

    pub fn format_flux(flux: u64) -> String {
        let mut f = flux as f64;
        if f < 1000.0 {
            return format!("{} B", f);
        }
        f /= 1000.0;
        if f < 1000.0 {
            return format!("{:.2} K", f);
        }
        f /= 1000.0;
        if f < 1000.0 {
            return format!("{:.2} M", f);
        }
        f /= 1000.0;
        return format!("{:.2} G", f);
    }

    pub fn colored_flux(flux: u64, total: bool, right_aligned: bool) -> ANSIString<'static> {
        let f = if right_aligned {
            format!("{:>8}", format_flux(flux))
        } else {
            format_flux(flux)
        };
        if flux == 0 {
            Color::Blue.normal().paint(f)
        } else if flux < if total { 20_000_000_000 } else { 2_000_000_000 } {
            Color::Yellow.bold().paint(f)
        } else {
            Color::Purple.bold().paint(f)
        }
    }

    pub fn format_duration(d: time::Duration) -> String {
        let mut total_sec = d.as_secs();
        let sec = total_sec % 60;
        total_sec /= 60;
        let min = total_sec % 60;
        total_sec /= 60;
        let h = total_sec % 24;
        total_sec /= 24;
        if total_sec > 0 {
            format!("{}.{:02}:{:02}:{:02}", total_sec, h, min, sec)
        } else {
            format!("{:02}:{:02}:{:02}", h, min, sec)
        }
    }

    pub fn colored_duration(d: time::Duration) -> ANSIString<'static> {
        Color::Green.normal().paint(format_duration(d))
    }

    pub fn format_currency(c: f64) -> String {
        format!("¥{:.2}", c)
    }

    pub fn colored_currency(c: f64) -> ANSIString<'static> {
        Color::Yellow.normal().paint(format_currency(c))
    }

    const TUNET_DATE_TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";
    const TUNET_DATE_FORMAT: &'static str = "%Y-%m-%d";

    pub fn format_date_time(t: NaiveDateTime) -> String {
        t.format(TUNET_DATE_TIME_FORMAT).to_string()
    }

    pub fn colored_date_time(t: NaiveDateTime) -> ANSIString<'static> {
        Color::Green
            .normal()
            .paint(format!("{:20}", format_date_time(t)))
    }

    pub fn format_date(t: NaiveDate) -> String {
        t.format(TUNET_DATE_FORMAT).to_string()
    }

    pub fn colored_date(t: NaiveDate) -> ANSIString<'static> {
        Color::Green
            .normal()
            .paint(format!("{:10}", format_date(t)))
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "TsinghuaNet.Rust",
    about = "清华大学校园网客户端",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
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
    Online {},
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
}

fn main() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    let console_color_ok = true;
    #[cfg(target_os = "windows")]
    let console_color_ok = ansi_term::enable_ansi_support().is_ok();
    let opt = TUNet::from_args();
    match opt {
        TUNet::Login { host } => {
            do_login(host)?;
        }
        TUNet::Logout { host } => {
            do_logout(host)?;
        }
        TUNet::Status { host } => {
            do_status(host, console_color_ok)?;
        }
        TUNet::Online {} => {
            do_online(console_color_ok)?;
        }
        TUNet::Drop { address } => {
            do_drop(address)?;
        }
        TUNet::Detail {
            order,
            descending,
            grouping,
        } => {
            if grouping {
                do_detail_grouping(order, descending, console_color_ok)?;
            } else {
                do_detail(order, descending, console_color_ok)?;
            }
        }
    };
    Ok(())
}

fn read_cred() -> Result<(String, String)> {
    print!("请输入用户名：");
    stdout().flush()?;
    let mut u = String::new();
    stdin().read_line(&mut u)?;
    print!("请输入密码：");
    stdout().flush()?;
    let p = read_password()?;
    Ok((u, p))
}

fn do_login(s: NetState) -> Result<()> {
    let (u, p) = read_cred()?;
    let c = from_state_cred(s, u, p)?;
    let res = c.login()?;
    println!("{}", res);
    Ok(())
}

fn do_logout(s: NetState) -> Result<()> {
    let (u, p) = read_cred()?;
    let c = from_state_cred(s, u, p)?;
    let res = c.logout()?;
    println!("{}", res);
    Ok(())
}

fn do_status(s: NetState, color: bool) -> Result<()> {
    let c = from_state(s)?;
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
    let (u, p) = read_cred()?;
    let c = UseregHelper::from_cred(u, p)?;
    c.login()?;
    let us = c.users()?;
    for u in us {
        if color {
            println!(
                "{} {} {}",
                Color::Yellow
                    .normal()
                    .paint(format!("{:15}", u.address.to_string())),
                strfmt::colored_date_time(u.login_time),
                Color::Blue.normal().paint(format!("{:10}", u.client))
            );
        } else {
            println!(
                "{:15} {:20} {:10}",
                u.address.to_string(),
                strfmt::format_date_time(u.login_time),
                u.client
            );
        }
    }
    Ok(())
}

fn do_drop(a: Ipv4Addr) -> Result<()> {
    let (u, p) = read_cred()?;
    let c = UseregHelper::from_cred(u, p)?;
    c.login()?;
    let res = c.drop(a)?;
    println!("{}", res);
    Ok(())
}

fn do_detail(o: NetDetailOrder, d: bool, color: bool) -> Result<()> {
    let (u, p) = read_cred()?;
    let c = UseregHelper::from_cred(u, p)?;
    c.login()?;
    let details = c.details(o, d)?;
    let mut total_flux = 0u64;
    for d in details {
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
    Ok(())
}

fn do_detail_grouping(o: NetDetailOrder, d: bool, color: bool) -> Result<()> {
    let (u, p) = read_cred()?;
    let c = UseregHelper::from_cred(u, p)?;
    c.login()?;
    let mut details = c
        .details(NetDetailOrder::LogoutTime, d)?
        .iter()
        .group_by(|detail| detail.logout_time.date())
        .into_iter()
        .map(|(key, group)| (key, group.map(|detail| detail.flux).sum::<u64>()))
        .collect::<Vec<_>>();
    match o {
        NetDetailOrder::Flux => {
            if d {
                details.sort_unstable_by_key(|x| -(x.1 as i64));
            } else {
                details.sort_unstable_by_key(|x| x.1);
            }
        }
        _ => {
            if d {
                details.sort_unstable_by_key(|x| -(x.0.day() as i32));
            }
        }
    }
    let mut total_flux = 0u64;
    for d in details {
        if color {
            println!(
                "{} {}",
                strfmt::colored_date(d.0),
                strfmt::colored_flux(d.1, false, true)
            );
        } else {
            println!(
                "{:10} {:>8}",
                strfmt::format_date(d.0),
                strfmt::format_flux(d.1)
            );
        }
        total_flux += d.1;
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
    Ok(())
}
