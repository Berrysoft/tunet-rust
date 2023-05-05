use std::io::Result;

#[cfg(target_os = "windows")]
pub fn elevate() -> Result<()> {
    use std::process::exit;

    if !is_elevated::is_elevated() {
        let status = runas::Command::new(std::env::current_exe()?)
            .args(&std::env::args_os().skip(1).collect::<Vec<_>>())
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
