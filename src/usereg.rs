use super::*;
use chrono::naive::NaiveDateTime;
use crypto::digest::Digest;
use crypto::md5::Md5;
use reqwest::Client;
use select::document::Document;
use select::predicate::*;
use std::net;
use std::str::FromStr;
use std::string::String;
use std::vec::Vec;

pub struct NetUser {
    pub address: net::Ipv4Addr,
    pub login_time: NaiveDateTime,
    pub client: String,
}

impl NetUser {
    fn from_detail(a: net::Ipv4Addr, t: NaiveDateTime, c: String) -> Self {
        NetUser {
            address: a,
            login_time: t,
            client: c,
        }
    }
}

pub struct NetDetail {
    pub login_time: NaiveDateTime,
    pub logout_time: NaiveDateTime,
    pub flux: u64,
}

pub struct UseregHelper {
    credential: NetCredential,
    client: Client,
}

const USEREG_LOG_URI: &'static str = "https://usereg.tsinghua.edu.cn/do.php";
const USEREG_INFO_URI: &'static str = "https://usereg.tsinghua.edu.cn/online_user_ipv4.php";

impl UseregHelper {
    pub fn from_cred(u: String, p: String) -> Result<Self> {
        Ok(UseregHelper {
            credential: NetCredential::from_cred(u, p),
            client: Client::builder().cookie_store(true).build()?,
        })
    }

    pub fn drop(&self, addr: net::Ipv4Addr) -> Result<String> {
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
                NetUser::from_detail(
                    net::Ipv4Addr::from_str(&tds[0].text()).unwrap(),
                    NaiveDateTime::parse_from_str(&tds[1].text(), "%Y-%m-%d %H:%M:%S").unwrap(),
                    tds[10].text(),
                )
            })
            .collect::<Vec<_>>())
    }
}

impl NetHelper for UseregHelper {
    fn login(&self) -> Result<String> {
        let mut cry = Md5::new();
        cry.input_str(&self.credential.password);
        let params = [
            ("action", "login"),
            ("user_login_name", &self.credential.username),
            ("user_password", &cry.result_str()),
        ];
        let mut res = self.client.post(USEREG_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
    fn logout(&self) -> Result<String> {
        let params = [("action", "logout")];
        let mut res = self.client.post(USEREG_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
}
