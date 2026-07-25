cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[path = "windows.rs"]
        mod sys;
    } else {
        #[path = "stub.rs"]
        mod sys;
    }
}

pub use sys::*;
