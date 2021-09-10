use security_framework::{base::Result, os::macos::keychain::SecKeychain};

static TUNET_DUMMY_USERNAME: &str = "tunet-rust";

pub struct Keyring {
    keychain: SecKeychain,
    key: String,
}

impl Keyring {
    pub fn new(key: &str) -> Self {
        Self {
            keychain: SecKeychain::default(),
            key: key.to_owned(),
        }
    }

    pub fn get(&self) -> Result<String> {
        let (password_bytes, _) = self
            .keychain
            .find_generic_password(&self.key, TUNET_DUMMY_USERNAME)?;
        Ok(String::from_utf8_unchecked(password_bytes.to_vec()))
    }

    pub fn set(&self, value: &str) -> Result<()> {
        self.keychain
            .set_generic_password(&self.key, TUNET_DUMMY_USERNAME, value.as_bytes())
    }

    pub fn delete(&self) -> Result<()> {
        let (_, item) = self
            .keychain
            .find_generic_password(&self.key, TUNET_DUMMY_USERNAME)?;
        item.delete();
        Ok(())
    }
}
