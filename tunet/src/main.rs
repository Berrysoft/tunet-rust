#![forbid(unsafe_code)]

mod commands;

use anyhow::Result;
use clap::Parser;
use commands::{TUNet, TUNetCommand};
use compio::runtime::Runtime;

fn main() -> Result<()> {
    let opt = TUNet::parse();
    Runtime::new()?.block_on(opt.run())
}
