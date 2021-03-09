use crate::*;

mod ping;

#[cfg(feature = "winrt")]
mod winrt;

#[cfg(feature = "winrt")]
pub fn suggest(client: &HttpClient) -> NetState {
    match winrt::suggest().unwrap_or(NetState::Unknown) {
        NetState::Unknown => ping::suggest(client),
        state => state,
    }
}

#[cfg(not(feature = "winrt"))]
pub use ping::suggest;
