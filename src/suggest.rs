use super::*;
use reqwest;

fn can_connect(uri: &str) -> bool {
    let client = reqwest::Client::new();
    match client.get(uri).send() {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn suggest() -> NetState {
    if can_connect("https://auth4.tsinghua.edu.cn") {
        NetState::Auth4
    } else if can_connect("http://net.tsinghua.edu.cn") {
        NetState::Net
    } else if can_connect("https://auth6.tsinghua.edu.cn") {
        NetState::Auth6
    } else {
        NetState::Unknown
    }
}
