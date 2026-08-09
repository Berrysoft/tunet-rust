#![forbid(unsafe_code)]

#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(not(target_os = "linux"))]
use std::{
    borrow::Cow,
    fs::{DirBuilder, File, remove_file},
    io::{BufReader, BufWriter},
};
use std::{
    io::{Write, stdin, stdout},
    path::PathBuf,
};

#[cfg(not(target_os = "linux"))]
use dirs::config_dir;
use keyring_core::Entry;
use rpassword::read_password;
#[cfg(target_os = "linux")]
use secret_service::{EncryptionType, blocking::SecretService};
#[cfg(not(target_os = "linux"))]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_os = "linux")]
use zbus_secret_service_keyring_store as keyring_store;

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

impl SettingsError {
    pub fn is_no_entry(&self) -> bool {
        matches!(self, Self::Keyring(keyring_core::Error::NoEntry))
    }
}

pub type SettingsResult<T> = Result<T, SettingsError>;

#[cfg(not(target_os = "linux"))]
#[derive(Deserialize, Serialize)]
struct Settings<'a> {
    #[serde(default)]
    pub username: Cow<'a, str>,
}

static TUNET_NAME: &str = "tunet";

pub struct SettingsReader {
    #[cfg(not(target_os = "linux"))]
    path: PathBuf,
}

impl SettingsReader {
    pub fn new() -> SettingsResult<Self> {
        #[cfg(target_os = "linux")]
        {
            Self::with_dir(PathBuf::new())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Self::with_dir(Self::file_dir()?)
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn file_dir() -> SettingsResult<PathBuf> {
        let mut p = config_dir().ok_or(SettingsError::ConfigDirNotFound)?;
        p.push(TUNET_NAME);
        Ok(p)
    }

    pub fn with_dir(path: impl Into<PathBuf>) -> SettingsResult<Self> {
        keyring_core::set_default_store(keyring_store::Store::new()?);
        #[cfg(target_os = "linux")]
        {
            let _ = path;
            Ok(Self {})
        }
        #[cfg(not(target_os = "linux"))]
        {
            let mut path = path.into();
            path.push("settings");
            path.set_extension("json");
            Ok(Self { path })
        }
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
    fn saved_entry() -> SettingsResult<Entry> {
        let mut entries = Entry::search(&HashMap::from([("service", TUNET_NAME)]))?;
        match entries.len() {
            0 => Err(keyring_core::Error::NoEntry.into()),
            1 => Ok(entries.pop().unwrap()),
            _ => Err(keyring_core::Error::Ambiguous(entries).into()),
        }
    }

    #[cfg(target_os = "linux")]
    fn entry_username(entry: &Entry) -> SettingsResult<String> {
        entry.get_attributes()?.remove("username").ok_or_else(|| {
            keyring_core::Error::BadStoreFormat("找不到凭据的用户名".to_string()).into()
        })
    }

    pub fn save(&mut self, u: &str, p: &str) -> SettingsResult<()> {
        #[cfg(target_os = "linux")]
        {
            Self::ensure_default_collection()?;
            match self.read_username() {
                Ok(old_user) if old_user != u => Self::entry(&old_user)?.delete_credential()?,
                Ok(_) => {}
                Err(e) if e.is_no_entry() => {}
                Err(e) => return Err(e),
            }
            let entry = Self::entry(u)?;
            entry.set_password(p)?;
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            if let Some(p) = self.path.parent() {
                DirBuilder::new().recursive(true).create(p)?;
            }
            let f = File::create(self.path.as_path())?;
            let writer = BufWriter::new(f);
            let entry = Self::entry(u)?;
            entry.set_password(p)?;
            let c = Settings {
                username: Cow::Borrowed(u),
            };
            serde_json::to_writer(writer, &c)?;
            Ok(())
        }
    }

    pub fn delete(&mut self, u: &str) -> SettingsResult<()> {
        let entry = Self::entry(u)?;
        entry.delete_credential()?;
        #[cfg(not(target_os = "linux"))]
        {
            if self.path.exists() {
                remove_file(self.path.as_path())?;
            }
        }
        Ok(())
    }

    pub fn read_username(&self) -> SettingsResult<String> {
        #[cfg(target_os = "linux")]
        {
            Self::entry_username(&Self::saved_entry()?)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let f = File::open(self.path.as_path())?;
            let reader = BufReader::new(f);
            let c: Settings = serde_json::from_reader(reader)?;
            Ok(c.username.into_owned())
        }
    }

    pub fn read_password(&self, u: &str) -> SettingsResult<String> {
        let entry = Self::entry(u)?;
        let password = entry.get_password()?;
        Ok(password)
    }

    pub fn read_full(&self) -> SettingsResult<(String, String)> {
        #[cfg(target_os = "linux")]
        {
            let entry = Self::saved_entry()?;
            let username = Self::entry_username(&entry)?;
            let password = entry.get_password()?;
            Ok((username, password))
        }
        #[cfg(not(target_os = "linux"))]
        {
            let u = self.read_username()?;
            let password = self.read_password(&u)?;
            Ok((u, password))
        }
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
        #[cfg(target_os = "linux")]
        {
            self.read_username().or_else(|e| {
                if e.is_no_entry() {
                    self.ask_username()
                } else {
                    Err(e)
                }
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.read_username().or_else(|_| self.ask_username())
        }
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
