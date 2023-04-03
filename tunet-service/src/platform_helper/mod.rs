cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[path = "winrt.rs"]
        mod platform;
    }  else {
        #[path = "stub.rs"]
        mod platform;
    }
}

use tokio_stream::Stream;
use tunet_helper::{NetFlux, Result};

#[allow(clippy::needless_question_mark)]
pub fn watch() -> Result<impl Stream<Item = ()>> {
    Ok(platform::watch()?)
}

#[allow(clippy::needless_question_mark)]
pub fn succeeded(flux: NetFlux) -> Result<()> {
    Ok(platform::succeeded(flux)?)
}
