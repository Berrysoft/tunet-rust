use anyhow::anyhow;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::Mutex;
use tunet_rust::{usereg::*, *};

pub static CREDENTIAL: OnceCell<Arc<NetCredential>> = OnceCell::new();
pub static USEREG_CLIENT: OnceCell<UseregHelper> = OnceCell::new();

lazy_static! {
    pub static ref HTTP_CLIENT: HttpClient = create_http_client().unwrap();
    pub static ref TUNET_CLIENT: Mutex<Option<TUNetConnect>> = Mutex::new(None);
}

pub fn init() -> Result<()> {
    let cred = Arc::new(NetCredential::default());
    CREDENTIAL
        .set(cred.clone())
        .map_err(|_| anyhow!("Cannot set CREDENTIAL."))?;
    let usereg = UseregHelper::new(cred, HTTP_CLIENT.clone());
    USEREG_CLIENT
        .set(usereg)
        .map_err(|_| anyhow!("Cannot set USEREG_CLIENT."))?;
    Ok(())
}

pub async fn replace_state(s: NetState) {
    *TUNET_CLIENT.lock().await = Some(match s {
        NetState::Net | NetState::Auth4 | NetState::Auth6 => {
            TUNetConnect::new(s, CREDENTIAL.get().unwrap().clone(), HTTP_CLIENT.clone())
                .await
                .unwrap()
        }
        _ => unreachable!(),
    });
}

pub async fn tunet() -> Result<TUNetConnect> {
    Ok(TUNET_CLIENT
        .lock()
        .await
        .as_ref()
        .ok_or_else(|| anyhow!("请选择连接方式"))?
        .clone())
}
