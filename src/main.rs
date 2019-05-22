use std::iter::FromIterator;
use std::option::Option;
use std::string::String;
use structopt::StructOpt;
use tunet_rust::strfmt;
use tunet_rust::suggest::suggest;
use tunet_rust::usereg::*;
use tunet_rust::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "tunet", about = "清华大学校园网客户端")]
enum TUNet {
    #[structopt(name = "login")]
    Login {
        #[structopt(name = "username", long, short)]
        username: String,
        #[structopt(name = "password", long, short)]
        password: String,
        #[structopt(name = "host", long, short = "s")]
        host: Option<NetState>,
    },
    #[structopt(name = "logout")]
    Logout {
        #[structopt(name = "username", long, short)]
        username: Option<String>,
        #[structopt(name = "password", long, short)]
        password: Option<String>,
        #[structopt(name = "host", long, short = "s")]
        host: Option<NetState>,
    },
    #[structopt(name = "status")]
    Status {
        #[structopt(name = "host", long, short = "s")]
        host: Option<NetState>,
    },
    #[structopt(name = "online")]
    Online {
        #[structopt(name = "username", long, short)]
        username: String,
        #[structopt(name = "password", long, short)]
        password: String,
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
    };
    Ok(())
}

fn do_login(u: String, p: String, soption: Option<NetState>) -> Result<()> {
    let s = soption.unwrap_or(suggest());
    let c = from_state_cred(s, u, p)?;
    let res = c.login()?;
    println!("{}", res);
    Ok(())
}

fn do_logout(
    uoption: Option<String>,
    poption: Option<String>,
    soption: Option<NetState>,
) -> Result<()> {
    let s = soption.unwrap_or(suggest());
    let u = uoption.unwrap_or(String::new());
    let p = poption.unwrap_or(String::new());
    let c = from_state_cred(s, u, p)?;
    let res = c.logout()?;
    println!("{}", res);
    Ok(())
}

fn do_status(soption: Option<NetState>) -> Result<()> {
    let s = soption.unwrap_or(suggest());
    let c = from_state(s)?;
    let f = c.flux()?;
    println!("用户：{}", f.username);
    println!("流量：{}", strfmt::format_flux(f.flux));
    println!("时长：{}", strfmt::format_duration(f.online_time));
    println!("余额：¥{:.2}", f.balance);
    Ok(())
}

fn do_online(u: String, p: String) -> Result<()> {
    let c = UseregHelper::from_cred(u, p)?;
    c.login()?;
    let us = c.users()?;
    println!("|       IP       |       登录时间       |   客户端   |");
    println!("{}", String::from_iter(['='; 54].iter()));
    for u in us {
        println!(
            "| {:15}| {:21}| {:11}|",
            u.address.to_string(),
            u.login_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            u.client
        );
    }
    Ok(())
}
