#![feature(try_trait)]
#![feature(const_str_as_bytes)]

use serde_json::error;
use std::cmp;
use std::convert;
use std::option;
use std::result;
use std::str;
use std::string::String;
use std::time;

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
        let split = s.split(',');
        let vec = split.collect::<Vec<_>>();
        if vec.len() <= 1 {
            NetFlux::new()
        } else {
            NetFlux::from_detail(
                vec[0].to_string(),
                vec[6].to_string().parse::<u64>().unwrap_or_default(),
                time::Duration::from_secs(cmp::max(
                    vec[2].to_string().parse::<i64>().unwrap_or_default()
                        - vec[1].to_string().parse::<i64>().unwrap_or_default(),
                    0,
                ) as u64),
                vec[11].to_string().parse::<f64>().unwrap_or_default(),
            )
        }
    }
}

#[derive(Debug)]
pub enum NetHelperError {
    HttpErr(reqwest::Error),
    JsonErr(error::Error),
    NoAcIdErr,
    NoneErr(option::NoneError),
}

impl convert::From<reqwest::Error> for NetHelperError {
    fn from(e: reqwest::Error) -> Self {
        NetHelperError::HttpErr(e)
    }
}

impl convert::From<error::Error> for NetHelperError {
    fn from(e: error::Error) -> Self {
        NetHelperError::JsonErr(e)
    }
}

impl convert::From<option::NoneError> for NetHelperError {
    fn from(e: option::NoneError) -> Self {
        NetHelperError::NoneErr(e)
    }
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
    fn login(&self) -> Result<String>;
    fn logout(&self) -> Result<String>;
}

pub trait NetConnectHelper: NetHelper {
    fn flux(&self) -> Result<NetFlux>;
}

pub fn from_state(s: NetState) -> Option<Box<NetConnectHelper>> {
    match s {
        NetState::Net => Some(Box::new(net::NetConnect::new())),
        NetState::Auth4 => Some(Box::new(auth::AuthConnect::new())),
        NetState::Auth6 => Some(Box::new(auth::AuthConnect::new_v6())),
        _ => None,
    }
}

pub fn from_state_cred(s: NetState, u: String, p: String) -> Option<Box<NetConnectHelper>> {
    match s {
        NetState::Net => Some(Box::new(net::NetConnect::from_cred(u, p))),
        NetState::Auth4 => Some(Box::new(auth::AuthConnect::from_cred(u, p))),
        NetState::Auth6 => Some(Box::new(auth::AuthConnect::from_cred_v6(u, p))),
        _ => None,
    }
}
