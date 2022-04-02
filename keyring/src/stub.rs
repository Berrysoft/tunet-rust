use anyhow::{anyhow, Result};

pub struct Keyring;

impl Keyring {
    pub fn new(_: &str) -> Result<Self> {
        Ok(Self)
    }

    pub fn get(&self) -> Result<String> {
        Err(anyhow!("Keyring: unsupported platform."))
    }

    pub fn set(&self, _value: &str) -> Result<String> {
        Err(anyhow!("Keyring: unsupported platform."))
    }

    pub fn delete(&self) -> Result<()> {
        Err(anyhow!("Keyring: unsupported platform."))
    }
}
