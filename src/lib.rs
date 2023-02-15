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
//! - [How get server events](#how-get-server-events)
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
//! ```rust,no_run
//! use std::sync::Arc;
//! use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
//!
//! let context = LongPollingServiceContextBuilder::new()
//!     .build();
//! # let context: Arc<axum_cometd::LongPollingServiceContext<(), ()>> = context;
//!
//! let router = RouterBuilder::new()
//!     .handshake_base_path("/node/0")
//!     .connect_base_path("/node/1")
//!     .build(Arc::clone(&context));
//!
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
//!
//! # How get server events
//!
//! Server have 3 events:
//! 1) [`Event::SessionAdded`]
//! 2) [`Event::Subscribe`]
//! 3) [`Event::SessionRemoved`]
//! 4) [`Event::CustomData`]
//!
//! `SessionAdded` and `Subscribe` can contain additional data, which will be attached through
//! [`axum::Extension`].
//! To get those events, you must use get receive channel [`LongPollingServiceContext::rx`].
//! Server do not use [`Event::CustomData`], it user custom message which can be received in
//! receiver.
//! ```rust,no_run
//! use std::sync::Arc;
//! use axum::Extension;
//! use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
//!
//! #[derive(Debug, Clone)]
//! struct ContextData {
//!     server_name: Arc<str>,
//! }
//!
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() {
//! use std::time::Duration;
//! use axum_cometd::Event;
//! let context = LongPollingServiceContextBuilder::new()
//!     .build::<ContextData, &'static str>();
//!
//! let app = RouterBuilder::new()
//!     .build_with_additional_data(Arc::clone(&context))
//!     .layer(Extension(ContextData {
//!         server_name: std::env::var("SERVER_NAME")
//!             .map(Arc::from)
//!             .unwrap_or_else(|_| Arc::from("Skalica")),
//!     }));
//!
//! let tx = context.tx();
//! let mut rx = context.rx();
//!
//! tokio::task::spawn(async move {
//!     loop {
//!         tx.send("CUSTOM_DATA").await;
//!         tokio::time::sleep(Duration::from_secs(1)).await;
//!     }
//! });
//!
//! while let Some(event) = rx.recv().await {
//!     match *event {
//!         Event::SessionAdded{
//!             client_id,
//!             ref headers,
//!             ref data,
//!         } => {
//!             println!("sessionAdded with clientId({client_id}), headers({headers:?}), data({data:?})");
//!         }
//!         Event::Subscribe{
//!             client_id,
//!             ref headers,
//!             ref channels,
//!             ref data,
//!         } => {
//!             println!("subscribed on channels({channels:?}) with clientId({client_id}), headers({headers:?}), data({data:?})");
//!         }
//!         Event::SessionRemoved{
//!             client_id,
//!         } => println!("clientId({client_id}) session removed"),
//!         Event::CustomData(msg) => println!("got CustomData({msg})"),
//!     }
//! }
//! # }
//! ```

mod consts;
mod context;
mod ext;
mod handlers;
mod types;
mod utils;

pub(crate) use ext::*;
pub use {context::*, types::error::*, types::*};
