#![windows_subsystem = "windows"]

mod commands;
mod console;

use anyhow::Result;
use commands::{TUNet, TUNetCommand};

fn main() -> Result<()> {
    if std::env::args_os().count() == 1 {
        return tunet_gui::start();
    }
    console::attach_or_alloc_console()?;
    let opt: TUNet = argh::from_env();
    opt.run()
}
