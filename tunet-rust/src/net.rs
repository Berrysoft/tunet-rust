use crate::*;
use data_encoding::HEXLOWER;
use md5::{Digest, Md5};

pub struct NetConnect<'a> {
    cred: NetCredential,
    client: &'a HttpClient,
}

static NET_LOG_URI: &str = "http://net.tsinghua.edu.cn/do_login.php";
static NET_FLUX_URI: &str = "http://net.tsinghua.edu.cn/rad_user_info.php";

impl<'a> NetConnect<'a> {
    pub fn from_cred_client(cred: NetCredential, client: &'a HttpClient) -> Self {
        NetConnect { cred, client }
    }
}

impl<'a> TUNetHelper for NetConnect<'a> {
    fn login(&mut self) -> Result<String> {
        let password_md5 = {
            let mut md5 = Md5::new();
            md5.update(self.cred.password.as_bytes());
            md5.finalize()
        };
        let password_md5 = format!("{{MD5_HEX}}{}", HEXLOWER.encode(&password_md5));
        let params = [
            ("action", "login"),
            ("ac_id", "1"),
            ("username", &self.cred.username),
            ("password", &password_md5),
        ];
        let res = self.client.post(NET_LOG_URI).send_form(&params)?;
        Ok(res.into_string()?)
    }

    fn logout(&mut self) -> Result<String> {
        let params = [("action", "logout")];
        let res = self.client.post(NET_LOG_URI).send_form(&params)?;
        Ok(res.into_string()?)
    }

    fn flux(&self) -> Result<NetFlux> {
        let res = self.client.get(NET_FLUX_URI).call()?;
        res.into_string()?.parse()
    }

    fn cred(&self) -> &NetCredential {
        &self.cred
    }
}
