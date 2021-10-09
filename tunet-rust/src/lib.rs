#![forbid(unsafe_code)]

use async_trait::async_trait;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use trait_enum::trait_enum;

pub use anyhow::Result;
pub use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, Timelike};
pub use reqwest::Client as HttpClient;

mod auth;
mod net;
pub mod suggest;
pub mod usereg;

pub use auth::{Auth4Connect, Auth6Connect};
pub use net::NetConnect;

#[derive(Debug, Error)]
pub enum NetHelperError {
    #[error("无法获取 ac_id")]
    NoAcIdErr,
    #[error("操作失败：{0}")]
    LogErr(String),
    #[error("登录状态异常")]
    NoFluxErr,
    #[error("无法识别的用户信息：{0}")]
    ParseFluxErr(String),
    #[error("排序方式无效")]
    OrderErr,
    #[error("无法确定登录方式")]
    HostErr,
}

pub type NetHelperResult<T> = std::result::Result<T, NetHelperError>;

#[derive(Debug, Default)]
pub struct NetCredential {
    pub username: String,
    pub password: String,
    pub ac_ids: RwLock<Vec<i32>>,
}

impl NetCredential {
    pub fn new(username: String, password: String, ac_ids: Vec<i32>) -> Self {
        Self {
            username,
            password,
            ac_ids: RwLock::new(ac_ids),
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
            if s.is_empty() {
                Err(NetHelperError::NoFluxErr)
            } else {
                Err(NetHelperError::ParseFluxErr(s.to_string()))
            }
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
    async fn login(&self) -> Result<String>;
    async fn logout(&self) -> Result<String>;
    async fn flux(&self) -> Result<NetFlux>;
    fn cred(&self) -> Arc<NetCredential>;
}

trait_enum! {
    #[derive(Clone)]
    pub enum TUNetConnect : TUNetHelper {
        NetConnect,
        Auth4Connect,
        Auth6Connect,
    }
}

impl TUNetConnect {
    pub async fn new(
        mut s: NetState,
        cred: Arc<NetCredential>,
        client: HttpClient,
    ) -> NetHelperResult<Self> {
        if let NetState::Auto = s {
            s = suggest::suggest(&client).await;
            debug_assert_ne!(s, NetState::Auto);
        }
        match s {
            NetState::Net => Ok(Self::NetConnect(net::NetConnect::new(cred, client))),
            NetState::Auth4 => Ok(Self::Auth4Connect(auth::AuthConnect::new(cred, client))),
            NetState::Auth6 => Ok(Self::Auth6Connect(auth::AuthConnect::new(cred, client))),
            _ => Err(NetHelperError::HostErr),
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
