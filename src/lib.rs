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

mod context;
mod error;
mod handlers;
mod messages;
mod types;

pub mod ext;

pub use {context::*, error::*, types::*};
