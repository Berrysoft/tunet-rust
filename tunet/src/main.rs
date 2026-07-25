// #![windows_subsystem = "windows"]
#![forbid(unsafe_code)]

mod commands;

use anyhow::Result;
use commands::{TUNet, TUNetCommand};

fn main() -> Result<()> {
    if std::env::args_os().count() == 1 {
        return tunet_gui::start();
    }
    let opt: TUNet = argh::from_env();
    opt.run()
}
