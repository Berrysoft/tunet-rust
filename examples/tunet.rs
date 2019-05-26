use chrono::{Datelike, Duration};
use itertools::Itertools;
use std::net::Ipv4Addr;
use std::option::Option;
use std::string::String;
use structopt::StructOpt;
use tunet_rust::usereg::*;
use tunet_rust::*;

mod strfmt {
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
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "TsinghuaNet.Rust",
    about = "清华大学校园网客户端",
    raw(setting = "structopt::clap::AppSettings::ColorAlways")
)]
enum TUNet {
    #[structopt(name = "login")]
    /// 登录
    Login {
        #[structopt(long, short)]
        /// 用户名
        username: String,
        #[structopt(long, short)]
        /// 密码
        password: String,
        #[structopt(long, short = "s", default_value = "auto")]
        /// 连接方式
        host: NetState,
    },
    #[structopt(name = "logout")]
    /// 注销
    Logout {
        #[structopt(long, short)]
        /// 用户名，Auth连接方式必选
        username: Option<String>,
        #[structopt(long, short)]
        /// 密码，Auth连接方式必选
        password: Option<String>,
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
    Online {
        #[structopt(long, short)]
        /// 用户名
        username: String,
        #[structopt(long, short)]
        /// 密码
        password: String,
    },
    #[structopt(name = "drop")]
    /// 下线IP
    Drop {
        #[structopt(long, short)]
        /// 用户名
        username: String,
        #[structopt(long, short)]
        /// 密码
        password: String,
        #[structopt(long, short)]
        /// IP地址
        address: Ipv4Addr,
    },
    #[structopt(name = "detail")]
    /// 流量明细
    Detail {
        #[structopt(long, short)]
        /// 用户名
        username: String,
        #[structopt(long, short)]
        /// 密码
        password: String,
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
    let opt = TUNet::from_args();
    match opt {
        TUNet::Login {
            username,
            password,
            host,
        } => {
            do_login(username, password, host)?;
        }
        TUNet::Logout {
            username,
            password,
            host,
        } => {
            do_logout(username, password, host)?;
        }
        TUNet::Status { host } => {
            do_status(host)?;
        }
        TUNet::Online { username, password } => {
            do_online(username, password)?;
        }
        TUNet::Drop {
            username,
            password,
            address,
        } => {
            do_drop(username, password, address)?;
        }
        TUNet::Detail {
            username,
            password,
            order,
            descending,
            grouping,
        } => {
            if grouping {
                do_detail_grouping(username, password, order, descending)?;
            } else {
                do_detail(username, password, order, descending)?;
            }
        }
    };
    Ok(())
}

fn do_login(u: String, p: String, s: NetState) -> Result<()> {
    let c = from_state_cred(s, u, p)?;
    let res = c.login()?;
    println!("{}", res);
    Ok(())
}

fn do_logout(uoption: Option<String>, poption: Option<String>, s: NetState) -> Result<()> {
    let u = uoption.unwrap_or(String::new());
    let p = poption.unwrap_or(String::new());
    let c = from_state_cred(s, u, p)?;
    let res = c.logout()?;
    println!("{}", res);
    Ok(())
}

fn do_status(s: NetState) -> Result<()> {
    let c = from_state(s)?;
    let f = c.flux()?;
    println!("用户：{}", f.username);
    println!("流量：{}", strfmt::format_flux(f.flux));
    println!("时长：{}", strfmt::format_duration(f.online_time));
    println!("余额：¥{:.2}", f.balance);
    Ok(())
}

const TUNET_DATE_TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";
const TUNET_DATE_FORMAT: &'static str = "%Y-%m-%d";

fn do_online(u: String, p: String) -> Result<()> {
    let c = UseregHelper::from_cred(u, p)?;
    c.login()?;
    let us = c.users()?;
    println!("|       IP       |       登录时间       |   客户端   |");
    println!("{}", "=".repeat(54));
    for u in us {
        println!(
            "| {:14} | {:20} | {:10} |",
            u.address.to_string(),
            u.login_time.format(TUNET_DATE_TIME_FORMAT).to_string(),
            u.client
        );
    }
    Ok(())
}

fn do_drop(u: String, p: String, a: Ipv4Addr) -> Result<()> {
    let c = UseregHelper::from_cred(u, p)?;
    c.login()?;
    let res = c.drop(a)?;
    println!("{}", res);
    Ok(())
}

fn do_detail(u: String, p: String, o: NetDetailOrder, d: bool) -> Result<()> {
    let c = UseregHelper::from_cred(u, p)?;
    c.login()?;
    let details = c.details(o, d)?;
    let mut total_time = Duration::zero();
    let mut total_flux = 0u64;
    println!("|       登录时间       |       注销时间       |   流量   |");
    println!("{}", "=".repeat(58));
    for d in details {
        println!(
            "| {:20} | {:20} | {:>8} |",
            d.login_time.format(TUNET_DATE_TIME_FORMAT).to_string(),
            d.logout_time.format(TUNET_DATE_TIME_FORMAT).to_string(),
            strfmt::format_flux(d.flux)
        );
        total_time = total_time + (d.logout_time - d.login_time);
        total_flux += d.flux;
    }
    println!(
        "总时长：{}",
        strfmt::format_duration(total_time.to_std().unwrap_or_default())
    );
    println!("总流量：{}", strfmt::format_flux(total_flux));
    Ok(())
}

fn do_detail_grouping(u: String, p: String, o: NetDetailOrder, d: bool) -> Result<()> {
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
    println!("|    日期    |   流量   |");
    println!("{}", "=".repeat(25));
    for d in details {
        println!(
            "| {:10} | {:>8} |",
            d.0.format(TUNET_DATE_FORMAT).to_string(),
            strfmt::format_flux(d.1)
        );
        total_flux += d.1;
    }
    println!("总流量：{}", strfmt::format_flux(total_flux));
    Ok(())
}
