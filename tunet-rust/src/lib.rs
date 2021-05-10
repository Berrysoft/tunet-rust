#![forbid(unsafe_code)]

use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
pub use ureq::Agent as HttpClient;

mod auth;
mod net;
pub mod suggest;
pub mod usereg;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct NetCredential {
    #[cfg_attr(feature = "serde", serde(rename = "Username"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub username: String,
    #[cfg_attr(feature = "serde", serde(rename = "Password"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub password: String,
    #[cfg_attr(feature = "serde", serde(rename = "AcIds"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub ac_ids: Vec<i32>,
}

#[repr(transparent)]
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Flux(pub u64);

impl Flux {
    fn string(&self) -> String {
        let mut flux = self.0 as f64;
        if flux < 1000.0 {
            return format!("{} B", flux);
        }
        flux /= 1000.0;
        if flux < 1000.0 {
            return format!("{:.2} K", flux);
        }
        flux /= 1000.0;
        if flux < 1000.0 {
            return format!("{:.2} M", flux);
        }
        flux /= 1000.0;
        format!("{:.2} G", flux)
    }
}

impl Display for Flux {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.string())
    }
}

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct Balance(pub f64);

impl Display for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "¥{:.2}", self.0)
    }
}

#[derive(Debug)]
pub struct NetFlux {
    pub username: String,
    pub flux: Flux,
    pub online_time: Duration,
    pub balance: Balance,
}

impl Default for NetFlux {
    fn default() -> Self {
        Self {
            username: String::default(),
            flux: Flux::default(),
            online_time: Duration::zero(),
            balance: Balance::default(),
        }
    }
}

impl NetFlux {
    pub fn from_detail(u: String, f: Flux, t: Duration, b: Balance) -> Self {
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
            NetFlux::default()
        } else {
            NetFlux::from_detail(
                vec[0].to_string(),
                Flux(vec[6].parse::<u64>().unwrap_or_default()),
                Duration::seconds(
                    (vec[2].parse::<i64>().unwrap_or_default()
                        - vec[1].parse::<i64>().unwrap_or_default())
                    .max(0),
                ),
                Balance(vec[11].parse::<f64>().unwrap_or_default()),
            )
        }
    }
}

#[derive(Debug, Error)]
pub enum NetHelperError {
    #[error(transparent)]
    HttpErr(#[from] ureq::Error),
    #[error(transparent)]
    JsonErr(#[from] serde_json::error::Error),
    #[error("无法获取ac_id")]
    NoAcIdErr,
    #[error("操作失败：{0}")]
    LogErr(String),
    #[error(transparent)]
    IoErr(#[from] std::io::Error),
    #[error("排序方式无效")]
    OrderErr,
    #[error("无法确定登录方式")]
    HostErr,
    #[error("找不到配置文件目录")]
    ConfigDirErr,
}

pub type Result<T> = std::result::Result<T, NetHelperError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetState {
    Unknown,
    Net,
    Auth4,
    Auth6,
    Auto,
}

impl std::str::FromStr for NetState {
    type Err = NetHelperError;
    fn from_str(s: &str) -> Result<Self> {
        let ls = s.to_lowercase();
        if ls == "net" {
            Ok(NetState::Net)
        } else if ls == "auth4" {
            Ok(NetState::Auth4)
        } else if ls == "auth6" {
            Ok(NetState::Auth6)
        } else if ls == "auto" {
            Ok(NetState::Auto)
        } else {
            Err(NetHelperError::HostErr)
        }
    }
}

pub trait TUNetHelper {
    fn login(&mut self) -> Result<String>;
    fn logout(&mut self) -> Result<String>;
    fn flux(&self) -> Result<NetFlux>;
    fn cred(&self) -> &NetCredential;
}

pub enum TUNetConnect<'a> {
    Net(net::NetConnect<'a>),
    Auth4(auth::AuthConnect<'a, 4>),
    Auth6(auth::AuthConnect<'a, 6>),
}

impl<'a> TUNetConnect<'a> {
    pub fn from_state_cred_client(
        s: NetState,
        cred: NetCredential,
        client: &'a HttpClient,
    ) -> Result<Self> {
        match s {
            NetState::Net => Ok(Self::Net(net::NetConnect::from_cred_client(cred, client))),
            NetState::Auth4 => Ok(Self::Auth4(auth::AuthConnect::from_cred_client(
                cred, client,
            ))),
            NetState::Auth6 => Ok(Self::Auth6(auth::AuthConnect::from_cred_client(
                cred, client,
            ))),
            NetState::Auto => {
                let s = suggest::suggest(client);
                debug_assert_ne!(s, NetState::Auto);
                Self::from_state_cred_client(s, cred, client)
            }
            NetState::Unknown => Err(NetHelperError::HostErr),
        }
    }
}

impl<'a> Deref for TUNetConnect<'a> {
    type Target = dyn TUNetHelper + 'a;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Net(c) => c,
            Self::Auth4(c) => c,
            Self::Auth6(c) => c,
        }
    }
}

impl<'a> DerefMut for TUNetConnect<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Net(c) => c,
            Self::Auth4(c) => c,
            Self::Auth6(c) => c,
        }
    }
}

pub fn create_http_client() -> HttpClient {
    ureq::AgentBuilder::new().redirects(0).build()
}
