#![feature(try_trait)]

use serde_json::error;
use std::cmp;
use std::convert;
use std::num;
use std::option;
use std::result;
use std::str;
use std::string;
use std::time;

mod auth;
mod net;
pub mod strfmt;
pub mod suggest;

pub struct NetCredential {
    username: string::String,
    password: string::String,
}

impl NetCredential {
    pub fn new() -> Self {
        NetCredential::from_cred(string::String::new(), string::String::new())
    }

    pub fn from_cred(u: string::String, p: string::String) -> Self {
        NetCredential {
            username: u,
            password: p,
        }
    }
}

#[derive(Debug)]
pub enum ParseNetFluxError {
    IntErr(num::ParseIntError),
    FloatErr(num::ParseFloatError),
}

impl convert::From<num::ParseIntError> for ParseNetFluxError {
    fn from(e: num::ParseIntError) -> Self {
        ParseNetFluxError::IntErr(e)
    }
}

impl convert::From<num::ParseFloatError> for ParseNetFluxError {
    fn from(e: num::ParseFloatError) -> Self {
        ParseNetFluxError::FloatErr(e)
    }
}

pub struct NetFlux {
    pub username: string::String,
    pub flux: u64,
    pub online_time: time::Duration,
    pub balance: f64,
}

impl NetFlux {
    pub fn new() -> Self {
        NetFlux::from_detail(string::String::new(), 0, time::Duration::new(0, 0), 0.0)
    }

    pub fn from_detail(u: string::String, f: u64, t: time::Duration, b: f64) -> Self {
        NetFlux {
            username: u,
            flux: f,
            online_time: t,
            balance: b,
        }
    }

    pub fn from_str(s: &str) -> result::Result<NetFlux, ParseNetFluxError> {
        let split = s.split(',');
        let vec: Vec<&str> = split.collect();
        if vec.len() <= 1 {
            Ok(NetFlux::new())
        } else {
            Ok(NetFlux::from_detail(
                vec[0].to_string(),
                vec[6].to_string().parse::<u64>()?,
                time::Duration::from_secs(cmp::max(
                    vec[2].to_string().parse::<i64>()? - vec[1].to_string().parse::<i64>()?,
                    0,
                ) as u64),
                vec[11].to_string().parse::<f64>()?,
            ))
        }
    }
}

#[derive(Debug)]
pub enum NetHelperError {
    HttpErr(reqwest::Error),
    NetFluxErr(ParseNetFluxError),
    JsonErr(error::Error),
    NoAcIdErr,
    NoneErr(option::NoneError),
}

impl convert::From<reqwest::Error> for NetHelperError {
    fn from(e: reqwest::Error) -> Self {
        NetHelperError::HttpErr(e)
    }
}

impl convert::From<ParseNetFluxError> for NetHelperError {
    fn from(e: ParseNetFluxError) -> Self {
        NetHelperError::NetFluxErr(e)
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
    type Err = string::String;
    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let ls = s.to_lowercase();
        if ls == "net" {
            Ok(NetState::Net)
        } else if ls == "auth4" {
            Ok(NetState::Auth4)
        } else if ls == "auth6" {
            Ok(NetState::Auth6)
        } else {
            Ok(NetState::Unknown)
        }
    }
}

pub trait NetHelper {
    fn login(&self) -> Result<string::String>;
    fn logout(&self) -> Result<string::String>;
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

pub fn from_state_cred(
    s: NetState,
    u: string::String,
    p: string::String,
) -> Option<Box<NetConnectHelper>> {
    match s {
        NetState::Net => Some(Box::new(net::NetConnect::from_cred(u, p))),
        NetState::Auth4 => Some(Box::new(auth::AuthConnect::from_cred(u, p))),
        NetState::Auth6 => Some(Box::new(auth::AuthConnect::from_cred_v6(u, p))),
        _ => None,
    }
}
