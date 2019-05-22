use super::*;
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use reqwest;
use rustc_serialize::hex::ToHex;
use serde_json::{self, Value};
use std::string;

mod encode;

pub struct AuthConnect {
    credential: NetCredential,
    client: reqwest::Client,
    ver: i32,
}

const AC_IDS: [i32; 5] = [1, 25, 33, 35, 37];

impl AuthConnect {
    pub fn new() -> Self {
        AuthConnect::from_cred(string::String::new(), string::String::new())
    }

    pub fn new_v6() -> Self {
        AuthConnect::from_cred_v6(string::String::new(), string::String::new())
    }

    pub fn from_cred(u: string::String, p: string::String) -> Self {
        AuthConnect::from_cred_v(u, p, 4)
    }

    pub fn from_cred_v6(u: string::String, p: string::String) -> Self {
        AuthConnect::from_cred_v(u, p, 6)
    }

    fn from_cred_v(u: string::String, p: string::String, v: i32) -> Self {
        AuthConnect {
            credential: NetCredential::from_cred(u, p),
            client: reqwest::Client::new(),
            ver: v,
        }
    }

    fn challenge(&self) -> Result<string::String> {
        let params = [
            ("double_stack", "1"),
            ("ip", ""),
            ("username", &self.credential.username),
            ("callback", "callback"),
        ];
        let mut res = self
            .client
            .get(&format!(
                "https://auth{0}.tsinghua.edu.cn/cgi-bin/get_challenge",
                self.ver
            ))
            .form(&params)
            .send()?;
        let t = res.text()?;
        let json: Value = serde_json::from_str(&t[9..t.len() - 1])?;
        match &json["challenge"] {
            Value::String(s) => Ok(s.to_string()),
            _ => Ok(string::String::new()),
        }
    }
}

impl NetHelper for AuthConnect {
    fn login(&self) -> Result<string::String> {
        for ac_id in &AC_IDS {
            let token = self.challenge()?;
            let mut md5 = Md5::new();
            md5.input_str(&token);
            let mut hmac = Hmac::new(md5, &[]);
            let password_md5 = hmac.result().code().to_hex();
            let p_mmd5 = "{MD5}".to_owned() + &password_md5;
            let encode_json = serde_json::json!({
                "username":self.credential.username,
                "password":self.credential.password,
                "ip":"",
                "acid":ac_id,
                "enc_ver":"srun_bx1"
            });
            let info = "{SRBX1}".to_owned()
                + &encode::base64(&encode::xencode(&encode_json.to_string(), &token));
            let mut sha1 = Sha1::new();
            sha1.input_str(&format!(
                "{0}{1}{0}{2}{0}{4}{0}{0}200{0}1{0}{3}",
                token, self.credential.username, password_md5, info, ac_id
            ));
            let params = [
                ("action", "login"),
                ("ac_id", &ac_id.to_string()),
                ("double_stack", "1"),
                ("n", "200"),
                ("type", "1"),
                ("username", &self.credential.username),
                ("password", &p_mmd5),
                ("info", &info),
                ("chksum", &sha1.result_str()),
                ("callback", "callback"),
            ];
            let mut res = self
                .client
                .post(&format!(
                    "https://auth{0}.tsinghua.edu.cn/cgi-bin/srun_portal",
                    self.ver
                ))
                .form(&params)
                .send()?;
            let t = res.text()?;
            let json: Value = serde_json::from_str(&t[9..t.len() - 1])?;
            match &json["error"] {
                Value::String(s) => {
                    if s == "ok" {
                        return Ok(json.to_string());
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };
        }
        Err(NetHelperError::NoAcIdErr)
    }

    fn logout(&self) -> Result<string::String> {
        for ac_id in &AC_IDS {
            let token = self.challenge()?;
            let encode_json = serde_json::json!({
                "username":self.credential.username,
                "ip":"",
                "acid":ac_id,
                "enc_ver":"srun_bx1"
            });
            let info = "{SRBX1}".to_owned()
                + &encode::base64(&encode::xencode(&encode_json.to_string(), &token));
            let mut sha1 = Sha1::new();
            sha1.input_str(&format!(
                "{0}{1}{0}{3}{0}{0}200{0}1{0}{2}",
                token, self.credential.username, info, ac_id
            ));
            let params = [
                ("action", "logout"),
                ("ac_id", &ac_id.to_string()),
                ("double_stack", "1"),
                ("n", "200"),
                ("type", "1"),
                ("username", &self.credential.username),
                ("info", &info),
                ("chksum", &sha1.result_str()),
                ("callback", "callback"),
            ];
            let mut res = self
                .client
                .post(&format!(
                    "https://auth{0}.tsinghua.edu.cn/cgi-bin/srun_portal",
                    self.ver
                ))
                .form(&params)
                .send()?;
            let t = res.text()?;
            let json: Value = serde_json::from_str(&t[9..t.len() - 1])?;
            match &json["error"] {
                Value::String(s) => {
                    if s == "ok" {
                        return Ok(json.to_string());
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };
        }
        Err(NetHelperError::NoAcIdErr)
    }
}

impl NetConnectHelper for AuthConnect {
    fn flux(&self) -> Result<NetFlux> {
        Ok(NetFlux::new())
    }
}
