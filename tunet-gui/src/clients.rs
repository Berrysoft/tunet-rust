use crate::*;
use anyhow::anyhow;
use netstatus::*;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::Mutex;
use tunet_rust::usereg::*;
use tunet_settings::*;

static CREDENTIAL: OnceCell<Arc<NetCredential>> = OnceCell::new();
static USEREG_CLIENT: OnceCell<UseregHelper> = OnceCell::new();

lazy_static! {
    static ref HTTP_CLIENT: HttpClient = create_http_client().unwrap();
    static ref TUNET_CLIENT: Mutex<Option<TUNetConnect>> = Mutex::new(None);
}

pub fn init() -> Result<()> {
    let cred = Arc::new(FileSettingsReader::new()?.read_with_password()?);
    CREDENTIAL
        .set(cred.clone())
        .map_err(|_| anyhow!("Cannot set CREDENTIAL."))?;
    let usereg = UseregHelper::new(cred, HTTP_CLIENT.clone());
    USEREG_CLIENT
        .set(usereg)
        .map_err(|_| anyhow!("Cannot set USEREG_CLIENT."))?;
    Ok(())
}

pub fn cred() -> Arc<NetCredential> {
    CREDENTIAL.get().unwrap().clone()
}

pub async fn replace_state(s: NetState) -> Result<()> {
    *TUNET_CLIENT.lock().await = Some(match s {
        NetState::Net | NetState::Auth4 | NetState::Auth6 => {
            TUNetConnect::new(s, CREDENTIAL.get().unwrap().clone(), HTTP_CLIENT.clone())
                .await
                .unwrap()
        }
        _ => return Err(anyhow!("无法判断连接方式")),
    });
    Ok(())
}

pub async fn tunet() -> Result<TUNetConnect> {
    Ok(TUNET_CLIENT
        .lock()
        .await
        .as_ref()
        .ok_or_else(|| anyhow!("请选择连接方式"))?
        .clone())
}

pub fn status() -> NetStatus {
    NetStatus::current()
}

pub async fn suggest(s: NetStatus) -> NetState {
    suggest::suggest_with_status(&HTTP_CLIENT, s).await
}

pub fn usereg() -> UseregHelper {
    USEREG_CLIENT.get().unwrap().clone()
}
