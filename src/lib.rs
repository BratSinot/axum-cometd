#![deny(unused_must_use)]
#![warn(
    rust_2018_idioms,
    rust_2021_compatibility,
    missing_docs,
    missing_debug_implementations,
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    clippy::unwrap_used,
    clippy::if_let_mutex
)]

//! This crate aims to make ability to use CometD protocol in servers written in Rust.
//!
//! This project is in progress and might change a lot from version to version.
//!

mod context;
mod error;
mod handlers;
mod messages;
mod types;

pub(crate) use types::*;
pub use {context::*, error::*};
