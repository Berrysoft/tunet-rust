use async_trait::async_trait;
use tunet_helper::*;

#[async_trait]
pub trait TUNetHelperExt {
    async fn new_with_suggest(
        s: Option<NetState>,
        client: HttpClient,
    ) -> NetHelperResult<TUNetConnect>;
}

#[async_trait]
impl TUNetHelperExt for TUNetConnect {
    async fn new_with_suggest(
        s: Option<NetState>,
        client: HttpClient,
    ) -> NetHelperResult<TUNetConnect> {
        match s {
            None => {
                let s = crate::suggest(&client).await;
                Self::new(s, client)
            }
            Some(s) => Self::new(s, client),
        }
    }
}
