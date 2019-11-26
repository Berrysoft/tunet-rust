use super::*;
use chrono::prelude::*;
use crypto::digest::Digest;
use crypto::md5::Md5;
use reqwest::Client;
use select::document::Document;
use select::predicate::*;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::string::String;
use std::vec::Vec;

pub struct NetUser {
    pub address: Ipv4Addr,
    pub login_time: NaiveDateTime,
    pub client: String,
}

impl NetUser {
    pub fn from_detail(a: Ipv4Addr, t: NaiveDateTime, c: String) -> Self {
        NetUser { address: a, login_time: t, client: c }
    }
}

pub struct NetDetail {
    pub login_time: NaiveDateTime,
    pub logout_time: NaiveDateTime,
    pub flux: u64,
}

impl NetDetail {
    pub fn from_detail(i: NaiveDateTime, o: NaiveDateTime, f: u64) -> Self {
        NetDetail { login_time: i, logout_time: o, flux: f }
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
    type Err = String;
    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let ls = s.to_lowercase();
        if ls == "login" || ls == "logintime" {
            Ok(NetDetailOrder::LoginTime)
        } else if ls == "logout" || ls == "logouttime" {
            Ok(NetDetailOrder::LogoutTime)
        } else if ls == "flux" {
            Ok(NetDetailOrder::Flux)
        } else {
            Err("排序方式错误".to_string())
        }
    }
}

pub struct UseregHelper {
    credential: NetCredential,
    client: Client,
}

const USEREG_LOG_URI: &'static str = "https://usereg.tsinghua.edu.cn/do.php";
const USEREG_INFO_URI: &'static str = "https://usereg.tsinghua.edu.cn/online_user_ipv4.php";
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

impl UseregHelper {
    pub fn from_cred(u: String, p: String, proxy: bool) -> Result<Self> {
        let client = if proxy { Client::builder().cookie_store(true).build()? } else { Client::builder().cookie_store(true).no_proxy().build()? };
        Ok(UseregHelper { credential: NetCredential::from_cred(u, p), client })
    }

    pub fn drop(&self, addr: Ipv4Addr) -> Result<String> {
        let params = [("action", "drop"), ("user_ip", &addr.to_string())];
        let mut res = self.client.post(USEREG_INFO_URI).form(&params).send()?;
        Ok(res.text()?)
    }

    pub fn users(&self) -> Result<Vec<NetUser>> {
        let mut res = self.client.get(USEREG_INFO_URI).send()?;
        let doc = Document::from(&res.text()? as &str);
        Ok(doc
            .find(Name("tr").descendant(Attr("align", "center")))
            .skip(1)
            .map(|node| {
                let tds = node.find(Name("td")).skip(1).collect::<Vec<_>>();
                NetUser::from_detail(Ipv4Addr::from_str(&tds[0].text()).unwrap(), NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT).unwrap(), tds[10].text())
            })
            .collect::<Vec<_>>())
    }

    pub fn details(&self, o: NetDetailOrder, des: bool) -> Result<Vec<NetDetail>> {
        let now = Local::now();
        let off = 100;
        let mut list: Vec<NetDetail> = Vec::new();
        let mut i = 1;
        loop {
            let mut res = self.client.get(&format!("https://usereg.tsinghua.edu.cn/user_detail_list.php?action=query&desc={6}&order={5}&start_time={0}-{1:02}-01&end_time={0}-{1:02}-{2:02}&page={3}&offset={4}", now.year(), now.month(), now.day(), i, off, o.get_query(), if des { "DESC" } else { "" },)).send()?;
            let doc = Document::from(&res.text()? as &str);
            let mut ds = doc
                .find(Name("tr").descendant(Attr("align", "center")))
                .skip(1)
                .filter_map(|node| {
                    let tds = node.find(Name("td")).skip(1).collect::<Vec<_>>();
                    if tds.len() == 0 {
                        None
                    } else {
                        Some(NetDetail::from_detail(NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT).unwrap(), NaiveDateTime::parse_from_str(&tds[2].text(), DATE_TIME_FORMAT).unwrap(), parse_flux(&tds[4].text())))
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

impl NetHelper for UseregHelper {
    fn login(&self) -> Result<String> {
        let mut cry = Md5::new();
        cry.input_str(&self.credential.password);
        let params = [("action", "login"), ("user_login_name", &self.credential.username), ("user_password", &cry.result_str())];
        let mut res = self.client.post(USEREG_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
    fn logout(&self) -> Result<String> {
        let params = [("action", "logout")];
        let mut res = self.client.post(USEREG_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
}
