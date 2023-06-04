#![forbid(unsafe_code)]

use dirs::config_dir;
use keyring::Entry;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs::{remove_file, DirBuilder, File};
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tunet_helper::*;

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
    #[serde(default)]
    pub ac_ids: BTreeSet<i32>,
}

static TUNET_NAME: &str = "tunet";

pub struct FileSettingsReader {
    path: PathBuf,
}

impl FileSettingsReader {
    pub fn new() -> SettingsResult<Self> {
        Ok(Self::with_path(Self::file_path()?))
    }

    pub fn file_path() -> SettingsResult<PathBuf> {
        let mut p = config_dir().ok_or(SettingsError::ConfigDirNotFound)?;
        p.push(TUNET_NAME);
        p.push("settings");
        p.set_extension("json");
        Ok(p)
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub async fn save(&mut self, settings: Arc<NetCredential>) -> SettingsResult<()> {
        if let Some(p) = self.path.parent() {
            DirBuilder::new().recursive(true).create(p)?;
        }
        let f = File::create(self.path.as_path())?;
        let writer = BufWriter::new(f);
        let ac_ids = settings.ac_ids.read().await;
        let entry = Entry::new(TUNET_NAME, &settings.username)?;
        entry.set_password(&settings.password)?;
        let c = Settings {
            username: Cow::Borrowed(&settings.username),
            ac_ids: ac_ids.clone(),
        };
        serde_json::to_writer(writer, &c)?;
        Ok(())
    }

    fn read_impl(&self) -> SettingsResult<Settings> {
        let f = File::open(self.path.as_path())?;
        let reader = BufReader::new(f);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn delete(&mut self) -> SettingsResult<()> {
        if self.path.exists() {
            let c = self.read_impl()?;
            let entry = Entry::new(TUNET_NAME, &c.username)?;
            entry.delete_password()?;
            remove_file(self.path.as_path())?;
        }
        Ok(())
    }

    pub fn read(&self) -> SettingsResult<NetCredential> {
        let c = self.read_impl()?;
        Ok(NetCredential::new(
            c.username.into_owned(),
            String::new(),
            c.ac_ids,
        ))
    }

    pub fn read_with_password(&self) -> SettingsResult<NetCredential> {
        let c = self.read_impl()?;
        let entry = Entry::new(TUNET_NAME, &c.username)?;
        let password = match entry.get_password() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("{}", e);
                Default::default()
            }
        };
        Ok(NetCredential::new(
            c.username.into_owned(),
            password,
            c.ac_ids,
        ))
    }
}

struct StdioSettingsReader;

impl StdioSettingsReader {
    fn read_username(&self) -> SettingsResult<String> {
        print!("请输入用户名：");
        stdout().flush()?;
        let mut u = String::new();
        stdin().read_line(&mut u)?;
        Ok(u.trim().to_string())
    }

    fn read_password(&self) -> SettingsResult<String> {
        print!("请输入密码：");
        stdout().flush()?;
        Ok(read_password()?)
    }

    pub fn read(&self) -> SettingsResult<NetCredential> {
        let u = self.read_username()?;
        Ok(NetCredential::new(u, String::new(), BTreeSet::new()))
    }

    pub fn read_with_password(&self) -> SettingsResult<NetCredential> {
        let u = self.read_username()?;
        let entry = Entry::new(TUNET_NAME, &u)?;
        let p = match entry.get_password() {
            Ok(p) => p,
            Err(_) => self.read_password()?,
        };
        Ok(NetCredential::new(u, p, BTreeSet::new()))
    }
}

pub fn read_cred() -> SettingsResult<Arc<NetCredential>> {
    if let Ok(reader) = FileSettingsReader::new() {
        if let Ok(cred) = reader.read_with_password() {
            return Ok(Arc::new(cred));
        }
    }
    Ok(Arc::new(StdioSettingsReader.read_with_password()?))
}

pub fn read_username() -> SettingsResult<Arc<NetCredential>> {
    if let Ok(reader) = FileSettingsReader::new() {
        if let Ok(cred) = reader.read() {
            return Ok(Arc::new(cred));
        }
    }
    Ok(Arc::new(StdioSettingsReader.read()?))
}

pub async fn save_cred(cred: Arc<NetCredential>) -> SettingsResult<()> {
    FileSettingsReader::new()?.save(cred).await
}

pub fn delete_cred() -> SettingsResult<()> {
    let mut reader = FileSettingsReader::new()?;
    print!("是否删除设置文件？[y/N]");
    stdout().flush()?;
    let mut s = String::new();
    stdin().read_line(&mut s)?;
    if s.trim().eq_ignore_ascii_case("y") {
        reader.delete()?;
        println!("已删除");
    }
    Ok(())
}
