#![forbid(unsafe_code)]

mod commands;

#[cfg(feature = "cui")]
mod cui;

use commands::TUNet;
use structopt::StructOpt;
use tokio::runtime::Builder as RuntimeBuilder;
use tunet_helper::Result;

#[cfg(feature = "cui")]
fn runtime_builder(cui: bool) -> RuntimeBuilder {
    if cui {
        let mut builder = RuntimeBuilder::new_multi_thread();
        builder.worker_threads(4);
        builder
    } else {
        RuntimeBuilder::new_current_thread()
    }
}

#[cfg(not(feature = "cui"))]
fn runtime_builder(_cui: bool) -> RuntimeBuilder {
    RuntimeBuilder::new_current_thread()
}

fn main() -> Result<()> {
    let opt = TUNet::from_args_safe()?;
    runtime_builder(opt.is_cui())
        .enable_all()
        .build()?
        .block_on(opt.run())
}
