use super::*;
use chrono::prelude::*;
use crypto::digest::Digest;
use crypto::md5::Md5;
use hwaddr::HwAddr;
use reqwest::blocking::Client;
use select::document::Document;
use select::predicate::*;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::string::String;
use std::vec::Vec;

pub struct NetUser {
    pub address: Ipv4Addr,
    pub login_time: NaiveDateTime,
    pub mac_address: HwAddr,
}

impl NetUser {
    pub fn from_detail(a: Ipv4Addr, t: NaiveDateTime, m: HwAddr) -> Self {
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

#[derive(Debug)]
pub enum NetDetailOrder {
    LoginTime,
    LogoutTime,
    Flux,
}

impl NetDetailOrder {
    fn get_query(&self) -> String {
        String::from(match self {
            NetDetailOrder::LoginTime => "user_login_time",
            NetDetailOrder::LogoutTime => "user_drop_time",
            NetDetailOrder::Flux => "user_in_bytes",
        })
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
    client: &'a Client,
}

const USEREG_LOG_URI: &'static str = "https://usereg.tsinghua.edu.cn/do.php";
const USEREG_INFO_URI: &'static str = "https://usereg.tsinghua.edu.cn/online_user_ipv4.php";
const USEREG_CONNECT_URI: &'static str = "https://usereg.tsinghua.edu.cn/ip_login.php";
const DATE_TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

fn parse_flux(s: &str) -> u64 {
    let mut flux = s[0..s.len() - 1].parse::<f64>().unwrap_or_default();
    flux *= match s.as_bytes()[s.len() - 1] {
        b'G' => 1000.0 * 1000.0 * 1000.0,
        b'M' => 1000.0 * 1000.0,
        b'K' => 1000.0,
        _ => 1.0,
    };
    flux as u64
}

impl<'a, 's> UseregHelper<'a, 's> {
    pub fn from_cred_client<S: Into<Cow<'s, str>>>(u: S, p: S, client: &'a Client) -> Self {
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
        let doc = Document::from(&res.text()? as &str);
        Ok(doc
            .find(Name("tr").descendant(Attr("align", "center")))
            .skip(1)
            .map(|node| {
                let tds = node.find(Name("td")).skip(1).collect::<Vec<_>>();
                NetUser::from_detail(
                    Ipv4Addr::from_str(&tds[0].text()).unwrap(),
                    NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT).unwrap(),
                    tds[10].text().parse::<HwAddr>().unwrap(),
                )
            })
            .collect::<Vec<_>>())
    }

    pub fn details(&self, o: NetDetailOrder, des: bool) -> Result<Vec<NetDetail>> {
        let now = Local::now();
        let off = 100;
        let mut list: Vec<NetDetail> = Vec::new();
        let mut i = 1;
        loop {
            let res = self.client.get(&format!("https://usereg.tsinghua.edu.cn/user_detail_list.php?action=query&desc={6}&order={5}&start_time={0}-{1:02}-01&end_time={0}-{1:02}-{2:02}&page={3}&offset={4}", now.year(), now.month(), now.day(), i, off, o.get_query(), if des { "DESC" } else { "" },)).send()?;
            let doc = Document::from(&res.text()? as &str);
            let mut ds = doc
                .find(Name("tr").descendant(Attr("align", "center")))
                .skip(1)
                .filter_map(|node| {
                    let tds = node.find(Name("td")).skip(1).collect::<Vec<_>>();
                    if tds.len() == 0 {
                        None
                    } else {
                        Some(NetDetail::from_detail(
                            NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT)
                                .unwrap(),
                            NaiveDateTime::parse_from_str(&tds[2].text(), DATE_TIME_FORMAT)
                                .unwrap(),
                            parse_flux(&tds[4].text()),
                        ))
                    }
                })
                .collect::<Vec<_>>();
            let new_len = ds.len();
            list.append(&mut ds);
            if new_len < off {
                break;
            }
            i += 1;
        }
        Ok(list)
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
