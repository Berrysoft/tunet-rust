#![forbid(unsafe_code)]

#[cfg(target_os = "linux")]
use std::cell::Cell;
use std::{
    borrow::Cow,
    fs::{DirBuilder, File, remove_file},
    io::{BufReader, BufWriter, Write, stdin, stdout},
    path::PathBuf,
};

use dirs::config_dir;
use keyring_core::Entry;
use rpassword::read_password;
#[cfg(target_os = "linux")]
use secret_service::{EncryptionType, blocking::SecretService};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_os = "linux")]
use zbus_secret_service_keyring_store as keyring_store;

#[cfg(target_os = "linux")]
#[path = "key_fallback.rs"]
mod key_fallback;

#[cfg(target_os = "android")]
use android_native_keyring_store as keyring_store;
#[cfg(target_os = "macos")]
use apple_native_keyring_store::keychain as keyring_store;
#[cfg(target_os = "ios")]
use apple_native_keyring_store::protected as keyring_store;
#[cfg(windows)]
use windows_native_keyring_store as keyring_store;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("找不到配置文件目录")]
    ConfigDirNotFound,
    #[error("系统错误：{0}")]
    IoError(#[from] std::io::Error),
    #[error("密码管理错误：{0}")]
    Keyring(#[from] keyring_core::Error),
    #[error("JSON 解析错误：{0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(target_os = "linux")]
impl SettingsError {
    pub fn is_no_entry(&self) -> bool {
        matches!(self, Self::Keyring(keyring_core::Error::NoEntry))
    }

    pub fn is_config_not_found(&self) -> bool {
        matches!(self, Self::IoError(e) if e.kind() == std::io::ErrorKind::NotFound)
    }
}

pub type SettingsResult<T> = Result<T, SettingsError>;

#[derive(Deserialize, Serialize)]
struct Settings<'a> {
    #[serde(default)]
    pub username: Cow<'a, str>,
}

static TUNET_NAME: &str = "tunet";

pub struct SettingsReader {
    path: PathBuf,
    #[cfg(target_os = "linux")]
    use_secret_service: Cell<bool>,
}

impl SettingsReader {
    pub fn new() -> SettingsResult<Self> {
        Self::with_dir(Self::file_dir()?)
    }

    fn file_dir() -> SettingsResult<PathBuf> {
        let mut p = config_dir().ok_or(SettingsError::ConfigDirNotFound)?;
        p.push(TUNET_NAME);
        Ok(p)
    }

    pub fn with_dir(path: impl Into<PathBuf>) -> SettingsResult<Self> {
        #[cfg(target_os = "linux")]
        let use_secret_service = match keyring_store::Store::new() {
            Ok(store) => {
                keyring_core::set_default_store(store);
                true
            }
            Err(_) => {
                keyring_core::set_default_store(key_fallback::Store::new()?);
                false
            }
        };
        #[cfg(not(target_os = "linux"))]
        keyring_core::set_default_store(keyring_store::Store::new()?);
        let mut path = path.into();
        path.push("settings");
        path.set_extension("json");
        Ok(Self {
            path,
            #[cfg(target_os = "linux")]
            use_secret_service: Cell::new(use_secret_service),
        })
    }

    fn entry(u: &str) -> SettingsResult<Entry> {
        Ok(Entry::new(TUNET_NAME, u)?)
    }

    #[cfg(target_os = "linux")]
    fn ensure_default_collection() -> SettingsResult<()> {
        let service = SecretService::connect(EncryptionType::Dh)
            .map_err(keyring_store::errors::decode_error)?;
        match service.get_default_collection() {
            Ok(_) => Ok(()),
            Err(secret_service::Error::NoResult) => {
                service
                    .create_collection("Default", "default")
                    .map_err(keyring_store::errors::decode_error)?;
                Ok(())
            }
            Err(e) => Err(keyring_store::errors::decode_error(e).into()),
        }
    }

    #[cfg(target_os = "linux")]
    fn use_fallback(&self) -> SettingsResult<()> {
        keyring_core::set_default_store(key_fallback::Store::new()?);
        self.use_secret_service.set(false);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn entry_with_fallback<T>(
        &self,
        u: &str,
        f: impl Fn(&Entry) -> keyring_core::Result<T>,
    ) -> SettingsResult<T> {
        let result = Self::entry(u).and_then(|entry| Ok(f(&entry)?));
        match result {
            Err(e) if self.use_secret_service.get() && !e.is_no_entry() => {
                self.use_fallback()?;
                let entry = Self::entry(u)?;
                Ok(f(&entry)?)
            }
            result => result,
        }
    }

    pub fn save(&mut self, u: &str, p: &str) -> SettingsResult<()> {
        #[cfg(target_os = "linux")]
        if self.use_secret_service.get() && Self::ensure_default_collection().is_err() {
            self.use_fallback()?;
        }
        if let Some(p) = self.path.parent() {
            DirBuilder::new().recursive(true).create(p)?;
        }
        let f = File::create(self.path.as_path())?;
        let writer = BufWriter::new(f);
        #[cfg(target_os = "linux")]
        self.entry_with_fallback(u, |entry| entry.set_password(p))?;
        #[cfg(not(target_os = "linux"))]
        {
            let entry = Self::entry(u)?;
            entry.set_password(p)?;
        }
        let c = Settings {
            username: Cow::Borrowed(u),
        };
        serde_json::to_writer(writer, &c)?;
        Ok(())
    }

    pub fn delete(&mut self, u: &str) -> SettingsResult<()> {
        #[cfg(target_os = "linux")]
        self.entry_with_fallback(u, Entry::delete_credential)?;
        #[cfg(not(target_os = "linux"))]
        {
            let entry = Self::entry(u)?;
            entry.delete_credential()?;
        }
        if self.path.exists() {
            remove_file(self.path.as_path())?;
        }
        Ok(())
    }

    pub fn read_username(&self) -> SettingsResult<String> {
        let f = File::open(self.path.as_path())?;
        let reader = BufReader::new(f);
        let c: Settings = serde_json::from_reader(reader)?;
        Ok(c.username.into_owned())
    }

    pub fn read_password(&self, u: &str) -> SettingsResult<String> {
        #[cfg(target_os = "linux")]
        let password = self.entry_with_fallback(u, Entry::get_password)?;
        #[cfg(not(target_os = "linux"))]
        let password = Self::entry(u)?.get_password()?;
        Ok(password)
    }

    pub fn read_full(&self) -> SettingsResult<(String, String)> {
        let u = self.read_username()?;
        let password = self.read_password(&u)?;
        Ok((u, password))
    }

    pub fn ask_username(&self) -> SettingsResult<String> {
        print!("请输入用户名：");
        stdout().flush()?;
        let mut u = String::new();
        stdin().read_line(&mut u)?;
        Ok(u.trim().to_string())
    }

    pub fn ask_password(&self) -> SettingsResult<String> {
        print!("请输入密码：");
        stdout().flush()?;
        Ok(read_password()?)
    }

    pub fn read_ask_username(&self) -> SettingsResult<String> {
        self.read_username().or_else(|_| self.ask_username())
    }

    pub fn read_ask_password(&self, u: &str) -> SettingsResult<String> {
        #[cfg(target_os = "linux")]
        {
            self.read_password(u).or_else(|e| {
                if e.is_no_entry() {
                    self.ask_password()
                } else {
                    Err(e)
                }
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.read_password(u).or_else(|_| self.ask_password())
        }
    }

    pub fn read_ask_full(&self) -> SettingsResult<(String, String)> {
        let u = self.read_ask_username()?;
        let p = self.read_ask_password(&u)?;
        Ok((u, p))
    }
}
