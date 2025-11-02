use crate::*;
use authtea::AuthTea;
use base64::{
    alphabet::Alphabet,
    engine::{general_purpose::PAD, GeneralPurpose},
    Engine,
};
use data_encoding::HEXLOWER;
use hmac::{Hmac, Mac};
use md5::Md5;
use nyquest::Request;
use regex::Regex;
use serde_json::{json, Value as JsonValue};
use sha1::{Digest, Sha1};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::LazyLock;
use url::Url;

#[derive(Clone)]
pub struct AuthConnect<U: AuthConnectUri + Send + Sync> {
    client: HttpClient,
    _p: PhantomData<U>,
}

pub static AUTH_BASE64: LazyLock<GeneralPurpose> = LazyLock::new(|| {
    let alphabet =
        Alphabet::new("LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA").unwrap();
    GeneralPurpose::new(&alphabet, PAD)
});

static REDIRECT_URI: &str = "http://www.tsinghua.edu.cn/";

static AC_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/index_([0-9]+)\.html").unwrap());

impl<U: AuthConnectUri + Send + Sync> AuthConnect<U> {
    pub fn new(client: HttpClient) -> Self {
        Self {
            client,
            _p: PhantomData,
        }
    }

    async fn challenge(&self, u: &str) -> NetHelperResult<String> {
        let uri = Url::parse_with_params(
            U::challenge_uri(),
            &[
                ("username", u),
                ("double_stack", "1"),
                ("ip", ""),
                ("callback", "callback"),
            ],
        )
        .unwrap();
        let res = self.client.request(Request::get(uri.to_string())).await?;
        let t = res.text().await?;
        let mut json: JsonValue = serde_json::from_str(&t[9..t.len() - 1])?;
        Ok(json
            .remove("challenge")
            .and_then(|v| v.into_str())
            .unwrap_or_default())
    }

    async fn get_ac_id(&self) -> Option<i32> {
        let res = self.client.request(Request::get(REDIRECT_URI)).await.ok()?;
        let t = res.text().await.ok()?;
        let cap = AC_ID_REGEX.captures(&t)?;
        cap[1].parse::<i32>().ok()
    }

    async fn try_login(&self, ac_id: i32, u: &str, p: &str) -> NetHelperResult<String> {
        let token = self.challenge(u).await?;
        let password_md5 = {
            let mut hmacmd5 = Hmac::<Md5>::new_from_slice(&[]).unwrap();
            hmacmd5.update(token.as_bytes());
            hmacmd5.finalize().into_bytes()
        };
        let password_md5 = HEXLOWER.encode(&password_md5);
        let encode_json = json!({
            "username": u,
            "password": p,
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
                "{token}{u}{token}{password_md5}{token}{ac_id}{token}{token}200{token}1{token}{info}"
            ));
            sha1.finalize()
        };
        let params = [
            ("action", Cow::Borrowed("login")),
            ("ac_id", Cow::Owned(ac_id.to_string())),
            ("double_stack", Cow::Borrowed("1")),
            ("n", Cow::Borrowed("200")),
            ("type", Cow::Borrowed("1")),
            ("username", Cow::Owned(u.to_string())),
            ("password", Cow::Owned(format!("{{MD5}}{password_md5}"))),
            ("info", Cow::Owned(info)),
            ("chksum", Cow::Owned(HEXLOWER.encode(&chksum))),
            ("callback", Cow::Borrowed("callback")),
        ];
        let res = self
            .client
            .request(Request::post(U::log_uri()).with_body(nyquest::Body::form(
                params.map(|(k, v)| (Cow::Borrowed(k), v)),
            )))
            .await?;
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

impl<U: AuthConnectUri + Send + Sync> TUNetHelper for AuthConnect<U> {
    async fn login(&self, u: &str, p: &str) -> NetHelperResult<String> {
        let ac_id = self.get_ac_id().await.unwrap_or(1);
        self.try_login(ac_id, u, p).await
    }

    async fn logout(&self, u: &str) -> NetHelperResult<String> {
        let params = [
            ("action", Cow::Borrowed("logout")),
            ("ac_id", Cow::Borrowed("1")),
            ("double_stack", Cow::Borrowed("1")),
            ("username", Cow::Owned(u.to_string())),
            ("callback", Cow::Borrowed("callback")),
        ];
        let res = self
            .client
            .request(Request::post(U::log_uri()).with_body(nyquest::Body::form(
                params.map(|(k, v)| (Cow::Borrowed(k), v)),
            )))
            .await?;
        let t = res.text().await?;
        Self::parse_response(&t)
    }

    async fn flux(&self) -> NetHelperResult<NetFlux> {
        let res = self.client.request(Request::get(U::flux_uri())).await?;
        res.text().await?.parse()
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
        "https://auth4.tsinghua.edu.cn/cgi-bin/rad_user_info"
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
        "https://auth6.tsinghua.edu.cn/cgi-bin/rad_user_info"
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
