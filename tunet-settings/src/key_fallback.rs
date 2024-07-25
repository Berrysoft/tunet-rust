use crate::{SettingsError, SettingsResult};
use dirs::config_dir;
use keyring::{credential::CredentialApi, Error, Result};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use std::{
    collections::HashMap,
    fs::{File, Permissions},
    io::{BufReader, BufWriter},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

pub struct KeyFallback {
    service_path: PathBuf,
    user: String,
}

impl KeyFallback {
    pub fn new(service: &str, user: &str) -> SettingsResult<Self> {
        let mut service_path = config_dir().ok_or(SettingsError::ConfigDirNotFound)?;
        service_path.push(service);
        service_path.push("cred");
        service_path.set_extension("json");
        Ok(Self {
            service_path,
            user: user.into(),
        })
    }

    fn read_cred_file(&self) -> Result<HashMap<String, Password>> {
        if self.service_path.exists() {
            let f = File::open(&self.service_path).map_err(|e| Error::NoStorageAccess(e.into()))?;
            let reader = BufReader::new(f);
            serde_json::from_reader(reader).map_err(|e| Error::PlatformFailure(e.into()))
        } else {
            Ok(HashMap::new())
        }
    }

    fn save_cred_file(&self, map: &HashMap<String, Password>) -> Result<()> {
        let f = File::create(&self.service_path).map_err(|e| Error::NoStorageAccess(e.into()))?;
        f.set_permissions(Permissions::from_mode(0o600))
            .map_err(|e| Error::PlatformFailure(e.into()))?;
        let writer = BufWriter::new(f);
        serde_json::to_writer(writer, map).map_err(|e| Error::PlatformFailure(e.into()))
    }

    fn set_password_impl(&self, password: Password) -> Result<()> {
        let mut cred = self.read_cred_file()?;
        cred.insert(self.user.clone(), password);
        self.save_cred_file(&cred)
    }
}

impl CredentialApi for KeyFallback {
    fn set_password(&self, password: &str) -> Result<()> {
        self.set_password_impl(Password(password.as_bytes().to_vec()))
    }

    fn set_secret(&self, password: &[u8]) -> Result<()> {
        self.set_password_impl(Password(password.to_vec()))
    }

    fn get_password(&self) -> Result<String> {
        self.get_secret()
            .and_then(|p| String::from_utf8(p).map_err(|e| Error::BadEncoding(e.into_bytes())))
    }

    fn get_secret(&self) -> Result<Vec<u8>> {
        let mut cred = self.read_cred_file()?;
        cred.remove(&self.user).ok_or(Error::NoEntry).map(|p| p.0)
    }

    fn delete_credential(&self) -> Result<()> {
        let mut cred = self.read_cred_file()?;
        cred.remove(&self.user);
        self.save_cred_file(&cred)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct Password(#[serde_as(as = "Base64")] pub Vec<u8>);
