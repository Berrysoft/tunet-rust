use super::*;
use chrono::prelude::*;
use crypto::digest::Digest;
use crypto::md5::Md5;
use mac_address::MacAddress;
use select::document::Document;
use select::predicate::*;
use std::collections::VecDeque;
use std::net::Ipv4Addr;

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
    pub flux: u64,
}

impl NetDetail {
    pub fn from_detail(i: NaiveDateTime, o: NaiveDateTime, f: u64) -> Self {
        NetDetail {
            login_time: i,
            logout_time: o,
            flux: f,
        }
    }
}

#[repr(C)]
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

impl str::FromStr for NetDetailOrder {
    type Err = NetHelperError;
    fn from_str(s: &str) -> Result<Self> {
        let ls = s.to_lowercase();
        if ls == "login" || ls == "logintime" {
            Ok(NetDetailOrder::LoginTime)
        } else if ls == "logout" || ls == "logouttime" {
            Ok(NetDetailOrder::LogoutTime)
        } else if ls == "flux" {
            Ok(NetDetailOrder::Flux)
        } else {
            Err(NetHelperError::OrderError)
        }
    }
}

pub struct UseregHelper<'a, 's> {
    credential: NetCredential<'s>,
    client: &'a HttpClient,
}

static USEREG_LOG_URI: &str = "https://usereg.tsinghua.edu.cn/do.php";
static USEREG_INFO_URI: &str = "https://usereg.tsinghua.edu.cn/online_user_ipv4.php";
static USEREG_CONNECT_URI: &str = "https://usereg.tsinghua.edu.cn/ip_login.php";
static DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

fn parse_flux(s: &str) -> u64 {
    let (flux, unit) = s.split_at(s.len() - 1);
    (flux.parse::<f64>().unwrap_or_default()
        * match unit {
            "G" => 1_000_000_000.0,
            "M" => 1_000_000.0,
            "K" => 1_000.0,
            _ => 1.0,
        }) as u64
}

impl<'a, 's> UseregHelper<'a, 's> {
    pub fn from_cred_client<S: Into<Cow<'s, str>>>(u: S, p: S, client: &'a HttpClient) -> Self {
        UseregHelper {
            credential: NetCredential::from_cred(u, p),
            client,
        }
    }

    pub fn connect(&self, addr: Ipv4Addr) -> Result<String> {
        let params = [
            ("n", "100"),
            ("is_pad", "1"),
            ("type", "1"),
            ("action", "do_login"),
            ("user_ip", &addr.to_string()),
            ("drop", "0"),
        ];
        let res = self.client.post(USEREG_CONNECT_URI).form(&params).send()?;
        Ok(res.text()?)
    }

    pub fn drop(&self, addr: Ipv4Addr) -> Result<String> {
        let params = [("action", "drop"), ("user_ip", &addr.to_string())];
        let res = self.client.post(USEREG_INFO_URI).form(&params).send()?;
        Ok(res.text()?)
    }

    pub fn users(&self) -> Result<Vec<NetUser>> {
        let res = self.client.get(USEREG_INFO_URI).send()?;
        let doc = Document::from(res.text()?.as_str());
        Ok(doc
            .find(Name("tr").descendant(Attr("align", "center")))
            .skip(1)
            .map(|node| {
                let tds = node.find(Name("td")).skip(1).collect::<Vec<_>>();
                NetUser::from_detail(
                    tds[0].text().parse().unwrap_or(Ipv4Addr::new(0, 0, 0, 0)),
                    NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT)
                        .unwrap_or(NaiveDateTime::from_timestamp(0, 0)),
                    tds[6].text().parse().ok(),
                )
            })
            .collect())
    }

    pub fn details(&self, o: NetDetailOrder, des: bool) -> Result<UseregDetails<'a>> {
        Ok(UseregDetails::new(self.client, o, des))
    }
}

impl<'a, 's> NetHelper for UseregHelper<'a, 's> {
    fn login(&mut self) -> Result<String> {
        let mut cry = Md5::new();
        cry.input_str(&self.credential.password);
        let params = [
            ("action", "login"),
            ("user_login_name", &self.credential.username),
            ("user_password", &cry.result_str()),
        ];
        let res = self.client.post(USEREG_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
    fn logout(&mut self) -> Result<String> {
        let params = [("action", "logout")];
        let res = self.client.post(USEREG_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
}

const USEREG_OFF: usize = 100;

pub struct UseregDetails<'a> {
    client: &'a HttpClient,
    index: usize,
    now: DateTime<Local>,
    order: &'static str,
    des: &'static str,
    len: usize,
    data: VecDeque<NetDetail>,
}

impl<'a> UseregDetails<'a> {
    pub(crate) fn new(client: &'a HttpClient, order: NetDetailOrder, des: bool) -> Self {
        Self {
            client,
            index: 0,
            now: Local::now(),
            order: order.get_query(),
            des: if des { "DESC" } else { "" },
            len: USEREG_OFF,
            data: VecDeque::new(),
        }
    }

    fn load_newpage(&mut self) -> Result<()> {
        let res = self.client.get(
                    &format!("https://usereg.tsinghua.edu.cn/user_detail_list.php?action=query&desc={6}&order={5}&start_time={0}-{1:02}-01&end_time={0}-{1:02}-{2:02}&page={3}&offset={4}",
                        self.now.year(), self.now.month(), self.now.day(), self.index, USEREG_OFF, self.order, self.des))
                    .send()?;
        let doc = Document::from(res.text()?.as_str());
        self.data = doc
            .find(Name("tr").descendant(Attr("align", "center")))
            .skip(1)
            .filter_map(|node| {
                let tds = node.find(Name("td")).skip(1).collect::<Vec<_>>();
                if tds.is_empty() {
                    None
                } else {
                    Some(NetDetail::from_detail(
                        NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT)
                            .unwrap_or(NaiveDateTime::from_timestamp(0, 0)),
                        NaiveDateTime::parse_from_str(&tds[2].text(), DATE_TIME_FORMAT)
                            .unwrap_or(NaiveDateTime::from_timestamp(0, 0)),
                        parse_flux(&tds[4].text()),
                    ))
                }
            })
            .collect();
        self.len = self.data.len();
        self.index += 1;
        Ok(())
    }
}

impl Iterator for UseregDetails<'_> {
    type Item = Result<NetDetail>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.data.pop_front() {
            Some(Ok(item))
        } else {
            if self.len < USEREG_OFF {
                None
            } else {
                if let Some(err) = self.load_newpage().err() {
                    self.len = 0;
                    Some(Err(err))
                } else {
                    self.next()
                }
            }
        }
    }
}
