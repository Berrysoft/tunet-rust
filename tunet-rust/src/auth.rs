use crate::*;
use authtea::AuthTea;
use data_encoding::{Encoding, HEXLOWER};
use data_encoding_macro::new_encoding;
use hmac::{Hmac, Mac, NewMac};
use lazy_static::lazy_static;
use md5::Md5;
use regex::Regex;
use serde_json::{json, Value as JsonValue};
use sha1::{Digest, Sha1};
use url::Url;

#[derive(Clone)]
pub struct AuthConnect<const V: i32> {
    cred: Arc<NetCredential>,
    client: HttpClient,
}

const AUTH_BASE64: Encoding = new_encoding! {
    symbols: "LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA",
    padding: '=',
};

lazy_static! {
    static ref AC_ID_REGEX: Regex = Regex::new(r"/index_([0-9]+)\.html").unwrap();
}

impl<const V: i32> AuthConnect<V>
where
    Self: AuthConnectUri,
{
    pub fn new(cred: Arc<NetCredential>, client: HttpClient) -> Self {
        Self { cred, client }
    }

    async fn challenge(&self) -> Result<String> {
        let uri = Url::parse_with_params(
            Self::challenge_uri(),
            &[
                ("username", self.cred.username.as_ref()),
                ("double_stack", "1"),
                ("ip", ""),
                ("callback", "callback"),
            ],
        )
        .unwrap();
        let res = self.client.get(uri).send().await?;
        let t = res.text().await?;
        let mut json: JsonValue = serde_json::from_str(&t[9..t.len() - 1])?;
        Ok(json
            .remove("challenge")
            .and_then(|v| v.into_str())
            .unwrap_or_default())
    }

    async fn get_ac_id(&self) -> Result<i32> {
        let res = self.client.get(Self::redirect_uri()).send().await?;
        let t = res.text().await?;
        match AC_ID_REGEX.captures(&t) {
            Some(cap) => Ok(cap[1].parse::<i32>().unwrap_or_default()),
            _ => Err(NetHelperError::NoAcIdErr.into()),
        }
    }

    async fn try_login(&self, ac_id: i32) -> Result<String> {
        let token = self.challenge().await?;
        let password_md5 = {
            let mut hmacmd5 = Hmac::<Md5>::new_from_slice(&[]).unwrap();
            hmacmd5.update(token.as_bytes());
            hmacmd5.finalize().into_bytes()
        };
        let password_md5 = HEXLOWER.encode(&password_md5);
        let encode_json = json!({
            "username": self.cred.username,
            "password": self.cred.password,
            "ip": "",
            "acid": ac_id,
            "enc_ver": "srun_bx1"
        });
        let info = {
            let tea = AuthTea::new(token.as_bytes());
            tea.encode(encode_json.to_string().as_bytes())
        };
        let info = format!("{{SRBX1}}{}", AUTH_BASE64.encode(&info));
        let chksum = {
            let mut sha1 = Sha1::new();
            sha1.update(format!(
                "{0}{1}{0}{2}{0}{4}{0}{0}200{0}1{0}{3}",
                token, self.cred.username, password_md5, info, ac_id
            ));
            sha1.finalize()
        };
        let params = [
            ("action", "login"),
            ("ac_id", &ac_id.to_string()),
            ("double_stack", "1"),
            ("n", "200"),
            ("type", "1"),
            ("username", &self.cred.username),
            ("password", &format!("{{MD5}}{}", password_md5)),
            ("info", &info),
            ("chksum", &HEXLOWER.encode(&chksum)),
            ("callback", "callback"),
        ];
        let res = self
            .client
            .post(Self::log_uri())
            .form(&params)
            .send()
            .await?;
        let t = res.text().await?;
        Self::parse_response(&t)
    }

    fn parse_response(t: &str) -> Result<String> {
        let mut json: JsonValue = serde_json::from_str(&t[9..t.len() - 1])?;
        if let Some(error) = json["error"].as_str() {
            if error == "ok" {
                Ok(json
                    .remove("suc_msg")
                    .and_then(|v| v.into_str())
                    .unwrap_or_default())
            } else {
                Err(NetHelperError::LogErr(
                    json.remove("error_msg")
                        .and_then(|v| v.into_str())
                        .unwrap_or_default(),
                )
                .into())
            }
        } else {
            Err(NetHelperError::LogErr(json.to_string()).into())
        }
    }
}

#[async_trait]
impl<const V: i32> TUNetHelper for AuthConnect<V>
where
    Self: AuthConnectUri,
{
    async fn login(&self) -> Result<String> {
        for ac_id in self.cred.ac_ids.read().await.iter() {
            let res = self.try_login(*ac_id).await;
            if res.is_ok() {
                return res;
            }
        }
        let ac_id = self.get_ac_id().await?;
        self.cred.ac_ids.write().await.push(ac_id);
        Ok(self.try_login(ac_id).await?)
    }

    async fn logout(&self) -> Result<String> {
        let params = [
            ("action", "logout"),
            ("ac_id", "1"),
            ("double_stack", "1"),
            ("username", &self.cred.username),
            ("callback", "callback"),
        ];
        let res = self
            .client
            .post(Self::log_uri())
            .form(&params)
            .send()
            .await?;
        let t = res.text().await?;
        Self::parse_response(&t)
    }

    async fn flux(&self) -> Result<NetFlux> {
        let res = self.client.get(Self::flux_uri()).send().await?;
        Ok(res.text().await?.parse()?)
    }

    fn cred(&self) -> Arc<NetCredential> {
        self.cred.clone()
    }
}

pub trait AuthConnectUri {
    fn log_uri() -> &'static str;
    fn challenge_uri() -> &'static str;
    fn flux_uri() -> &'static str;
    fn redirect_uri() -> &'static str;
}

impl AuthConnectUri for AuthConnect<4> {
    #[inline]
    fn log_uri() -> &'static str {
        "https://auth4.tsinghua.edu.cn/cgi-bin/srun_portal"
    }

    #[inline]
    fn challenge_uri() -> &'static str {
        "https://auth4.tsinghua.edu.cn/cgi-bin/get_challenge"
    }

    #[inline]
    fn flux_uri() -> &'static str {
        "https://auth4.tsinghua.edu.cn/rad_user_info.php"
    }

    #[inline]
    fn redirect_uri() -> &'static str {
        "http://3.3.3.3/"
    }
}

impl AuthConnectUri for AuthConnect<6> {
    #[inline]
    fn log_uri() -> &'static str {
        "https://auth6.tsinghua.edu.cn/cgi-bin/srun_portal"
    }

    #[inline]
    fn challenge_uri() -> &'static str {
        "https://auth6.tsinghua.edu.cn/cgi-bin/get_challenge"
    }

    #[inline]
    fn flux_uri() -> &'static str {
        "https://auth6.tsinghua.edu.cn/rad_user_info.php"
    }

    #[inline]
    fn redirect_uri() -> &'static str {
        "http://[333::3]/"
    }
}

trait ExactString {
    fn remove(&mut self, key: &str) -> Option<Self>
    where
        Self: Sized;
    fn into_str(self) -> Option<String>;
}

impl ExactString for JsonValue {
    fn remove(&mut self, key: &str) -> Option<Self> {
        match self {
            Self::Object(map) => map.remove(key),
            _ => None,
        }
    }

    fn into_str(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}
