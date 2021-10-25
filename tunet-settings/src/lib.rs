use dirs::config_dir;
use keyring::Keyring;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fs::{remove_file, DirBuilder, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;
use tunet_rust::*;

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
        Ok(Self {
            path: Self::file_path()?,
            keyring: Keyring::new(TUNET_NAME)?,
        })
    }

    pub fn file_path() -> Result<PathBuf> {
        let mut p = config_dir().ok_or_else(|| anyhow::anyhow!("找不到配置文件目录"))?;
        p.push(TUNET_NAME);
        p.push("settings");
        p.set_extension("json");
        Ok(p)
    }

    pub fn file_exists() -> bool {
        Self::file_path().map(|p| p.exists()).unwrap_or(false)
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
        if Self::file_exists() {
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
