use rpassword::read_password;
use std::io::{stdin, stdout, Write};
use std::sync::Arc;
use tunet_rust::*;
use tunet_settings::*;

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
