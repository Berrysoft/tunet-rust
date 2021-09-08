#![forbid(unsafe_code)]

use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use anyhow::Result;
use async_trait::async_trait;
pub use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
pub use reqwest::Client as HttpClient;

mod auth;
mod net;
pub mod suggest;
pub mod usereg;

#[derive(Debug, Error)]
pub enum NetHelperError {
    #[error("无法获取 ac_id")]
    NoAcIdErr,
    #[error("操作失败：{0}")]
    LogErr(String),
    #[error("无法识别的用户信息：{0}")]
    ParseNetFluxErr(String),
    #[error("排序方式无效")]
    OrderErr,
    #[error("无法确定登录方式")]
    HostErr,
}

pub type NetHelperResult<T> = std::result::Result<T, NetHelperError>;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct NetCredential {
    #[cfg_attr(feature = "serde", serde(rename = "Username", default))]
    pub username: String,
    #[cfg_attr(feature = "serde", serde(rename = "Password", default))]
    pub password: String,
    #[cfg_attr(feature = "serde", serde(rename = "AcIds", default))]
    pub ac_ids: Vec<i32>,
}

#[repr(transparent)]
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Flux(pub u64);

impl Flux {
    fn string(&self) -> String {
        let mut flux = self.0 as f64;
        if flux < 1000.0 {
            return format!("{} B", self.0);
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

impl std::str::FromStr for Flux {
    type Err = NetHelperError;
    fn from_str(s: &str) -> NetHelperResult<Self> {
        let (flux, unit) = s.split_at(s.len() - 1);
        Ok(Flux(
            (flux.trim_end().parse::<f64>().unwrap_or_default()
                * match unit {
                    "G" => 1_000_000_000.0,
                    "M" => 1_000_000.0,
                    "K" => 1_000.0,
                    _ => 1.0,
                }) as u64,
        ))
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

impl std::str::FromStr for NetFlux {
    type Err = NetHelperError;
    fn from_str(s: &str) -> NetHelperResult<Self> {
        let vec = s.split(',').collect::<Vec<_>>();
        if vec.len() >= 12 {
            Ok(NetFlux {
                username: vec[0].to_string(),
                flux: Flux(vec[6].parse::<u64>().unwrap_or_default()),
                online_time: Duration::seconds(
                    (vec[2].parse::<i64>().unwrap_or_default()
                        - vec[1].parse::<i64>().unwrap_or_default())
                    .max(0),
                ),
                balance: Balance(vec[11].parse::<f64>().unwrap_or_default()),
            })
        } else {
            Err(NetHelperError::ParseNetFluxErr(s.to_string()))
        }
    }
}

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
    fn from_str(s: &str) -> NetHelperResult<Self> {
        if s.eq_ignore_ascii_case("net") {
            Ok(NetState::Net)
        } else if s.eq_ignore_ascii_case("auth4") {
            Ok(NetState::Auth4)
        } else if s.eq_ignore_ascii_case("auth6") {
            Ok(NetState::Auth6)
        } else if s.eq_ignore_ascii_case("auto") {
            Ok(NetState::Auto)
        } else {
            Err(NetHelperError::HostErr)
        }
    }
}

#[async_trait]
pub trait TUNetHelper: Send + Sync {
    async fn login(&mut self) -> Result<String>;
    async fn logout(&mut self) -> Result<String>;
    async fn flux(&self) -> Result<NetFlux>;
    fn cred(&self) -> &NetCredential;
}

pub enum TUNetConnect {
    Net(net::NetConnect),
    Auth4(auth::AuthConnect<4>),
    Auth6(auth::AuthConnect<6>),
}

impl TUNetConnect {
    pub async fn new(
        mut s: NetState,
        cred: NetCredential,
        client: HttpClient,
    ) -> NetHelperResult<Self> {
        if let NetState::Auto = s {
            s = suggest::suggest(&client).await;
            debug_assert_ne!(s, NetState::Auto);
        }
        match s {
            NetState::Net => Ok(Self::Net(net::NetConnect::new(cred, client))),
            NetState::Auth4 => Ok(Self::Auth4(auth::AuthConnect::new(cred, client))),
            NetState::Auth6 => Ok(Self::Auth6(auth::AuthConnect::new(cred, client))),
            _ => Err(NetHelperError::HostErr),
        }
    }
}

impl Deref for TUNetConnect {
    type Target = dyn TUNetHelper;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Net(c) => c,
            Self::Auth4(c) => c,
            Self::Auth6(c) => c,
        }
    }
}

impl DerefMut for TUNetConnect {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Net(c) => c,
            Self::Auth4(c) => c,
            Self::Auth6(c) => c,
        }
    }
}

pub fn create_http_client() -> Result<HttpClient> {
    Ok(reqwest::ClientBuilder::new()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()?)
}
