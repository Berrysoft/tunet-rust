#![forbid(unsafe_code)]

mod commands;
mod settings;
mod strfmt;

use commands::TUNet;
use structopt::StructOpt;
use tunet_rust::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opt = TUNet::from_args();
    opt.run().await?;
    Ok(())
}
