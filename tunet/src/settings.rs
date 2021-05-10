use dirs::config_dir;
use rpassword::read_password;
use std::fs::{remove_file, DirBuilder, File};
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;
use tunet_rust::*;

trait SettingsReader {
    fn read(&self) -> Result<NetCredential>;
    fn read_with_password(&self) -> Result<NetCredential>;
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
}

impl SettingsReader for StdioSettingsReader {
    fn read(&self) -> Result<NetCredential> {
        let u = self.read_username()?;
        Ok(NetCredential {
            username: u,
            password: String::new(),
            ac_ids: Vec::new(),
        })
    }

    fn read_with_password(&self) -> Result<NetCredential> {
        let u = self.read_username()?;
        let p = self.read_password()?;
        Ok(NetCredential {
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

    pub fn save(&self, settings: &NetCredential) -> Result<()> {
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
    fn read(&self) -> Result<NetCredential> {
        self.read_with_password()
    }

    fn read_with_password(&self) -> Result<NetCredential> {
        let f = File::open(self.path.as_path())?;
        let reader = BufReader::new(f);
        Ok(serde_json::from_reader(reader)?)
    }
}

pub fn read_cred() -> Result<NetCredential> {
    Ok(if FileSettingsReader::file_exists() {
        FileSettingsReader::new()?.read_with_password()?
    } else {
        StdioSettingsReader.read_with_password()?
    })
}

pub fn read_username() -> Result<NetCredential> {
    Ok(if FileSettingsReader::file_exists() {
        FileSettingsReader::new()?.read()?
    } else {
        StdioSettingsReader.read()?
    })
}

pub fn save_cred(cred: &NetCredential) -> Result<()> {
    FileSettingsReader::new()?.save(cred)
}

pub fn delete_cred() -> Result<()> {
    let reader = FileSettingsReader::new()?;
    print!("是否删除设置文件？[y/N]");
    stdout().flush()?;
    let mut s = String::new();
    stdin().read_line(&mut s)?;
    if s.replace(&['\n', '\r'][..], "").to_ascii_lowercase() == "y" {
        reader.delete()?;
        println!("已删除");
    }
    Ok(())
}
