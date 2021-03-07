use super::*;

fn can_connect(client: &HttpClient, uri: &str) -> bool {
    client.head(uri).send().is_ok()
}

pub fn suggest(client: &HttpClient) -> NetState {
    if can_connect(client, "https://auth4.tsinghua.edu.cn") {
        NetState::Auth4
    } else if can_connect(client, "http://net.tsinghua.edu.cn") {
        NetState::Net
    } else if can_connect(client, "https://auth6.tsinghua.edu.cn") {
        NetState::Auth6
    } else {
        NetState::Unknown
    }
}
