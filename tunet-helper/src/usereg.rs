use crate::*;
use async_stream::try_stream;
use data_encoding::HEXLOWER;
use futures_core::Stream;
use mac_address2::MacAddress;
use md5::{Digest, Md5};
use select::document::Document;
use select::predicate::*;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::Deref;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NetDateTime(pub NaiveDateTime);

impl FromStr for NetDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> chrono::ParseResult<Self> {
        NaiveDateTime::parse_from_str(s, DATE_TIME_FORMAT).map(Self)
    }
}

impl Display for NetDateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <NaiveDateTime as Display>::fmt(&self.0, f)
    }
}

impl Deref for NetDateTime {
    type Target = NaiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NetUser {
    pub address: Ipv4Addr,
    pub address_v6: Ipv6Addr,
    pub login_time: NetDateTime,
    pub mac_address: Option<MacAddress>,
    pub flux: Flux,
}

impl NetUser {
    pub fn from_detail(
        a: Ipv4Addr,
        a6: Ipv6Addr,
        t: NetDateTime,
        m: Option<MacAddress>,
        f: Flux,
    ) -> Self {
        NetUser {
            address: a,
            address_v6: a6,
            login_time: t,
            mac_address: m,
            flux: f,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NetDetail {
    pub login_time: NetDateTime,
    pub logout_time: NetDateTime,
    pub flux: Flux,
}

impl NetDetail {
    pub fn from_detail(i: NetDateTime, o: NetDateTime, f: Flux) -> Self {
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
    fn from_str(s: &str) -> NetHelperResult<Self> {
        if s.eq_ignore_ascii_case("login") || s.eq_ignore_ascii_case("logintime") {
            Ok(NetDetailOrder::LoginTime)
        } else if s.eq_ignore_ascii_case("logout") || s.eq_ignore_ascii_case("logouttime") {
            Ok(NetDetailOrder::LogoutTime)
        } else if s.eq_ignore_ascii_case("flux") {
            Ok(NetDetailOrder::Flux)
        } else {
            Err(NetHelperError::InvalidOrder)
        }
    }
}

#[derive(Clone)]
pub struct UseregHelper {
    client: HttpClient,
}

static USEREG_LOG_URI: &str = "https://usereg.tsinghua.edu.cn/do.php";
static USEREG_INFO_URI: &str = "https://usereg.tsinghua.edu.cn/import_online_user.php";
static USEREG_INFO_URI2: &str = "https://usereg.tsinghua.edu.cn/1x_online_user.php";
static USEREG_CONNECT_URI: &str = "https://usereg.tsinghua.edu.cn/ip_login.php";
static USEREG_DETAIL_URI: &str = "https://usereg.tsinghua.edu.cn/user_detail_list.php";
static DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const USEREG_OFF: usize = 1000;

impl UseregHelper {
    pub fn new(client: HttpClient) -> Self {
        UseregHelper { client }
    }

    pub async fn login(&self, u: &str, p: &str) -> NetHelperResult<String> {
        let password_md5 = {
            let mut md5 = Md5::new();
            md5.update(p.as_bytes());
            md5.finalize()
        };
        let params = [
            ("action", "login"),
            ("user_login_name", u),
            ("user_password", &HEXLOWER.encode(&password_md5)),
        ];
        let res = self
            .client
            .post(USEREG_LOG_URI)?
            .form(&params)?
            .send()
            .await?;
        let text = res.text().await?;
        if text == "ok" {
            Ok(text)
        } else {
            Err(NetHelperError::Log(text))
        }
    }

    pub async fn logout(&self) -> NetHelperResult<String> {
        let params = [("action", "logout")];
        let res = self
            .client
            .post(USEREG_LOG_URI)?
            .form(&params)?
            .send()
            .await?;
        Ok(res.text().await?)
    }

    pub async fn connect(&self, addr: Ipv4Addr) -> NetHelperResult<String> {
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
            .post(USEREG_CONNECT_URI)?
            .form(&params)?
            .send()
            .await?;
        Ok(res.text().await?)
    }

    pub async fn drop(&self, addr: Ipv4Addr) -> NetHelperResult<String> {
        let params = [("action", "drop"), ("user_ip", &addr.to_string())];
        let res = self
            .client
            .post(USEREG_INFO_URI)?
            .form(&params)?
            .send()
            .await?;
        Ok(res.text().await?)
    }

    pub fn users(&self) -> impl Stream<Item = NetHelperResult<NetUser>> {
        let client = self.client.clone();
        try_stream! {
            for uri in [USEREG_INFO_URI, USEREG_INFO_URI2] {
                let res = client.get(uri)?.send().await?;
                let doc = Document::from(res.text().await?.as_str());
                let doc = doc
                    .find(Name("tr").descendant(Attr("align", "center")))
                    .skip(1)
                    .map(|node| node.find(Name("td")).skip(1).map(|n| n.text()).collect::<Vec<_>>());
                for tds in doc {
                    if let Ok(ip) = tds[0].parse() {
                        yield NetUser::from_detail(
                            ip,
                            tds[1]
                                .parse()
                                .unwrap_or_else(|_| Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)),
                            tds[2].parse().unwrap_or_default(),
                            tds[7].parse().ok(),
                            tds[3].parse().unwrap_or_default(),
                        );
                    }
                }
            }
        }
    }

    pub fn details(
        &self,
        o: NetDetailOrder,
        des: bool,
    ) -> impl Stream<Item = NetHelperResult<NetDetail>> {
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
                let res = client.get(uri)?.send().await?;
                let doc = Document::from(res.text().await?.as_str());
                let doc = doc
                        .find(Name("tr").descendant(Attr("align", "center")))
                        .skip(1)
                        .map(|node| node.find(Name("td")).skip(1).map(|n| n.text()).collect::<Vec<_>>());
                let mut new_len = 0;
                for tds in doc {
                    if !tds.is_empty() {
                        yield NetDetail::from_detail(
                            tds[1].parse().unwrap_or_default(),
                            tds[2].parse().unwrap_or_default(),
                            tds[4].parse().unwrap_or_default(),
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

#[cfg(feature = "dart")]
mod impl_dart {
    use super::*;
    use allo_isolate::{ffi::DartCObject, IntoDart, IntoDartExceptPrimitive};

    impl IntoDart for NetDateTime {
        fn into_dart(self) -> DartCObject {
            vec![self.0.into_dart()].into_dart()
        }
    }

    impl IntoDartExceptPrimitive for NetDateTime {}

    impl IntoDart for NetDetail {
        fn into_dart(self) -> DartCObject {
            vec![
                self.login_time.into_dart(),
                self.logout_time.into_dart(),
                self.flux.into_dart(),
            ]
            .into_dart()
        }
    }

    impl IntoDartExceptPrimitive for NetDetail {}
}
