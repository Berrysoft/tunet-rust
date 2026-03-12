#![forbid(unsafe_code)]

mod commands;

use anyhow::Result;
use commands::{TUNet, TUNetCommand};
use compio::runtime::Runtime;

fn main() -> Result<()> {
    let opt: TUNet = argh::from_env();
    Runtime::new()?.block_on(opt.run())
}
