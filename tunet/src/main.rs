#![forbid(unsafe_code)]

mod commands;
mod settings;
mod strfmt;

use commands::TUNet;
use structopt::StructOpt;
use tunet_rust::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    TUNet::from_args().run().await
}
