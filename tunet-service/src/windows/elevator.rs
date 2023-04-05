use std::io::Result;

pub fn elevate() -> Result<()> {
    if !is_elevated::is_elevated() {
        let status = std::process::Command::new("powershell.exe")
            .arg("-c")
            .arg("Start-Process")
            .arg(std::env::current_exe()?)
            .arg("-Verb")
            .arg("runas")
            .arg("-ArgumentList")
            .arg(
                std::env::args()
                    .skip(1)
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(","),
            )
            .arg("-Wait")
            .status()?;
        std::process::exit(status.code().unwrap_or_default());
    } else {
        Ok(())
    }
}
