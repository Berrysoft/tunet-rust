use dirs::config_dir;
use rpassword::read_password;
use serde_json::json;
use std::fs::{remove_file, DirBuilder, File};
use std::io::{stdin, stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;
use tunet_rust::*;

fn read_cred_from_stdio() -> Result<(String, String, Vec<i32>)> {
    let (u, ac_ids) = read_username_from_stdio()?;
    print!("请输入密码：");
    stdout().flush()?;
    let p = read_password()?;
    Ok((u, p, ac_ids))
}

fn read_username_from_stdio() -> Result<(String, Vec<i32>)> {
    print!("请输入用户名：");
    stdout().flush()?;
    let mut u = String::new();
    stdin().read_line(&mut u)?;
    u = u.replace("\n", "").replace("\r", "");
    Ok((u, vec![]))
}

fn settings_folder_path() -> Result<PathBuf> {
    let mut path = config_dir().ok_or(NetHelperError::ConfigDirErr)?;
    path.push("TsinghuaNet.CLI");
    Ok(path)
}

fn settings_file_path() -> Result<PathBuf> {
    let mut p = settings_folder_path()?;
    p.push("settings");
    p.set_extension("json");
    Ok(p)
}

fn read_json_from_file() -> Result<serde_json::Value> {
    let p = settings_file_path()?;
    let f = File::open(p)?;
    let reader = BufReader::new(f);
    Ok(serde_json::from_reader(reader)?)
}

fn read_cred_from_file() -> Result<(String, String, Vec<i32>)> {
    let json = read_json_from_file()?;
    let u = json["Username"]
        .as_str()
        .map(|s| s.to_owned())
        .unwrap_or_default();
    let p = json["Password"]
        .as_str()
        .map(|s| s.to_owned())
        .unwrap_or_default();
    let ac_ids = json["AcIds"]
        .as_array()
        .map(|v| {
            v.iter()
                .map(|v| v.as_i64().unwrap_or_default() as i32)
                .collect()
        })
        .unwrap_or_default();
    Ok((u, p, ac_ids))
}

fn read_username_from_file() -> Result<(String, Vec<i32>)> {
    let json = read_json_from_file()?;
    let u = json["Username"]
        .as_str()
        .map(|s| s.to_owned())
        .unwrap_or_default();
    let ac_ids = json["AcIds"]
        .as_array()
        .map(|v| {
            v.iter()
                .map(|v| v.as_i64().unwrap_or_default() as i32)
                .collect()
        })
        .unwrap_or_default();
    Ok((u, ac_ids))
}

fn settings_file_exists() -> bool {
    match settings_file_path() {
        Ok(p) => p.exists(),
        Err(_) => false,
    }
}

fn create_settings_folder() -> Result<()> {
    let p = settings_folder_path()?;
    DirBuilder::new().recursive(true).create(p)?;
    Ok(())
}

pub fn read_cred() -> Result<(String, String, Vec<i32>)> {
    if settings_file_exists() {
        read_cred_from_file()
    } else {
        read_cred_from_stdio()
    }
}

pub fn read_username() -> Result<(String, Vec<i32>)> {
    if settings_file_exists() {
        read_username_from_file()
    } else {
        read_username_from_stdio()
    }
}

pub fn save_cred(u: &str, p: &str, ac_ids: &[i32]) -> Result<()> {
    create_settings_folder()?;
    let json = json!({
        "Username":u,
        "Password":p,
        "AcIds":ac_ids
    });
    let p = settings_file_path()?;
    let f = File::create(p)?;
    let writer = BufWriter::new(f);
    serde_json::to_writer(writer, &json)?;
    Ok(())
}

pub fn delete_cred() -> Result<()> {
    if settings_file_exists() {
        print!("是否删除设置文件？[y/N]");
        stdout().flush()?;
        let mut s = String::new();
        stdin().read_line(&mut s)?;
        let c = s.chars().next().unwrap_or_default();
        if c.to_ascii_lowercase() == 'y' {
            let p = settings_file_path()?;
            remove_file(p)?;
            println!("已删除");
        }
    }
    Ok(())
}
