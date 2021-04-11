#![forbid(unsafe_code)]

use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use thiserror::Error;

pub use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
pub use ureq::Agent as HttpClient;

mod auth;
mod net;
pub mod suggest;
pub mod usereg;

#[derive(Debug, Default)]
pub struct NetCredential<'a> {
    username: Cow<'a, str>,
    password: Cow<'a, str>,
}

impl<'a> NetCredential<'a> {
    pub fn from_cred<SU: Into<Cow<'a, str>>, SP: Into<Cow<'a, str>>>(u: SU, p: SP) -> Self {
        NetCredential {
            username: u.into(),
            password: p.into(),
        }
    }
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

pub enum TUNetConnect<'a, 's> {
    Net(net::NetConnect<'a, 's>),
    Auth4(auth::AuthConnect<'a, 's, 4>),
    Auth6(auth::AuthConnect<'a, 's, 6>),
}

impl<'a, 's> TUNetConnect<'a, 's> {
    pub fn login(&mut self) -> Result<String> {
        match self {
            Self::Net(c) => c.login(),
            Self::Auth4(c) => c.login(),
            Self::Auth6(c) => c.login(),
        }
    }
    pub fn logout(&mut self) -> Result<String> {
        match self {
            Self::Net(c) => c.logout(),
            Self::Auth4(c) => c.logout(),
            Self::Auth6(c) => c.logout(),
        }
    }
    pub fn flux(&self) -> Result<NetFlux> {
        match self {
            Self::Net(c) => c.flux(),
            Self::Auth4(c) => c.flux(),
            Self::Auth6(c) => c.flux(),
        }
    }
    pub fn ac_ids(&self) -> &[i32] {
        match self {
            Self::Net(_) => &[0; 0],
            Self::Auth4(c) => c.ac_ids(),
            Self::Auth6(c) => c.ac_ids(),
        }
    }
    pub fn from_state_cred_client<SU: Into<Cow<'s, str>>, SP: Into<Cow<'s, str>>>(
        s: NetState,
        u: SU,
        p: SP,
        client: &'a HttpClient,
        ac_ids: Vec<i32>,
    ) -> Result<Self> {
        match s {
            NetState::Net => Ok(TUNetConnect::Net(net::NetConnect::from_cred_client(
                u, p, client,
            ))),
            NetState::Auth4 => Ok(TUNetConnect::Auth4(auth::AuthConnect::from_cred_client(
                u, p, client, ac_ids,
            ))),
            NetState::Auth6 => Ok(TUNetConnect::Auth6(auth::AuthConnect::from_cred_client(
                u, p, client, ac_ids,
            ))),
            NetState::Auto => {
                let s = suggest::suggest(client);
                debug_assert_ne!(s, NetState::Auto);
                Self::from_state_cred_client(s, u, p, client, ac_ids)
            }
            NetState::Unknown => Err(NetHelperError::HostErr),
        }
    }
}

pub fn create_http_client() -> HttpClient {
    ureq::AgentBuilder::new().redirects(0).build()
}
