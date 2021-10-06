#![forbid(unsafe_code)]

mod commands;
mod settings;
mod strfmt;

#[cfg(feature = "cui")]
mod cui;

use commands::TUNet;
use structopt::StructOpt;
use tokio::runtime::Builder as RuntimeBuilder;
use tunet_rust::Result;

fn main() -> Result<()> {
    let opt = TUNet::from_args();
    if opt.is_cui() {
        let mut builder = RuntimeBuilder::new_multi_thread();
        builder.worker_threads(4);
        builder
    } else {
        RuntimeBuilder::new_current_thread()
    }
    .enable_all()
    .build()?
    .block_on(TUNet::from_args().run())
}
