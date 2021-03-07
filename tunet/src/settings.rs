use dirs::config_dir;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use std::fs::{remove_file, DirBuilder, File};
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;
use tunet_rust::*;

#[derive(Debug, Default, Deserialize, Serialize)]
struct Settings {
    #[serde(rename = "Username")]
    #[serde(default)]
    pub username: String,
    #[serde(rename = "Password")]
    #[serde(default)]
    pub password: String,
    #[serde(rename = "AcIds")]
    #[serde(default)]
    pub ac_ids: Vec<i32>,
}

trait SettingsReader {
    fn read(&self) -> Result<Settings>;
    fn read_with_password(&self) -> Result<Settings>;
}

struct StdioSettingsReader;

impl StdioSettingsReader {
    fn read_username(&self) -> Result<String> {
        print!("请输入用户名：");
        stdout().flush()?;
        let mut u = String::new();
        stdin().read_line(&mut u)?;
        Ok(u.replace("\n", "").replace("\r", ""))
    }

    fn read_password(&self) -> Result<String> {
        print!("请输入密码：");
        stdout().flush()?;
        Ok(read_password()?)
    }
}

impl SettingsReader for StdioSettingsReader {
    fn read(&self) -> Result<Settings> {
        let u = self.read_username()?;
        Ok(Settings {
            username: u,
            password: String::new(),
            ac_ids: Vec::new(),
        })
    }

    fn read_with_password(&self) -> Result<Settings> {
        let u = self.read_username()?;
        let p = self.read_password()?;
        Ok(Settings {
            username: u,
            password: p,
            ac_ids: Vec::new(),
        })
    }
}

struct FileSettingsReader {
    path: PathBuf,
}

impl FileSettingsReader {
    pub fn new() -> Result<Self> {
        Ok(Self {
            path: Self::file_path()?,
        })
    }

    pub fn file_path() -> Result<PathBuf> {
        let mut p = config_dir().ok_or(NetHelperError::ConfigDirErr)?;
        p.push("TsinghuaNet.CLI");
        p.push("settings");
        p.set_extension("json");
        Ok(p)
    }

    pub fn file_exists() -> bool {
        Self::file_path().map(|p| p.exists()).unwrap_or(false)
    }

    pub fn save(&self, settings: &Settings) -> Result<()> {
        if let Some(p) = self.path.parent() {
            DirBuilder::new().recursive(true).create(p)?;
        }
        let f = File::create(self.path.as_path())?;
        let writer = BufWriter::new(f);
        serde_json::to_writer(writer, settings)?;
        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        if self.path.exists() {
            remove_file(self.path.as_path())?;
        }
        Ok(())
    }
}

impl SettingsReader for FileSettingsReader {
    fn read(&self) -> Result<Settings> {
        self.read_with_password()
    }

    fn read_with_password(&self) -> Result<Settings> {
        let f = File::open(self.path.as_path())?;
        let reader = BufReader::new(f);
        Ok(serde_json::from_reader(reader)?)
    }
}

pub fn read_cred() -> Result<(String, String, Vec<i32>)> {
    match if FileSettingsReader::file_exists() {
        FileSettingsReader::new()?.read_with_password()?
    } else {
        StdioSettingsReader.read_with_password()?
    } {
        Settings {
            username,
            password,
            ac_ids,
        } => Ok((username, password, ac_ids)),
    }
}

pub fn read_username() -> Result<(String, Vec<i32>)> {
    match if FileSettingsReader::file_exists() {
        FileSettingsReader::new()?.read()?
    } else {
        StdioSettingsReader.read()?
    } {
        Settings {
            username,
            password: _,
            ac_ids,
        } => Ok((username, ac_ids)),
    }
}

pub fn save_cred(u: String, p: String, ac_ids: Vec<i32>) -> Result<()> {
    FileSettingsReader::new()?.save(&Settings {
        username: u,
        password: p,
        ac_ids,
    })
}

pub fn delete_cred() -> Result<()> {
    let reader = FileSettingsReader::new()?;
    print!("是否删除设置文件？[y/N]");
    stdout().flush()?;
    let mut s = String::new();
    stdin().read_line(&mut s)?;
    if s.replace("\n", "").replace("\r", "").to_ascii_lowercase() == "y" {
        reader.delete()?;
        println!("已删除");
    }
    Ok(())
}
