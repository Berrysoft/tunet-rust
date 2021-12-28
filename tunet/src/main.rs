#![forbid(unsafe_code)]

mod commands;

use commands::{TUNet, TUNetCommand};
use structopt::StructOpt;
use tokio::runtime::Builder as RuntimeBuilder;
use tunet_helper::Result;

fn main() -> Result<()> {
    let opt = TUNet::from_args_safe()?;
    RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(opt.run())
}
