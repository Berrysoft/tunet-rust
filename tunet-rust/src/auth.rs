use super::*;
use authtea::AuthTea;
use crypto2::hash::Sha1;
use crypto2::mac::HmacMd5;
use lazy_static::lazy_static;
use radix64::CustomConfig;
use regex::Regex;
use serde_json::{self, Value};

lazy_static! {
    static ref AUTH_BASE64: CustomConfig = CustomConfig::with_alphabet(
        "LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA"
    )
    .build()
    .unwrap();
}

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

    fn parse_response(t: &str) -> Result<(bool, String)> {
        let json: Value = serde_json::from_str(&t[9..t.len() - 1])?;
        if let Value::String(error) = &json["error"] {
            if let Value::String(error_msg) = &json["error_msg"] {
                return Ok((
                    error == "ok",
                    format!("error: {}; error_msg: {}", error, error_msg),
                ));
            }
        }
        Ok((false, String::new()))
    }

    pub fn login(&mut self) -> Result<String> {
        self.do_log(|s, ac_id| {
            let token = s.challenge()?;
            let mut hmac = HmacMd5::new(&[]);
            hmac.update(token.as_bytes());
            let password_md5 = hex::encode(hmac.finalize());
            let p_mmd5 = format!("{{MD5}}{}", password_md5);
            let encode_json = serde_json::json!({
                "username": s.credential.username,
                "password": s.credential.password,
                "ip": "",
                "acid": ac_id,
                "enc_ver": "srun_bx1"
            });
            let tea = AuthTea::new(token.as_bytes());
            let info = format!(
                "{{SRBX1}}{}",
                AUTH_BASE64.encode(&tea.encrypt_str(&encode_json.to_string()))
            );
            let mut sha1 = Sha1::new();
            sha1.update(
                format!(
                    "{0}{1}{0}{2}{0}{4}{0}{0}200{0}1{0}{3}",
                    token, s.credential.username, password_md5, info, ac_id
                )
                .as_bytes(),
            );
            let params = [
                ("action", "login"),
                ("ac_id", &ac_id.to_string()),
                ("double_stack", "1"),
                ("n", "200"),
                ("type", "1"),
                ("username", &s.credential.username),
                ("password", &p_mmd5),
                ("info", &info),
                ("chksum", &hex::encode(sha1.finalize())),
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
            let (suc, msg) = Self::parse_response(&t)?;
            if suc {
                Ok(msg)
            } else {
                Err(NetHelperError::LogErr(msg))
            }
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
        let (suc, msg) = Self::parse_response(&t)?;
        if suc {
            Ok(msg)
        } else {
            Err(NetHelperError::LogErr(msg))
        }
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
