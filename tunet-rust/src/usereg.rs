use crate::*;
use async_stream::try_stream;
use chrono::Local;
use data_encoding::HEXLOWER;
use futures_core::Stream;
use mac_address::MacAddress;
use md5::{Digest, Md5};
use select::document::Document;
use select::predicate::*;
use std::net::Ipv4Addr;
use url::Url;

pub struct NetUser {
    pub address: Ipv4Addr,
    pub login_time: NaiveDateTime,
    pub mac_address: Option<MacAddress>,
}

impl NetUser {
    pub fn from_detail(a: Ipv4Addr, t: NaiveDateTime, m: Option<MacAddress>) -> Self {
        NetUser {
            address: a,
            login_time: t,
            mac_address: m,
        }
    }
}

pub struct NetDetail {
    pub login_time: NaiveDateTime,
    pub logout_time: NaiveDateTime,
    pub flux: Flux,
}

impl NetDetail {
    pub fn from_detail(i: NaiveDateTime, o: NaiveDateTime, f: Flux) -> Self {
        NetDetail {
            login_time: i,
            logout_time: o,
            flux: f,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NetDetailOrder {
    LoginTime,
    LogoutTime,
    Flux,
}

impl NetDetailOrder {
    fn get_query(&self) -> &'static str {
        match self {
            NetDetailOrder::LoginTime => "user_login_time",
            NetDetailOrder::LogoutTime => "user_drop_time",
            NetDetailOrder::Flux => "user_in_bytes",
        }
    }
}

impl std::str::FromStr for NetDetailOrder {
    type Err = NetHelperError;
    fn from_str(s: &str) -> Result<Self> {
        if s.eq_ignore_ascii_case("login") || s.eq_ignore_ascii_case("logintime") {
            Ok(NetDetailOrder::LoginTime)
        } else if s.eq_ignore_ascii_case("logout") || s.eq_ignore_ascii_case("logouttime") {
            Ok(NetDetailOrder::LogoutTime)
        } else if s.eq_ignore_ascii_case("flux") {
            Ok(NetDetailOrder::Flux)
        } else {
            Err(NetHelperError::OrderErr)
        }
    }
}

pub struct UseregHelper {
    cred: NetCredential,
    client: HttpClient,
}

// Use HTTP because TLS1.0/1.1 aren't supported.
static USEREG_LOG_URI: &str = "http://usereg.tsinghua.edu.cn/do.php";
static USEREG_INFO_URI: &str = "http://usereg.tsinghua.edu.cn/online_user_ipv4.php";
static USEREG_CONNECT_URI: &str = "http://usereg.tsinghua.edu.cn/ip_login.php";
static USEREG_DETAIL_URI: &str = "http://usereg.tsinghua.edu.cn/user_detail_list.php";
static DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const USEREG_OFF: usize = 100;

impl UseregHelper {
    pub fn new(cred: NetCredential, client: HttpClient) -> Self {
        UseregHelper { cred, client }
    }

    pub async fn login(&mut self) -> Result<String> {
        let password_md5 = {
            let mut md5 = Md5::new();
            md5.update(self.cred.password.as_bytes());
            md5.finalize()
        };
        let params = [
            ("action", "login"),
            ("user_login_name", &self.cred.username),
            ("user_password", &HEXLOWER.encode(&password_md5)),
        ];
        let res = self
            .client
            .post(USEREG_LOG_URI)
            .form(&params)
            .send()
            .await?;
        Ok(res.text().await?)
    }

    pub async fn logout(&mut self) -> Result<String> {
        let params = [("action", "logout")];
        let res = self
            .client
            .post(USEREG_LOG_URI)
            .form(&params)
            .send()
            .await?;
        Ok(res.text().await?)
    }

    pub fn cred(&self) -> &NetCredential {
        &self.cred
    }

    pub async fn connect(&self, addr: Ipv4Addr) -> Result<String> {
        let params = [
            ("n", "100"),
            ("is_pad", "1"),
            ("type", "1"),
            ("action", "do_login"),
            ("user_ip", &addr.to_string()),
            ("drop", "0"),
        ];
        let res = self
            .client
            .post(USEREG_CONNECT_URI)
            .form(&params)
            .send()
            .await?;
        Ok(res.text().await?)
    }

    pub async fn drop(&self, addr: Ipv4Addr) -> Result<String> {
        let params = [("action", "drop"), ("user_ip", &addr.to_string())];
        let res = self
            .client
            .post(USEREG_INFO_URI)
            .form(&params)
            .send()
            .await?;
        Ok(res.text().await?)
    }

    pub fn users(&self) -> impl Stream<Item = Result<NetUser>> {
        let client = self.client.clone();
        try_stream! {
            let res = client.get(USEREG_INFO_URI).send().await?;
            let doc = Document::from(res.text().await?.as_str());
            for tds in doc
                .find(Name("tr").descendant(Attr("align", "center")))
                .skip(1)
                .map(|node| node.find(Name("td")).skip(1).collect::<Vec<_>>())
            {
                yield NetUser::from_detail(
                    tds[0]
                        .text()
                        .parse()
                        .unwrap_or_else(|_| Ipv4Addr::new(0, 0, 0, 0)),
                    NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT)
                        .unwrap_or_else(|_| NaiveDateTime::from_timestamp(0, 0)),
                    tds[6].text().parse().ok(),
                );
            }
        }
    }

    pub fn details(&self, o: NetDetailOrder, des: bool) -> impl Stream<Item = Result<NetDetail>> {
        let client = self.client.clone();
        let now = Local::now();
        let start_time = now.format("%Y-%m-01").to_string();
        let end_time = now.format("%Y-%m-%d").to_string();
        let des = if des { "DESC" } else { "" };
        try_stream! {
            for i in 1usize.. {
                let uri = Url::parse_with_params(
                    USEREG_DETAIL_URI,
                    &[
                        ("action", "query"),
                        ("desc", des),
                        ("order", o.get_query()),
                        ("start_time", &start_time),
                        ("end_time", &end_time),
                        ("page", &i.to_string()),
                        ("offset", &USEREG_OFF.to_string()),
                    ],
                )
                .unwrap();
                let res = client.get(uri).send().await?;
                let doc = Document::from(res.text().await?.as_str());
                let mut new_len = 0;
                for node in doc
                    .find(Name("tr").descendant(Attr("align", "center")))
                    .skip(1)
                {
                    let tds = node.find(Name("td")).skip(1).collect::<Vec<_>>();
                    if !tds.is_empty() {
                        yield NetDetail::from_detail(
                            NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT)
                                .unwrap_or_else(|_| NaiveDateTime::from_timestamp(0, 0)),
                            NaiveDateTime::parse_from_str(&tds[2].text(), DATE_TIME_FORMAT)
                                .unwrap_or_else(|_| NaiveDateTime::from_timestamp(0, 0)),
                            tds[4].text().parse().unwrap_or_default(),
                            );
                        new_len += 1;
                    }
                }
                if new_len < USEREG_OFF {
                    break;
                }
            }
        }
    }
}
