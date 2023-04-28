#![forbid(unsafe_code)]

use dirs::config_dir;
use keyring::Keyring;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs::{remove_file, DirBuilder, File};
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tunet_helper::*;

#[derive(Deserialize, Serialize)]
struct Settings<'a> {
    #[serde(default)]
    pub username: Cow<'a, str>,
    #[serde(default)]
    pub password: Cow<'a, str>,
    #[serde(default)]
    pub ac_ids: Cow<'a, [i32]>,
}

impl From<Settings<'_>> for NetCredential {
    fn from(s: Settings) -> Self {
        Self::new(
            s.username.into_owned(),
            s.password.into_owned(),
            s.ac_ids.into_owned(),
        )
    }
}

static TUNET_NAME: &str = "tunet";

pub struct FileSettingsReader {
    path: PathBuf,
    keyring: Keyring,
}

impl FileSettingsReader {
    pub fn new() -> Result<Self> {
        Self::with_path(Self::file_path()?)
    }

    pub fn file_path() -> Result<PathBuf> {
        let mut p = config_dir().ok_or_else(|| anyhow::anyhow!("找不到配置文件目录"))?;
        p.push(TUNET_NAME);
        p.push("settings");
        p.set_extension("json");
        Ok(p)
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Result<Self> {
        Ok(Self {
            path: path.into(),
            keyring: Keyring::new(TUNET_NAME)?,
        })
    }

    pub async fn save(&mut self, settings: Arc<NetCredential>) -> Result<()> {
        if let Some(p) = self.path.parent() {
            DirBuilder::new().recursive(true).create(p)?;
        }
        let f = File::create(self.path.as_path())?;
        let writer = BufWriter::new(f);
        let ac_ids = settings.ac_ids.read().await;
        let c = if let Err(e) = self.keyring.set(&settings.password) {
            if cfg!(debug_assertions) {
                eprintln!("WARNING: {}", e);
            }
            Settings {
                username: Cow::Borrowed(&settings.username),
                password: Cow::Borrowed(&settings.password),
                ac_ids: Cow::Borrowed(ac_ids.as_ref()),
            }
        } else {
            // Don't write password.
            Settings {
                username: Cow::Borrowed(&settings.username),
                password: Cow::default(),
                ac_ids: Cow::Borrowed(ac_ids.as_ref()),
            }
        };
        serde_json::to_writer(writer, &c)?;
        Ok(())
    }

    pub fn delete(&mut self) -> Result<()> {
        self.keyring.delete().unwrap_or_else(|e| {
            if cfg!(debug_assertions) {
                eprintln!("WARNING: {}", e);
            }
        });
        if self.path.exists() {
            remove_file(self.path.as_path())?;
        }
        Ok(())
    }

    pub fn read(&self) -> Result<NetCredential> {
        let f = File::open(self.path.as_path())?;
        let reader = BufReader::new(f);
        let c: Settings = serde_json::from_reader(reader)?;
        Ok(c.into())
    }

    pub fn read_with_password(&self) -> Result<NetCredential> {
        let mut settings = self.read()?;
        match self.keyring.get() {
            Ok(password) => settings.password = password,
            Err(e) => {
                if cfg!(debug_assertions) {
                    eprintln!("WARNING: {}", e);
                }
            }
        }
        Ok(settings)
    }
}

struct StdioSettingsReader;

impl StdioSettingsReader {
    fn read_username(&self) -> Result<String> {
        print!("请输入用户名：");
        stdout().flush()?;
        let mut u = String::new();
        stdin().read_line(&mut u)?;
        Ok(u.replace(&['\n', '\r'][..], ""))
    }

    fn read_password(&self) -> Result<String> {
        print!("请输入密码：");
        stdout().flush()?;
        Ok(read_password()?)
    }

    pub fn read(&self) -> Result<NetCredential> {
        let u = self.read_username()?;
        Ok(NetCredential::new(u, String::new(), Vec::new()))
    }

    pub fn read_with_password(&self) -> Result<NetCredential> {
        let u = self.read_username()?;
        let p = self.read_password()?;
        Ok(NetCredential::new(u, p, Vec::new()))
    }
}

pub fn read_cred() -> Result<Arc<NetCredential>> {
    if let Ok(reader) = FileSettingsReader::new() {
        if let Ok(cred) = reader.read_with_password() {
            return Ok(Arc::new(cred));
        }
    }
    Ok(Arc::new(StdioSettingsReader.read_with_password()?))
}

pub fn read_username() -> Result<Arc<NetCredential>> {
    if let Ok(reader) = FileSettingsReader::new() {
        if let Ok(cred) = reader.read() {
            return Ok(Arc::new(cred));
        }
    }
    Ok(Arc::new(StdioSettingsReader.read()?))
}

pub async fn save_cred(cred: Arc<NetCredential>) -> Result<()> {
    FileSettingsReader::new()?.save(cred).await
}

pub fn delete_cred() -> Result<()> {
    let mut reader = FileSettingsReader::new()?;
    print!("是否删除设置文件？[y/N]");
    stdout().flush()?;
    let mut s = String::new();
    stdin().read_line(&mut s)?;
    if s.replace(&['\n', '\r'][..], "").eq_ignore_ascii_case("y") {
        reader.delete()?;
        println!("已删除");
    }
    Ok(())
}
