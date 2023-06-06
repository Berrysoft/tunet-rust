use crate::*;

async fn can_connect(client: &HttpClient, uri: &str) -> bool {
    client.head(uri).send().await.is_ok()
}

pub async fn suggest(client: &HttpClient) -> NetState {
    if can_connect(client, "https://auth4.tsinghua.edu.cn").await {
        NetState::Auth4
    } else if can_connect(client, "http://net.tsinghua.edu.cn").await {
        NetState::Net
    } else if can_connect(client, "https://auth6.tsinghua.edu.cn").await {
        NetState::Auth6
    } else {
        NetState::Unknown
    }
}
