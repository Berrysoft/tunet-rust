use anyhow::*;

pub struct Keyring;

impl Keyring {
    pub fn new(_: &str) -> Self {
        Self
    }

    pub fn get(&self) -> Result<String> {
        anyhow!("Keyring: unsupported platform.")
    }

    pub fn set(&self, value: &str) -> Result<String> {
        anyhow!("Keyring: unsupported platform.")
    }

    pub fn delete(&self) -> Result<()> {
        anyhow!("Keyring: unsupported platform.")
    }
}
