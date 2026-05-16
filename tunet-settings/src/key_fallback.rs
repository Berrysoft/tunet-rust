use std::{
    collections::HashMap,
    fs::{File, Permissions},
    io::{BufReader, BufWriter},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use dirs::config_dir;
use keyring_core::{
    Credential, Entry, Error, Result,
    api::{CredentialApi, CredentialStoreApi},
};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

use crate::SettingsError;

pub struct Store {
    id: String,
}

impl Store {
    pub fn new() -> Result<Arc<Self>> {
        let now = SystemTime::now();
        let elapsed = if now.lt(&UNIX_EPOCH) {
            UNIX_EPOCH.duration_since(now)
        } else {
            now.duration_since(UNIX_EPOCH)
        }
        .map_err(|e| Error::PlatformFailure(e.into()))?;
        Ok(Arc::new(Self {
            id: format!(
                "Crate version {}, Instantiated at {}",
                env!("CARGO_PKG_VERSION"),
                elapsed.as_secs_f64()
            ),
        }))
    }
}

impl CredentialStoreApi for Store {
    fn vendor(&self) -> String {
        "tunet-settings, https://github.com/Berrysoft/tunet-rust".to_string()
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn build(
        &self,
        service: &str,
        user: &str,
        _modifiers: Option<&HashMap<&str, &str>>,
    ) -> Result<Entry> {
        Ok(Entry::new_with_credential(Arc::new(KeyFallback::new(
            service, user,
        )?)))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct KeyFallback {
    service_path: PathBuf,
    service: String,
    user: String,
}

impl KeyFallback {
    pub fn new(service: &str, user: &str) -> Result<Self> {
        let mut service_path = config_dir().ok_or(Error::NoStorageAccess(
            SettingsError::ConfigDirNotFound.into(),
        ))?;
        service_path.push(service);
        service_path.push("cred");
        service_path.set_extension("json");
        Ok(Self {
            service_path,
            service: service.into(),
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
    fn set_secret(&self, password: &[u8]) -> Result<()> {
        self.set_password_impl(Password(password.to_vec()))
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

    fn get_credential(&self) -> Result<Option<Arc<Credential>>> {
        Ok(None)
    }

    fn get_specifiers(&self) -> Option<(String, String)> {
        Some((self.service.clone(), self.user.clone()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct Password(#[serde_as(as = "Base64")] pub Vec<u8>);
