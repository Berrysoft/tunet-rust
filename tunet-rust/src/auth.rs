use super::*;
use authtea::AuthTea;
use data_encoding::{Encoding, HEXLOWER};
use data_encoding_macro::new_encoding;
use hmac::{Hmac, Mac, NewMac};
use lazy_static::lazy_static;
use md5::Md5;
use regex::Regex;
use serde_json::{self, Value};
use sha1::{Digest, Sha1};

const AUTH_BASE64: Encoding = new_encoding! {
    symbols: "LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA",
    padding: '=',
};

pub struct AuthConnect<'a, 's> {
    credential: NetCredential<'s>,
    client: &'a HttpClient,
    ver: i32,
    additional_ac_ids: Vec<i32>,
}

lazy_static! {
    static ref AC_ID_REGEX: Regex = Regex::new(r"/index_([0-9]+)\.html").unwrap();
}

impl<'a, 's> AuthConnect<'a, 's> {
    pub fn from_cred_client<SU: Into<Cow<'s, str>>, SP: Into<Cow<'s, str>>>(
        u: SU,
        p: SP,
        client: &'a HttpClient,
        ac_ids: Vec<i32>,
    ) -> Self {
        Self::from_cred_client_v(u, p, client, 4, ac_ids)
    }

    pub fn from_cred_client_v6<SU: Into<Cow<'s, str>>, SP: Into<Cow<'s, str>>>(
        u: SU,
        p: SP,
        client: &'a HttpClient,
        ac_ids: Vec<i32>,
    ) -> Self {
        Self::from_cred_client_v(u, p, client, 6, ac_ids)
    }

    fn from_cred_client_v<SU: Into<Cow<'s, str>>, SP: Into<Cow<'s, str>>>(
        u: SU,
        p: SP,
        client: &'a HttpClient,
        v: i32,
        ac_ids: Vec<i32>,
    ) -> Self {
        AuthConnect {
            credential: NetCredential::from_cred(u, p),
            client,
            ver: v,
            additional_ac_ids: ac_ids,
        }
    }

    fn challenge(&self) -> Result<String> {
        let res = self.client.get(&format!("https://auth{0}.tsinghua.edu.cn/cgi-bin/get_challenge?username={1}&double_stack=1&ip&callback=callback", self.ver, self.credential.username)).call()?;
        let t = res.into_string()?;
        let json: Value = serde_json::from_str(&t[9..t.len() - 1])?;
        match &json["challenge"] {
            Value::String(s) => Ok(s.to_string()),
            _ => Ok(String::new()),
        }
    }

    fn get_ac_id(&self) -> Result<i32> {
        let res = self
            .client
            .get(if self.ver == 4 {
                "http://3.3.3.3/"
            } else {
                "http://[333::3]/"
            })
            .call()?;
        let t = res.into_string()?;
        match AC_ID_REGEX.captures(&t) {
            Some(cap) => Ok(cap[1].parse::<i32>().unwrap_or_default()),
            _ => Err(NetHelperError::NoAcIdErr),
        }
    }

    fn do_log<F>(&mut self, action: F) -> Result<String>
    where
        F: Fn(&Self, i32) -> Result<String>,
    {
        for ac_id in &self.additional_ac_ids {
            let res = action(self, *ac_id);
            if res.is_ok() {
                return res;
            }
        }
        let ac_id = self.get_ac_id()?;
        self.additional_ac_ids.push(ac_id);
        return action(self, ac_id);
    }

    fn parse_response(t: &str) -> Result<String> {
        let json: Value = serde_json::from_str(&t[9..t.len() - 1])?;
        if let Value::String(error) = &json["error"] {
            if error == "ok" {
                Ok(json["suc_msg"]
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_default())
            } else {
                Err(NetHelperError::LogErr(
                    json["error_msg"]
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                ))
            }
        } else {
            Err(NetHelperError::LogErr(json.to_string()))
        }
    }

    pub fn login(&mut self) -> Result<String> {
        self.do_log(|s, ac_id| {
            let token = s.challenge()?;
            let password_md5 = {
                let mut hmacmd5 = Hmac::<Md5>::new_varkey(&[]).unwrap();
                hmacmd5.update(token.as_bytes());
                hmacmd5.finalize().into_bytes()
            };
            let password_md5 = HEXLOWER.encode(&password_md5);
            let encode_json = serde_json::json!({
                "username": s.credential.username,
                "password": s.credential.password,
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
                    token, s.credential.username, password_md5, info, ac_id
                ));
                sha1.finalize()
            };
            let params = [
                ("action", "login"),
                ("ac_id", &ac_id.to_string()),
                ("double_stack", "1"),
                ("n", "200"),
                ("type", "1"),
                ("username", &s.credential.username),
                ("password", &format!("{{MD5}}{}", password_md5)),
                ("info", &info),
                ("chksum", &HEXLOWER.encode(&chksum)),
                ("callback", "callback"),
            ];
            let res = s
                .client
                .post(&format!(
                    "https://auth{0}.tsinghua.edu.cn/cgi-bin/srun_portal",
                    s.ver
                ))
                .send_form(&params)?;
            let t = res.into_string()?;
            Self::parse_response(&t)
        })
    }

    pub fn logout(&mut self) -> Result<String> {
        let params = [
            ("action", "logout"),
            ("ac_id", "1"),
            ("double_stack", "1"),
            ("username", &self.credential.username),
            ("callback", "callback"),
        ];
        let res = self
            .client
            .post(&format!(
                "https://auth{0}.tsinghua.edu.cn/cgi-bin/srun_portal",
                self.ver
            ))
            .send_form(&params)?;
        let t = res.into_string()?;
        Self::parse_response(&t)
    }

    pub fn flux(&self) -> Result<NetFlux> {
        let res = self
            .client
            .get(&format!(
                "https://auth{0}.tsinghua.edu.cn/rad_user_info.php",
                self.ver
            ))
            .call()?;
        Ok(NetFlux::from_str(&res.into_string()?))
    }

    pub fn ac_ids(&self) -> &[i32] {
        &self.additional_ac_ids
    }
}
