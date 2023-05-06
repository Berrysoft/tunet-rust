#![forbid(unsafe_code)]

mod commands;

use anyhow::Result;
use clap::Parser;
use commands::{TUNet, TUNetCommand};
use tokio::runtime::Builder as RuntimeBuilder;

fn main() -> Result<()> {
    let opt = TUNet::parse();
    RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(opt.run())
}
