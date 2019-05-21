use std::convert;
use std::num;
use std::result;
use std::string;
use std::time;

mod net;
pub mod strfmt;

pub struct NetCredential {
    username: string::String,
    password: string::String,
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
    pub fn from(s: &str) -> result::Result<NetFlux, ParseNetFluxError> {
        let split = s.split(',');
        let vec: Vec<&str> = split.collect();
        if vec.iter().count() <= 1 {
            Ok(NetFlux {
                username: string::String::new(),
                flux: 0,
                online_time: time::Duration::new(0, 0),
                balance: 0.0,
            })
        } else {
            Ok(NetFlux {
                username: vec[0].to_string(),
                flux: vec[6].to_string().parse::<u64>()?,
                online_time: time::Duration::from_secs(
                    vec[2].to_string().parse::<u64>()? - vec[1].to_string().parse::<u64>()?,
                ),
                balance: vec[11].to_string().parse::<f64>()?,
            })
        }
    }
}

#[derive(Debug)]
pub enum NetHelperError {
    HttpErr(reqwest::Error),
    NetFluxErr(ParseNetFluxError),
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

pub type Result<T> = result::Result<T, NetHelperError>;

pub enum NetState {
    Unknown,
    Net,
    Auth4,
    Auth6,
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
        _ => None,
    }
}

pub fn from_state_cred(s: NetState, u: &str, p: &str) -> Option<Box<NetConnectHelper>> {
    match s {
        NetState::Net => Some(Box::new(net::NetConnect::from_cred(u, p))),
        _ => None,
    }
}
