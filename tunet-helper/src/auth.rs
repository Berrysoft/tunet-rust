use std::{borrow::Cow, sync::LazyLock};

use authtea::AuthTea;
use base64::{
    Engine,
    alphabet::Alphabet,
    engine::{GeneralPurpose, general_purpose::PAD},
};
use data_encoding::HEXLOWER;
use hmac::{Hmac, Mac};
use md5::Md5;
use nyquest::Request;
use regex_lite::Regex;
use serde_json::{Value as JsonValue, json};
use sha1::{Digest, Sha1};
use url::Url;

use crate::*;

#[derive(Clone)]
pub struct TUNetConnect {
    client: HttpClient,
    uri: &'static AuthConnectUri,
}

pub static AUTH_BASE64: LazyLock<GeneralPurpose> = LazyLock::new(|| {
    let alphabet =
        Alphabet::new("LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA").unwrap();
    GeneralPurpose::new(&alphabet, PAD)
});

static REDIRECT_URI: &str = "http://www.tsinghua.edu.cn/";

static AC_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/index_([0-9]+)\.html").unwrap());

impl TUNetConnect {
    pub(crate) fn new_auth4(client: HttpClient) -> Self {
        Self::new_impl(client, &AUTH4_URI)
    }

    pub(crate) fn new_auth6(client: HttpClient) -> Self {
        Self::new_impl(client, &AUTH6_URI)
    }

    fn new_impl(client: HttpClient, uri: &'static AuthConnectUri) -> Self {
        Self { client, uri }
    }

    async fn challenge(&self, u: &str) -> NetHelperResult<String> {
        let uri = Url::parse_with_params(
            self.uri.challenge_uri,
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
            .request(
                Request::post(self.uri.log_uri).with_body(nyquest::Body::form(
                    params.map(|(k, v)| (Cow::Borrowed(k), v)),
                )),
            )
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

    pub async fn login(&self, u: &str, p: &str) -> NetHelperResult<String> {
        let ac_id = self.get_ac_id().await.unwrap_or(1);
        self.try_login(ac_id, u, p).await
    }

    pub async fn logout(&self, u: &str) -> NetHelperResult<String> {
        let params = [
            ("action", Cow::Borrowed("logout")),
            ("ac_id", Cow::Borrowed("1")),
            ("double_stack", Cow::Borrowed("1")),
            ("username", Cow::Owned(u.to_string())),
            ("callback", Cow::Borrowed("callback")),
        ];
        let res = self
            .client
            .request(
                Request::post(self.uri.log_uri).with_body(nyquest::Body::form(
                    params.map(|(k, v)| (Cow::Borrowed(k), v)),
                )),
            )
            .await?;
        let t = res.text().await?;
        Self::parse_response(&t)
    }

    pub async fn flux(&self) -> NetHelperResult<NetFlux> {
        let res = self.client.request(Request::get(self.uri.flux_uri)).await?;
        res.text().await?.parse()
    }
}

struct AuthConnectUri {
    log_uri: &'static str,
    challenge_uri: &'static str,
    flux_uri: &'static str,
}

const AUTH4_URI: AuthConnectUri = AuthConnectUri {
    log_uri: "https://auth4.tsinghua.edu.cn/cgi-bin/srun_portal",
    challenge_uri: "https://auth4.tsinghua.edu.cn/cgi-bin/get_challenge",
    flux_uri: "https://auth4.tsinghua.edu.cn/cgi-bin/rad_user_info",
};

const AUTH6_URI: AuthConnectUri = AuthConnectUri {
    log_uri: "https://auth6.tsinghua.edu.cn/cgi-bin/srun_portal",
    challenge_uri: "https://auth6.tsinghua.edu.cn/cgi-bin/get_challenge",
    flux_uri: "https://auth6.tsinghua.edu.cn/cgi-bin/rad_user_info",
};

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
