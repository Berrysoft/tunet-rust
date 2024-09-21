use crate::*;

async fn can_connect_impl(client: &HttpClient, uri: &str) -> cyper::Result<()> {
    client.head(uri)?.send().await?;
    Ok(())
}

async fn can_connect(client: &HttpClient, uri: &str) -> bool {
    can_connect_impl(client, uri).await.is_ok()
}

pub async fn suggest(client: &HttpClient) -> NetState {
    if can_connect(client, "https://auth4.tsinghua.edu.cn").await {
        NetState::Auth4
    } else if can_connect(client, "https://net.tsinghua.edu.cn").await {
        NetState::Net
    } else if can_connect(client, "https://auth6.tsinghua.edu.cn").await {
        NetState::Auth6
    } else {
        NetState::Unknown
    }
}
