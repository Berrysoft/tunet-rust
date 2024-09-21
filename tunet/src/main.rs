#![forbid(unsafe_code)]

mod commands;

use anyhow::Result;
use clap::Parser;
use commands::{TUNet, TUNetCommand};
use compio::runtime::RuntimeBuilder;

fn main() -> Result<()> {
    let opt = TUNet::parse();
    RuntimeBuilder::new().build()?.block_on(opt.run())
}
