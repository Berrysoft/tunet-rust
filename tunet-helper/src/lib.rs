#![forbid(unsafe_code)]

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use std::fmt::{Display, Formatter};
use thiserror::Error;

pub use chrono::{
    DateTime, Datelike, Duration as NaiveDuration, FixedOffset, Local, NaiveDate, NaiveDateTime,
    Timelike,
};
pub use reqwest::Client as HttpClient;

mod auth;
mod net;
pub mod suggest;
pub mod usereg;

pub use auth::{Auth4Connect, Auth6Connect};
pub use net::NetConnect;

#[derive(Debug, Error)]
pub enum NetHelperError {
    #[error("操作失败：{0}")]
    Log(String),
    #[error("登录状态异常")]
    NoFlux,
    #[error("无法识别的用户信息：{0}")]
    InvalidFlux(String),
    #[error("排序方式无效")]
    InvalidOrder,
    #[error("无法确定登录方式")]
    InvalidHost,
    #[error("网络请求错误：{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON 解析错误：{0}")]
    Json(#[from] serde_json::Error),
}

pub type NetHelperResult<T> = Result<T, NetHelperError>;

#[repr(transparent)]
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Flux(pub u64);

impl Flux {
    pub fn from_gb(f: f64) -> Self {
        Self((f * 1_000_000_000.) as u64)
    }

    pub fn to_gb(self) -> f64 {
        self.0 as f64 / 1_000_000_000.
    }

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

#[derive(Debug, Clone)]
pub struct Duration(pub NaiveDuration);

impl Default for Duration {
    fn default() -> Self {
        Self(NaiveDuration::zero())
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let total_sec = self.0.num_seconds();
        let (total_min, sec) = (total_sec / 60, total_sec % 60);
        let (total_h, min) = (total_min / 60, total_min % 60);
        let (day, h) = (total_h / 24, total_h % 24);
        let str = if day != 0 {
            format!("{}.{:02}:{:02}:{:02}", day, h, min, sec)
        } else {
            format!("{:02}:{:02}:{:02}", h, min, sec)
        };
        f.pad(&str)
    }
}

#[repr(transparent)]
#[derive(Debug, Default, PartialEq, PartialOrd, Clone, Copy)]
pub struct Balance(pub f64);

impl Display for Balance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "¥{:.2}", self.0)
    }
}

#[derive(Debug, Default, Clone)]
pub struct NetFlux {
    pub username: String,
    pub flux: Flux,
    pub online_time: Duration,
    pub balance: Balance,
}

impl std::str::FromStr for NetFlux {
    type Err = NetHelperError;
    fn from_str(s: &str) -> NetHelperResult<Self> {
        let vec = s.split(',').collect::<Vec<_>>();
        if vec.len() >= 12 {
            Ok(NetFlux {
                username: vec[0].to_string(),
                flux: Flux(vec[6].parse::<u64>().unwrap_or_default()),
                online_time: Duration(NaiveDuration::seconds(
                    (vec[2].parse::<i64>().unwrap_or_default()
                        - vec[1].parse::<i64>().unwrap_or_default())
                    .max(0),
                )),
                balance: Balance(vec[11].parse::<f64>().unwrap_or_default()),
            })
        } else if s.is_empty() {
            Err(NetHelperError::NoFlux)
        } else {
            Err(NetHelperError::InvalidFlux(s.to_string()))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetState {
    Unknown,
    Net,
    Auth4,
    Auth6,
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
        } else {
            Err(NetHelperError::InvalidHost)
        }
    }
}

#[async_trait]
#[enum_dispatch(TUNetConnect)]
pub trait TUNetHelper: Send + Sync {
    async fn login(&self, u: &str, p: &str) -> NetHelperResult<String>;
    async fn logout(&self, u: &str) -> NetHelperResult<String>;
    async fn flux(&self) -> NetHelperResult<NetFlux>;
}

#[enum_dispatch]
#[derive(Clone)]
pub enum TUNetConnect {
    NetConnect,
    Auth4Connect,
    Auth6Connect,
}

impl TUNetConnect {
    pub fn new(s: NetState, client: HttpClient) -> NetHelperResult<TUNetConnect> {
        match s {
            NetState::Net => Ok(Self::NetConnect(net::NetConnect::new(client))),
            NetState::Auth4 => Ok(Self::Auth4Connect(auth::AuthConnect::new(client))),
            NetState::Auth6 => Ok(Self::Auth6Connect(auth::AuthConnect::new(client))),
            _ => Err(NetHelperError::InvalidHost),
        }
    }

    pub async fn new_with_suggest(
        s: Option<NetState>,
        client: HttpClient,
    ) -> NetHelperResult<TUNetConnect> {
        match s {
            None => {
                let s = suggest::suggest(&client).await;
                Self::new(s, client)
            }
            Some(s) => Self::new(s, client),
        }
    }
}

pub fn create_http_client() -> NetHelperResult<HttpClient> {
    Ok(reqwest::ClientBuilder::new()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()?)
}
