#![feature(try_trait)]

use serde_json::error;
use std::io;
use std::result;
use std::str;
use std::time;
use thiserror::Error;

mod auth;
mod net;
pub mod suggest;
pub mod usereg;

pub struct NetCredential {
    username: String,
    password: String,
}

impl NetCredential {
    pub fn new() -> Self {
        NetCredential::from_cred(String::new(), String::new())
    }

    pub fn from_cred(u: String, p: String) -> Self {
        NetCredential {
            username: u,
            password: p,
        }
    }
}

pub struct NetFlux {
    pub username: String,
    pub flux: u64,
    pub online_time: time::Duration,
    pub balance: f64,
}

impl NetFlux {
    pub fn new() -> Self {
        NetFlux::from_detail(String::new(), 0, time::Duration::new(0, 0), 0.0)
    }

    pub fn from_detail(u: String, f: u64, t: time::Duration, b: f64) -> Self {
        NetFlux {
            username: u,
            flux: f,
            online_time: t,
            balance: b,
        }
    }

    pub fn from_str(s: &str) -> Self {
        let vec = s.split(',').collect::<Vec<_>>();
        if vec.len() <= 1 {
            NetFlux::new()
        } else {
            NetFlux::from_detail(
                vec[0].to_string(),
                vec[6].parse::<u64>().unwrap_or_default(),
                time::Duration::from_secs(
                    (vec[2].parse::<i64>().unwrap_or_default()
                        - vec[1].parse::<i64>().unwrap_or_default())
                    .max(0) as u64,
                ),
                vec[11].parse::<f64>().unwrap_or_default(),
            )
        }
    }
}

#[derive(Debug, Error)]
pub enum NetHelperError {
    #[error(transparent)]
    HttpErr(#[from] reqwest::Error),
    #[error(transparent)]
    JsonErr(#[from] error::Error),
    #[error("Cannot determine value of ac_id")]
    NoAcIdErr,
    #[error("Auth log failed: `{0}`")]
    LogErr(String),
    #[error(transparent)]
    IoErr(#[from] io::Error),
    #[error("Cannot determine host type")]
    HostErr,
    #[error("Cannot determine config dir")]
    ConfigDirErr,
}

pub type Result<T> = result::Result<T, NetHelperError>;

#[derive(Debug)]
pub enum NetState {
    Unknown,
    Net,
    Auth4,
    Auth6,
}

impl str::FromStr for NetState {
    type Err = String;
    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let ls = s.to_lowercase();
        if ls == "net" {
            Ok(NetState::Net)
        } else if ls == "auth4" {
            Ok(NetState::Auth4)
        } else if ls == "auth6" {
            Ok(NetState::Auth6)
        } else if ls == "auto" {
            Ok(suggest::suggest())
        } else {
            Err("连接方式错误".to_string())
        }
    }
}

pub trait NetHelper {
    fn login(&mut self) -> Result<String>;
    fn logout(&mut self) -> Result<String>;
}

pub trait NetConnectHelper: NetHelper {
    fn flux(&self) -> Result<NetFlux>;
    fn ac_ids(&self) -> &[i32];
}

pub enum TUNetConnect<'a> {
    Net(net::NetConnect<'a>),
    Auth(auth::AuthConnect<'a>),
}

impl<'a> NetHelper for TUNetConnect<'a> {
    fn login(&mut self) -> Result<String> {
        match self {
            Self::Net(c) => c.login(),
            Self::Auth(c) => c.login(),
        }
    }
    fn logout(&mut self) -> Result<String> {
        match self {
            Self::Net(c) => c.logout(),
            Self::Auth(c) => c.logout(),
        }
    }
}

impl<'a> NetConnectHelper for TUNetConnect<'a> {
    fn flux(&self) -> Result<NetFlux> {
        match self {
            Self::Net(c) => c.flux(),
            Self::Auth(c) => c.flux(),
        }
    }
    fn ac_ids(&self) -> &[i32] {
        match self {
            Self::Net(c) => c.ac_ids(),
            Self::Auth(c) => c.ac_ids(),
        }
    }
}

pub fn from_state_cred_client<'a>(
    s: NetState,
    u: String,
    p: String,
    client: &'a reqwest::blocking::Client,
    ac_ids: Vec<i32>,
) -> Result<TUNetConnect<'a>> {
    match s {
        NetState::Net => Ok(TUNetConnect::Net(net::NetConnect::from_cred_client(
            u, p, client,
        ))),
        NetState::Auth4 => Ok(TUNetConnect::Auth(auth::AuthConnect::from_cred_client(
            u, p, client, ac_ids,
        ))),
        NetState::Auth6 => Ok(TUNetConnect::Auth(auth::AuthConnect::from_cred_client_v6(
            u, p, client, ac_ids,
        ))),
        _ => Err(NetHelperError::HostErr),
    }
}
