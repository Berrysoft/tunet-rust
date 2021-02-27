use super::*;
use crypto::digest::Digest;
use crypto::md5::Md5;
use std::string::String;

pub struct NetConnect<'a, 's> {
    credential: NetCredential<'s>,
    client: &'a HttpClient,
}

const NET_LOG_URI: &'static str = "http://net.tsinghua.edu.cn/do_login.php";
const NET_FLUX_URI: &'static str = "http://net.tsinghua.edu.cn/rad_user_info.php";

impl<'a, 's> NetConnect<'a, 's> {
    pub fn from_cred_client<SU: Into<Cow<'s, str>>, SP: Into<Cow<'s, str>>>(
        u: SU,
        p: SP,
        client: &'a HttpClient,
    ) -> Self {
        NetConnect {
            credential: NetCredential::from_cred(u, p),
            client,
        }
    }
}

impl<'a, 's> NetHelper for NetConnect<'a, 's> {
    fn login(&mut self) -> Result<String> {
        let mut cry = Md5::new();
        cry.input_str(&self.credential.password);
        let password_md5 = "{MD5_HEX}".to_owned() + &cry.result_str();
        let params = [
            ("action", "login"),
            ("ac_id", "1"),
            ("username", &self.credential.username),
            ("password", &password_md5),
        ];
        let res = self.client.post(NET_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
    fn logout(&mut self) -> Result<String> {
        let params = [("action", "logout")];
        let res = self.client.post(NET_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
}

impl<'a, 's> NetConnectHelper for NetConnect<'a, 's> {
    fn flux(&self) -> Result<NetFlux> {
        let res = self.client.get(NET_FLUX_URI).send()?;
        Ok(NetFlux::from_str(&res.text()?))
    }
    fn ac_ids(&self) -> &[i32] {
        &[0; 0]
    }
}
