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
    clippy::if_let_mutex,
    clippy::std_instead_of_core,
    clippy::missing_const_for_fn,
    clippy::future_not_send,
    clippy::str_to_string,
    clippy::clone_on_ref_ptr,
    clippy::panic,
    clippy::explicit_iter_loop,
    clippy::pattern_type_mismatch,
    clippy::indexing_slicing,
    clippy::use_debug,
    clippy::unnested_or_patterns,
    clippy::return_self_not_must_use,
    clippy::map_unwrap_or,
    clippy::items_after_statements,
    clippy::needless_pass_by_value,
    clippy::if_not_else,
    clippy::option_if_let_else
)]

//! This crate aims to make ability to use CometD protocol in servers written in Rust.
//!
//! This project is in progress and might change a lot from version to version.
//!

mod context;
mod handlers;
mod types;
mod utils;

pub use {context::*, types::error::*, types::*};
