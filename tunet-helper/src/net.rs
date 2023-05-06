use crate::*;
use data_encoding::HEXLOWER;
use md5::{Digest, Md5};

#[derive(Clone)]
pub struct NetConnect {
    cred: Arc<NetCredential>,
    client: HttpClient,
}

static NET_LOG_URI: &str = "http://net.tsinghua.edu.cn/do_login.php";
static NET_FLUX_URI: &str = "http://net.tsinghua.edu.cn/rad_user_info.php";

impl NetConnect {
    pub fn new(cred: Arc<NetCredential>, client: HttpClient) -> Self {
        NetConnect { cred, client }
    }
}

#[async_trait]
impl TUNetHelper for NetConnect {
    async fn login(&self) -> NetHelperResult<String> {
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
        let res = self.client.post(NET_LOG_URI).form(&params).send().await?;
        Ok(res.text().await?)
    }

    async fn logout(&self) -> NetHelperResult<String> {
        let params = [("action", "logout")];
        let res = self.client.post(NET_LOG_URI).form(&params).send().await?;
        Ok(res.text().await?)
    }

    async fn flux(&self) -> NetHelperResult<NetFlux> {
        let res = self.client.get(NET_FLUX_URI).send().await?;
        Ok(res.text().await?.parse()?)
    }

    fn cred(&self) -> Arc<NetCredential> {
        self.cred.clone()
    }
}
