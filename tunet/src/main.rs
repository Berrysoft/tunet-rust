#![forbid(unsafe_code)]

mod commands;
mod settings;
mod strfmt;

#[cfg(feature = "cui")]
mod cui;

use commands::TUNet;
use structopt::StructOpt;
use tunet_rust::Result;

async fn main_async() -> Result<()> {
    TUNet::from_args().run().await
}

#[cfg(feature = "cui")]
fn builder() -> tokio::runtime::Builder {
    tokio::runtime::Builder::new_multi_thread()
}

#[cfg(not(feature = "cui"))]
fn builder() -> tokio::runtime::Builder {
    tokio::runtime::Builder::new_current_thread()
}

fn main() -> Result<()> {
    builder().enable_all().build()?.block_on(main_async())
}
