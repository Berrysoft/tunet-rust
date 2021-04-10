use super::*;
use data_encoding::HEXLOWER;
use md5::{Digest, Md5};

pub struct NetConnect<'a, 's> {
    credential: NetCredential<'s>,
    client: &'a HttpClient,
}

static NET_LOG_URI: &str = "http://net.tsinghua.edu.cn/do_login.php";
static NET_FLUX_URI: &str = "http://net.tsinghua.edu.cn/rad_user_info.php";

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

    pub fn login(&mut self) -> Result<String> {
        let password_md5 = {
            let mut md5 = Md5::new();
            md5.update(self.credential.password.as_bytes());
            md5.finalize()
        };
        let password_md5 = format!("{{MD5_HEX}}{}", HEXLOWER.encode(&password_md5));
        let params = [
            ("action", "login"),
            ("ac_id", "1"),
            ("username", &self.credential.username),
            ("password", &password_md5),
        ];
        let res = self.client.post(NET_LOG_URI).send_form(&params)?;
        Ok(res.into_string()?)
    }

    pub fn logout(&mut self) -> Result<String> {
        let params = [("action", "logout")];
        let res = self.client.post(NET_LOG_URI).send_form(&params)?;
        Ok(res.into_string()?)
    }

    pub fn flux(&self) -> Result<NetFlux> {
        let res = self.client.get(NET_FLUX_URI).call()?;
        Ok(NetFlux::from_str(&res.into_string()?))
    }
}
