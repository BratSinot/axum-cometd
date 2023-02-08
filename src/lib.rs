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
//! # Table of contents
//! - [Server endpoints](#server-endpoints)
//! - [`clientId` and `BAYEUX_BROWSER` cookie](#clientId-bayeux-browser-cookie)
//! - [How server works](#how-server-works)
//!
//! # Server endpoints
//!
//! Server have 4 endpoints:
//! 1) `/handshake` -- to register and get `clientId`;
//! 2) `/` -- to subscribe on channels;
//! 3) `/connect` -- to receiving or publish messages;
//! 4) `/disconnect` -- to say to server clean data for `clientId`;
//!
//! You can change base part of these endpoints through
//! [`RouterBuilder::handshake_base_path`],
//! [`RouterBuilder::subscribe_base_path`],
//! [`RouterBuilder::connect_base_path`],
//! [`RouterBuilder::disconnect_base_path`].
//! For example, to make `/node/0/handshake` and `/node/1/connect` you can do this:
//! ```rust
//! use std::sync::Arc;
//! use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
//!
//! let context = LongPollingServiceContextBuilder::new()
//!     .build();
//!
//! let service = RouterBuilder::new()
//!     .handshake_base_path("/node/0")
//!     .connect_base_path("/node/1")
//!     .build(Arc::clone(&context));
//! ```
//!
//! # `clientId` and `BAYEUX_BROWSER` cookie
//!
//! `clientId` and `BAYEUX_BROWSER` cookie is 40-character length hex string,
//! with possibility of leading zeroes.
//! Server will return '402::session_unknown' error if it will be not.
//! To get some uniquity first 8 bytes is taken from Unix timestamp, and for randomness
//! last part filled with random numbers.
//!
//! # How server works
//!
//! `BAYEUX_BROWSER` cookie will be generated and set at `/handshake` request,
//! if there isn't one already.
//!
//! At others endpoints ([Server endpoints]) server check `clientId` and `BAYEUX_BROWSER` cookie
//! (in case of publish messages to `/connect` it will be check each `clientId`).
//! If `clientId` will be used with different `BAYEUX_BROWSER` cookie,
//! server will return '402::session_unknown' error.

mod context;
mod ext;
mod handlers;
mod types;
mod utils;

use std::sync::Arc;

pub(crate) use ext::*;
pub use {context::*, types::error::*, types::*};

#[allow(missing_docs)]
pub type Sender<AdditionalData, CustomData> =
    async_broadcast::Sender<Arc<Event<AdditionalData, CustomData>>>;
#[allow(missing_docs)]
pub type Receiver<AdditionalData, CustomData> =
    async_broadcast::Receiver<Arc<Event<AdditionalData, CustomData>>>;
