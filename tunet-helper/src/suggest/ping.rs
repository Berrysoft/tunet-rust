use nyquest::Request;

use crate::*;

async fn can_connect_impl(client: &HttpClient, uri: &'static str) -> nyquest::Result<()> {
    client.request(Request::head(uri)).await?;
    Ok(())
}

async fn can_connect(client: &HttpClient, uri: &'static str) -> bool {
    can_connect_impl(client, uri).await.is_ok()
}

pub async fn suggest(client: &HttpClient) -> NetState {
    if can_connect(client, "https://tauth4.tsinghua.edu.cn").await {
        NetState::Auth4
    } else if can_connect(client, "https://tauth6.tsinghua.edu.cn").await {
        NetState::Auth6
    } else {
        NetState::Unknown
    }
}
