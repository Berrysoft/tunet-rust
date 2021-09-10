use keyutils::{Keyring as LinuxKeyring, Result, SpecialKeyring};

pub struct Keyring {
    keyring: LinuxKeyring,
    key: String,
}

impl Keyring {
    pub fn new(key: &str) -> Self {
        Self {
            keyring: unsafe { LinuxKeyring::new(SpecialKeyring::UserSession.serial()) },
            key: key.to_owned(),
        }
    }

    pub fn get(&self) -> Result<String> {
        let key = self.keyring.search_for_key(&self.key, None)?;
        let value = key.read()?;
        Ok(unsafe { String::from_utf8_unchecked(value) })
    }

    pub fn set(&self, value: &str) -> Result<()> {
        self.keyring.add_key(&self.key, value.as_bytes())?;
        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        self.keyring.clear()
    }
}
