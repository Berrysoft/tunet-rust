use super::*;
use authtea::AuthTea;
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use reqwest::Client;
use rustc_serialize::hex::ToHex;
use serde_json::{self, Value};
use std::string::String;

const BASE64N: &[u8] = b"LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA";
const BASE64PAD: u8 = b'=';

fn base64(t: &[u8]) -> String {
    let a = t.len();
    let len = (a + 2) / 3 * 4;
    let mut u = vec![b'\0'; len];
    let mut ui = 0;
    for o in (0..a).step_by(3) {
        let h = ((t[o] as u32) << 16)
            + (if o + 1 < a { (t[o + 1] as u32) << 8 } else { 0 })
            + (if o + 2 < a { t[o + 2] as u32 } else { 0 });
        for i in 0..4 {
            if o * 8 + i * 6 > a * 8 {
                u[ui] = BASE64PAD;
            } else {
                u[ui] = BASE64N[(h >> (6 * (3 - i)) & 0x3F) as usize];
            }
            ui += 1;
        }
    }
    unsafe { String::from_utf8_unchecked(u) }
}

pub struct AuthConnect {
    credential: NetCredential,
    client: Client,
    ver: i32,
}

const AC_IDS: [i32; 5] = [1, 25, 33, 35, 37];

impl AuthConnect {
    pub fn new() -> Self {
        AuthConnect::from_cred(String::new(), String::new())
    }

    pub fn new_v6() -> Self {
        AuthConnect::from_cred_v6(String::new(), String::new())
    }

    pub fn from_cred(u: String, p: String) -> Self {
        AuthConnect::from_cred_v(u, p, 4)
    }

    pub fn from_cred_v6(u: String, p: String) -> Self {
        AuthConnect::from_cred_v(u, p, 6)
    }

    fn from_cred_v(u: String, p: String, v: i32) -> Self {
        AuthConnect {
            credential: NetCredential::from_cred(u, p),
            client: Client::new(),
            ver: v,
        }
    }

    fn challenge(&self) -> Result<String> {
        let mut res = self
            .client
            .get(&format!(
                "https://auth{0}.tsinghua.edu.cn/cgi-bin/get_challenge?username={1}&double_stack=1&ip&callback=callback",
                self.ver,
                self.credential.username
            ))
            .send()?;
        let t = res.text()?;
        let json: Value = serde_json::from_str(&t[9..t.len() - 1])?;
        match &json["challenge"] {
            Value::String(s) => Ok(s.to_string()),
            _ => Ok(String::new()),
        }
    }
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

impl NetHelper for AuthConnect {
    fn login(&self) -> Result<String> {
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
            let tea = AuthTea::new(token.as_bytes());
            let info = "{SRBX1}".to_owned() + &base64(&tea.encrypt_str(&encode_json.to_string()));
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
            let (suc, msg) = parse_response(&t)?;
            if suc {
                return Ok(msg);
            } else {
                continue;
            }
        }
        Err(NetHelperError::NoAcIdErr)
    }

    fn logout(&self) -> Result<String> {
        for ac_id in &AC_IDS {
            let token = self.challenge()?;
            let encode_json = serde_json::json!({
                "username":self.credential.username,
                "ip":"",
                "acid":ac_id,
                "enc_ver":"srun_bx1"
            });
            let tea = AuthTea::new(token.as_bytes());
            let info = "{SRBX1}".to_owned() + &base64(&tea.encrypt_str(&encode_json.to_string()));
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
            let (suc, msg) = parse_response(&t)?;
            if suc {
                return Ok(msg);
            } else {
                continue;
            }
        }
        Err(NetHelperError::NoAcIdErr)
    }
}

impl NetConnectHelper for AuthConnect {
    fn flux(&self) -> Result<NetFlux> {
        let mut res = self
            .client
            .get(&format!(
                "https://auth{0}.tsinghua.edu.cn/rad_user_info.php",
                self.ver
            ))
            .send()?;
        Ok(NetFlux::from_str(&res.text()?))
    }
}
