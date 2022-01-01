#![forbid(unsafe_code)]

mod commands;

use clap::Parser;
use commands::{TUNet, TUNetCommand};
use tokio::runtime::Builder as RuntimeBuilder;
use tunet_helper::Result;

fn main() -> Result<()> {
    let opt = TUNet::try_parse()?;
    RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(opt.run())
}
