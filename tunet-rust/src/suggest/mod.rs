use cfg_if::cfg_if;

mod ping;

cfg_if! {
    if #[cfg(any(windows, macos))] {
        use crate::*;

        #[cfg(windows)]
        mod winrt;

        #[cfg(macos)]
        mod macos;

        mod platform {
            #[cfg(windows)]
            pub use winrt::*;

            #[cfg(macos)]
            pub use macos::*;
        }

        pub fn suggest(client: &HttpClient) -> NetState {
            match platform::suggest() {
                NetState::Unknown => ping::suggest(client),
                state => state,
            }
        }
    } else {
        pub use ping::suggest;
    }
}
