use cfg_if::cfg_if;

mod ping;

cfg_if! {
    if #[cfg(windows)] {
        use crate::*;

        mod winrt;

        pub fn suggest(client: &HttpClient) -> NetState {
            match winrt::suggest() {
                NetState::Unknown => ping::suggest(client),
                state => state,
            }
        }
    } else {
        pub use ping::suggest;
    }
}
