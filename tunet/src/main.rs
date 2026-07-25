// #![windows_subsystem = "windows"]
#![forbid(unsafe_code)]

mod commands;

use anyhow::Result;
use commands::{TUNet, TUNetCommand, TUNetImpl};
use compio::runtime::Runtime;

fn main() -> Result<()> {
    if std::env::args_os().count() == 1 {
        return tunet_gui::start();
    }
    let opt: TUNet = argh::from_env();
    match &opt.cmd {
        TUNetImpl::Service(service) => tunet_service::Command::run(service),
        _ => Runtime::new()?.block_on(opt.run()),
    }
}
