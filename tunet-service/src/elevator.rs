use std::io::Result;

#[cfg(target_os = "windows")]
pub fn elevate() -> Result<()> {
    use std::process::{exit, Command};

    if !is_elevated::is_elevated() {
        let status = Command::new("powershell.exe")
            .arg("-c")
            .arg("Start-Process")
            .arg(std::env::current_exe()?)
            .arg("-Verb")
            .arg("runas")
            .arg("-ArgumentList")
            .arg(
                std::env::args()
                    .skip(1)
                    .map(|s| format!("\'{}\'", s))
                    .collect::<Vec<_>>()
                    .join(","),
            )
            .arg("-Wait")
            .status()?;
        exit(status.code().unwrap_or_default());
    } else {
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub fn elevate() -> Result<()> {
    Ok(())
}
