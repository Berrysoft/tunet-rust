#![feature(option_result_unwrap_unchecked)]

use std::borrow::Cow;
use std::result;
use std::str;
use std::time;
use thiserror::Error;

pub use reqwest::blocking::Client as HttpClient;

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

#[derive(Debug, Default)]
pub struct NetFlux {
    pub username: String,
    pub flux: u64,
    pub online_time: time::Duration,
    pub balance: f64,
}

impl NetFlux {
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
            NetFlux::default()
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
    JsonErr(#[from] serde_json::error::Error),
    #[error("无法获取ac_id")]
    NoAcIdErr,
    #[error("操作失败`{0}`")]
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

pub type Result<T> = result::Result<T, NetHelperError>;

#[derive(Debug, Clone, Copy)]
pub enum NetState {
    Unknown,
    Net,
    Auth4,
    Auth6,
    Auto,
}

impl str::FromStr for NetState {
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
    Auth(auth::AuthConnect<'a, 's>),
}

impl<'a, 's> TUNetConnect<'a, 's> {
    pub fn login(&mut self) -> Result<String> {
        match self {
            Self::Net(c) => c.login(),
            Self::Auth(c) => c.login(),
        }
    }
    pub fn logout(&mut self) -> Result<String> {
        match self {
            Self::Net(c) => c.logout(),
            Self::Auth(c) => c.logout(),
        }
    }
    pub fn flux(&self) -> Result<NetFlux> {
        match self {
            Self::Net(c) => c.flux(),
            Self::Auth(c) => c.flux(),
        }
    }
    pub fn ac_ids(&self) -> &[i32] {
        match self {
            Self::Net(_) => &[0; 0],
            Self::Auth(c) => c.ac_ids(),
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
            NetState::Auth4 => Ok(TUNetConnect::Auth(auth::AuthConnect::from_cred_client(
                u, p, client, ac_ids,
            ))),
            NetState::Auth6 => Ok(TUNetConnect::Auth(auth::AuthConnect::from_cred_client_v6(
                u, p, client, ac_ids,
            ))),
            NetState::Auto => {
                Self::from_state_cred_client(suggest::suggest(client), u, p, client, ac_ids)
            }
            NetState::Unknown => Err(NetHelperError::HostErr),
        }
    }
}

pub fn create_http_client(proxy: bool) -> Result<HttpClient> {
    Ok(if proxy {
        HttpClient::builder().cookie_store(true).build()?
    } else {
        HttpClient::builder()
            .cookie_store(true)
            .no_proxy()
            .build()?
    })
}
