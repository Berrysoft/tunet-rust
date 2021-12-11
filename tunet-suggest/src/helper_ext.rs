use async_trait::async_trait;
use std::sync::Arc;
use tunet_helper::*;

#[async_trait]
pub trait TUNetHelperExt {
    async fn new_with_suggest(
        s: Option<NetState>,
        cred: Arc<NetCredential>,
        client: HttpClient,
    ) -> Result<TUNetConnect>;
}

#[async_trait]
impl TUNetHelperExt for TUNetConnect {
    async fn new_with_suggest(
        s: Option<NetState>,
        cred: Arc<NetCredential>,
        client: HttpClient,
    ) -> Result<TUNetConnect> {
        match s {
            None => {
                let s = crate::suggest(&client).await;
                Self::new(s, cred, client)
            }
            Some(s) => Self::new(s, cred, client),
        }
    }
}
