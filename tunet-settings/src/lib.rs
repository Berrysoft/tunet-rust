#![forbid(unsafe_code)]

use dirs::config_dir;
use keyring::Entry;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs::{remove_file, DirBuilder, File};
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("找不到配置文件目录")]
    ConfigDirNotFound,
    #[error("系统错误：{0}")]
    IoError(#[from] std::io::Error),
    #[error("密码管理错误：{0}")]
    Keyring(#[from] keyring::Error),
    #[error("JSON 解析错误：{0}")]
    Json(#[from] serde_json::Error),
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
}

impl SettingsReader {
    pub fn new() -> SettingsResult<Self> {
        Ok(Self::with_dir(Self::file_dir()?))
    }

    fn file_dir() -> SettingsResult<PathBuf> {
        let mut p = config_dir().ok_or(SettingsError::ConfigDirNotFound)?;
        p.push(TUNET_NAME);
        Ok(p)
    }

    pub fn with_dir(path: impl Into<PathBuf>) -> Self {
        let mut path = path.into();
        path.push("settings");
        path.set_extension("json");
        Self { path }
    }

    pub fn save(&mut self, u: &str, p: &str) -> SettingsResult<()> {
        if let Some(p) = self.path.parent() {
            DirBuilder::new().recursive(true).create(p)?;
        }
        let f = File::create(self.path.as_path())?;
        let writer = BufWriter::new(f);
        let entry = Entry::new(TUNET_NAME, u)?;
        entry.set_password(p)?;
        let c = Settings {
            username: Cow::Borrowed(u),
        };
        serde_json::to_writer(writer, &c)?;
        Ok(())
    }

    pub fn delete(&mut self, u: &str) -> SettingsResult<()> {
        let entry = Entry::new(TUNET_NAME, u)?;
        entry.delete_password()?;
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
        let entry = Entry::new(TUNET_NAME, u)?;
        let password = entry.get_password()?;
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
        self.read_password(u).or_else(|_| self.ask_password())
    }

    pub fn read_ask_full(&self) -> SettingsResult<(String, String)> {
        let u = self.read_ask_username()?;
        let p = self.read_ask_password(&u)?;
        Ok((u, p))
    }
}
