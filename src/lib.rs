#![deny(unused_must_use)]
#![warn(
    rust_2018_idioms,
    rust_2021_compatibility,
    missing_debug_implementations,
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    clippy::unwrap_used,
    clippy::if_let_mutex
)]

mod consts;
mod context;
mod error;
pub mod ext;
mod handlers;
mod messages;
mod types;

pub use {context::*, error::*, types::*};
