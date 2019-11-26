use super::*;
use crypto::digest::Digest;
use crypto::md5::Md5;
use reqwest::Client;
use std::string::String;

pub struct NetConnect<'a> {
    credential: NetCredential,
    client: &'a Client,
}

const NET_LOG_URI: &'static str = "http://net.tsinghua.edu.cn/do_login.php";
const NET_FLUX_URI: &'static str = "http://net.tsinghua.edu.cn/rad_user_info.php";

impl<'a> NetConnect<'a> {
    pub fn from_cred_client(u: String, p: String, client: &'a Client) -> Self {
        NetConnect { credential: NetCredential::from_cred(u, p), client }
    }
}

impl<'a> NetHelper for NetConnect<'a> {
    fn login(&self) -> Result<String> {
        let mut cry = Md5::new();
        cry.input_str(&self.credential.password);
        let password_md5 = "{MD5_HEX}".to_owned() + &cry.result_str();
        let params = [("action", "login"), ("ac_id", "1"), ("username", &self.credential.username), ("password", &password_md5)];
        let mut res = self.client.post(NET_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
    fn logout(&self) -> Result<String> {
        let params = [("action", "logout")];
        let mut res = self.client.post(NET_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
}

impl<'a> NetConnectHelper for NetConnect<'a> {
    fn flux(&self) -> Result<NetFlux> {
        let mut res = self.client.get(NET_FLUX_URI).send()?;
        Ok(NetFlux::from_str(&res.text()?))
    }
}
