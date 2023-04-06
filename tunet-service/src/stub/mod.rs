use tunet_helper::{anyhow, Result};

pub fn register(_interval: Option<humantime::Duration>) -> Result<()> {
    Err(anyhow!("不支持的命令"))
}

pub fn unregister() -> Result<()> {
    Err(anyhow!("不支持的命令"))
}

pub fn start(_interval: Option<humantime::Duration>) -> Result<()> {
    Err(anyhow!("不支持的命令"))
}
