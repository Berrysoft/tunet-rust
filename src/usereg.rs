use super::*;
use crypto::digest::Digest;
use crypto::md5::Md5;
use reqwest::Client;
use std::net;
use std::string::String;
use std::time;

pub struct NetUser {
    pub address: net::Ipv4Addr,
    pub login_time: time::SystemTime,
    pub client: String,
}

pub struct NetDetail {
    pub login_time: time::SystemTime,
    pub logout_time: time::SystemTime,
    pub flux: u64,
}

pub struct UseregHelper {
    credential: NetCredential,
    client: Client,
}

const USEREG_LOG_URI: &'static str = "https://usereg.tsinghua.edu.cn/do.php";
const USEREG_INFO_URI: &'static str = "https://usereg.tsinghua.edu.cn/online_user_ipv4.php";

impl UseregHelper {
    pub fn new() -> Self {
        UseregHelper::from_cred(String::new(), String::new())
    }

    pub fn from_cred(u: String, p: String) -> Self {
        UseregHelper {
            credential: NetCredential::from_cred(u, p),
            client: Client::new(),
        }
    }

    pub fn drop(&self, addr: net::Ipv4Addr) -> Result<String> {
        let params = [("action", "drop"), ("user_ip", &addr.to_string())];
        let mut res = self.client.post(USEREG_INFO_URI).form(&params).send()?;
        Ok(res.text()?)
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
