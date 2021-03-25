mod bindings {
    ::windows::include_bindings!();
}

pub mod windows {
    pub use crate::bindings::windows::*;
    pub use ::windows::*;
}
