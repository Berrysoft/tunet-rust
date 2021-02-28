use super::*;
use chrono::prelude::*;
use crypto::digest::Digest;
use crypto::md5::Md5;
use hwaddr::HwAddr;
use select::document::Document;
use select::predicate::*;
use std::net::Ipv4Addr;
use std::ops::Generator;

pub struct NetUser {
    pub address: Ipv4Addr,
    pub login_time: NaiveDateTime,
    pub mac_address: HwAddr,
}

impl NetUser {
    pub fn from_detail(a: Ipv4Addr, t: NaiveDateTime, m: HwAddr) -> Self {
        NetUser {
            address: a,
            login_time: t,
            mac_address: m,
        }
    }
}

pub struct NetDetail {
    pub login_time: NaiveDateTime,
    pub logout_time: NaiveDateTime,
    pub flux: u64,
}

impl NetDetail {
    pub fn from_detail(i: NaiveDateTime, o: NaiveDateTime, f: u64) -> Self {
        NetDetail {
            login_time: i,
            logout_time: o,
            flux: f,
        }
    }
}

#[derive(Debug)]
pub enum NetDetailOrder {
    LoginTime,
    LogoutTime,
    Flux,
}

impl NetDetailOrder {
    fn get_query(&self) -> String {
        String::from(match self {
            NetDetailOrder::LoginTime => "user_login_time",
            NetDetailOrder::LogoutTime => "user_drop_time",
            NetDetailOrder::Flux => "user_in_bytes",
        })
    }
}

impl str::FromStr for NetDetailOrder {
    type Err = NetHelperError;
    fn from_str(s: &str) -> Result<Self> {
        let ls = s.to_lowercase();
        if ls == "login" || ls == "logintime" {
            Ok(NetDetailOrder::LoginTime)
        } else if ls == "logout" || ls == "logouttime" {
            Ok(NetDetailOrder::LogoutTime)
        } else if ls == "flux" {
            Ok(NetDetailOrder::Flux)
        } else {
            Err(NetHelperError::OrderError)
        }
    }
}

pub struct UseregHelper<'a, 's> {
    credential: NetCredential<'s>,
    client: &'a HttpClient,
}

static USEREG_LOG_URI: &str = "https://usereg.tsinghua.edu.cn/do.php";
static USEREG_INFO_URI: &str = "https://usereg.tsinghua.edu.cn/online_user_ipv4.php";
static USEREG_CONNECT_URI: &str = "https://usereg.tsinghua.edu.cn/ip_login.php";
static DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

fn parse_flux(s: &str) -> u64 {
    let (flux, unit) = s.split_at(s.len() - 1);
    (flux.parse::<f64>().unwrap_or_default()
        * match unit {
            "G" => 1_000_000_000.0,
            "M" => 1_000_000.0,
            "K" => 1_000.0,
            _ => 1.0,
        }) as u64
}

impl<'a, 's> UseregHelper<'a, 's> {
    pub fn from_cred_client<S: Into<Cow<'s, str>>>(u: S, p: S, client: &'a HttpClient) -> Self {
        UseregHelper {
            credential: NetCredential::from_cred(u, p),
            client,
        }
    }

    pub fn connect(&self, addr: Ipv4Addr) -> Result<String> {
        let params = [
            ("n", "100"),
            ("is_pad", "1"),
            ("type", "1"),
            ("action", "do_login"),
            ("user_ip", &addr.to_string()),
            ("drop", "0"),
        ];
        let res = self.client.post(USEREG_CONNECT_URI).form(&params).send()?;
        Ok(res.text()?)
    }

    pub fn drop(&self, addr: Ipv4Addr) -> Result<String> {
        let params = [("action", "drop"), ("user_ip", &addr.to_string())];
        let res = self.client.post(USEREG_INFO_URI).form(&params).send()?;
        Ok(res.text()?)
    }

    pub fn users(
        &self,
    ) -> Result<GeneratorIteratorAdapter<impl Generator<Return = (), Yield = NetUser>>> {
        let res = self.client.get(USEREG_INFO_URI).send()?;
        let doc = Document::from(res.text()?.as_str());
        Ok(GeneratorIteratorAdapter::new(move || {
            let doc = Box::new(doc);
            for node in doc
                .find(Name("tr").descendant(Attr("align", "center")))
                .skip(1)
            {
                let tds = node.find(Name("td")).skip(1).collect::<Vec<_>>();
                yield NetUser::from_detail(
                    tds[0]
                        .text()
                        .parse::<Ipv4Addr>()
                        .unwrap_or(Ipv4Addr::new(0, 0, 0, 0)),
                    NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT)
                        .unwrap_or(NaiveDateTime::from_timestamp(0, 0)),
                    tds[6].text().parse::<HwAddr>().unwrap_or(HwAddr::from(0)),
                )
            }
        }))
    }

    pub fn details(
        &self,
        o: NetDetailOrder,
        des: bool,
    ) -> Result<GeneratorIteratorAdapter<impl Generator<Return = Result<()>, Yield = NetDetail> + '_>>
    {
        let now = Local::now();
        let off = 100;
        let des = if des { "DESC" } else { "" };
        Ok(GeneratorIteratorAdapter::new(move || {
            let mut i: usize = 1;
            loop {
                let res = self.client.get(
                    &format!("https://usereg.tsinghua.edu.cn/user_detail_list.php?action=query&desc={6}&order={5}&start_time={0}-{1:02}-01&end_time={0}-{1:02}-{2:02}&page={3}&offset={4}",
                        now.year(), now.month(), now.day(), i, off, o.get_query(), des))
                    .send()?;
                let doc = Box::new(Document::from(res.text()?.as_str()));
                let mut new_len = 0;
                for node in doc
                    .find(Name("tr").descendant(Attr("align", "center")))
                    .skip(1)
                {
                    let tds = node.find(Name("td")).skip(1).collect::<Vec<_>>();
                    if !tds.is_empty() {
                        yield NetDetail::from_detail(
                            NaiveDateTime::parse_from_str(&tds[1].text(), DATE_TIME_FORMAT)
                                .unwrap_or(NaiveDateTime::from_timestamp(0, 0)),
                            NaiveDateTime::parse_from_str(&tds[2].text(), DATE_TIME_FORMAT)
                                .unwrap_or(NaiveDateTime::from_timestamp(0, 0)),
                            parse_flux(&tds[4].text()),
                        );
                        new_len += 1;
                    }
                }
                if new_len < off {
                    break;
                }
                i += 1;
            }
            Ok(())
        }))
    }
}

impl<'a, 's> NetHelper for UseregHelper<'a, 's> {
    fn login(&mut self) -> Result<String> {
        let mut cry = Md5::new();
        cry.input_str(&self.credential.password);
        let params = [
            ("action", "login"),
            ("user_login_name", &self.credential.username),
            ("user_password", &cry.result_str()),
        ];
        let res = self.client.post(USEREG_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
    fn logout(&mut self) -> Result<String> {
        let params = [("action", "logout")];
        let res = self.client.post(USEREG_LOG_URI).form(&params).send()?;
        Ok(res.text()?)
    }
}
