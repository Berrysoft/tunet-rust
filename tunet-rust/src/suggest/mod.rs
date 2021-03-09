use crate::*;
use cfg_if::cfg_if;

mod ping;

cfg_if! {
    if #[cfg(windows)] {
        mod winrt;

        pub fn suggest(client: &HttpClient) -> NetState {
            match winrt::suggest().unwrap_or(NetState::Unknown) {
                NetState::Unknown => ping::suggest(client),
                state => state,
            }
        }
    } else {
        pub use ping::suggest;
    }
}
