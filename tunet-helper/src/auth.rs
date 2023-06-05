use crate::*;
use authtea::AuthTea;
use data_encoding::{Encoding, HEXLOWER};
use data_encoding_macro::new_encoding;
use hmac::{Hmac, Mac};
use md5::Md5;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value as JsonValue};
use sha1::{Digest, Sha1};
use std::marker::PhantomData;
use url::Url;

#[derive(Clone)]
pub struct AuthConnect<U: AuthConnectUri + Send + Sync> {
    cred: Arc<NetCredential>,
    client: HttpClient,
    _p: PhantomData<U>,
}

const AUTH_BASE64: Encoding = new_encoding! {
    symbols: "LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA",
    padding: '=',
};

static REDIRECT_URI: &str = "http://www.tsinghua.edu.cn/";

static AC_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"/index_([0-9]+)\.html").unwrap());

impl<U: AuthConnectUri + Send + Sync> AuthConnect<U> {
    pub fn new(cred: Arc<NetCredential>, client: HttpClient) -> Self {
        Self {
            cred,
            client,
            _p: PhantomData,
        }
    }

    async fn challenge(&self) -> NetHelperResult<String> {
        let uri = Url::parse_with_params(
            U::challenge_uri(),
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

    async fn get_ac_id(&self) -> Option<i32> {
        let res = self.client.get(REDIRECT_URI).send().await.ok()?;
        let t = res.text().await.ok()?;
        let cap = AC_ID_REGEX.captures(&t)?;
        cap[1].parse::<i32>().ok()
    }

    async fn try_login(&self, ac_id: i32) -> NetHelperResult<String> {
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
        let res = self.client.post(U::log_uri()).form(&params).send().await?;
        let t = res.text().await?;
        Self::parse_response(&t)
    }

    fn parse_response(t: &str) -> NetHelperResult<String> {
        let mut json: JsonValue = serde_json::from_str(&t[9..t.len() - 1])?;
        if let Some(error) = json["error"].as_str() {
            if error == "ok" {
                Ok(json
                    .remove("suc_msg")
                    .and_then(|v| v.into_str())
                    .unwrap_or_default())
            } else {
                Err(NetHelperError::Log(
                    json.remove("error_msg")
                        .and_then(|v| v.into_str())
                        .unwrap_or_default(),
                ))
            }
        } else {
            Err(NetHelperError::Log(json.to_string()))
        }
    }
}

#[async_trait]
impl<U: AuthConnectUri + Send + Sync> TUNetHelper for AuthConnect<U> {
    async fn login(&self) -> NetHelperResult<String> {
        let ac_id = self.get_ac_id().await.unwrap_or(1);
        Ok(self.try_login(ac_id).await?)
    }

    async fn logout(&self) -> NetHelperResult<String> {
        let params = [
            ("action", "logout"),
            ("ac_id", "1"),
            ("double_stack", "1"),
            ("username", &self.cred.username),
            ("callback", "callback"),
        ];
        let res = self.client.post(U::log_uri()).form(&params).send().await?;
        let t = res.text().await?;
        Self::parse_response(&t)
    }

    async fn flux(&self) -> NetHelperResult<NetFlux> {
        let res = self.client.get(U::flux_uri()).send().await?;
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
}

#[derive(Debug, Clone, Copy)]
pub struct Auth4Uri;

impl AuthConnectUri for Auth4Uri {
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
}

#[derive(Debug, Clone, Copy)]
pub struct Auth6Uri;

impl AuthConnectUri for Auth6Uri {
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

pub type Auth4Connect = AuthConnect<Auth4Uri>;
pub type Auth6Connect = AuthConnect<Auth6Uri>;
