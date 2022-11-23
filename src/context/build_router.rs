use crate::{handlers::*, LongPoolingServiceContext};
use axum::{routing::post, Extension, Router};
use serde::Serialize;
use std::{fmt::Debug, sync::Arc, time::Duration};
use tower_http::timeout::TimeoutLayer;

/// A builder to construct `axum::Route` of CometD server.
#[derive(Debug)]
pub struct RouterBuilder {
    base_path: &'static str,
    subscribe_base_path: &'static str,
    handshake_base_path: &'static str,
    connect_base_path: &'static str,
    disconnect_base_path: &'static str,
}

impl Default for RouterBuilder {
    #[inline(always)]
    fn default() -> Self {
        Self {
            base_path: "/",
            subscribe_base_path: "/",
            handshake_base_path: "/",
            connect_base_path: "/",
            disconnect_base_path: "/",
        }
    }
}

impl RouterBuilder {
    /// Construct a new `RouterBuilder`.
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a `axum::Router`.
    ///
    /// # Example
    /// ```rust
    /// use axum_cometd::RouterBuilder;
    /// let app = RouterBuilder::new().build(&context);
    /// ```
    #[inline]
    pub fn build<Msg>(self, context: &Arc<LongPoolingServiceContext<Msg>>) -> Router
    where
        Msg: Debug + Clone + Send + Serialize + 'static,
    {
        let Self {
            base_path,
            subscribe_base_path,
            handshake_base_path,
            connect_base_path,
            disconnect_base_path,
        } = self;

        let subscribe_route = Router::new().route(subscribe_base_path, post(subscribe::<Msg>));
        let handshake_route = Router::new().nest(
            handshake_base_path,
            Router::new().route("/handshake", post(handshake::<Msg>)),
        );
        let connect_route = Router::new().nest(
            connect_base_path,
            Router::new().route("/connect", post(connect::<Msg>)),
        );
        let disconnect_route = Router::new().nest(
            disconnect_base_path,
            Router::new().route("/disconnect", post(disconnect::<Msg>)),
        );

        Router::new()
            .nest(
                base_path,
                Router::new()
                    .merge(subscribe_route)
                    .merge(handshake_route)
                    .merge(connect_route)
                    .merge(disconnect_route)
                    .layer(Extension(context.clone())),
            )
            .layer(TimeoutLayer::new(Duration::from_millis(
                context.consts.timeout_ms,
            )))
    }

    /// Set root base-path for routers.
    ///
    /// # Example
    /// ```rust
    /// use axum_cometd::RouterBuilder;
    /// let app = RouterBuilder::new()
    ///     // Ex: `/handshake` -> `/foo/handshake`
    ///     .base_path("/foo/")
    ///     .build(&context);
    /// ```
    #[inline(always)]
    pub fn base_path(self, path: &'static str) -> Self {
        Self {
            base_path: path,
            ..self
        }
    }

    /// Set subscribe base-path for routers.
    ///
    /// # Example
    /// ```rust
    /// use axum_cometd::RouterBuilder;
    /// let app = RouterBuilder::new()
    ///     .base_path("/foo/")
    ///     // Ex: `/foo` -> `/foo/bar`
    ///     .subscribe_base_path("/bar/")
    ///     .build(&context);
    /// ```
    #[inline(always)]
    pub fn subscribe_base_path(self, path: &'static str) -> Self {
        Self {
            subscribe_base_path: path,
            ..self
        }
    }

    /// Set handshake base-path for routers.
    ///
    /// # Example
    /// ```rust
    /// use axum_cometd::RouterBuilder;
    /// let app = RouterBuilder::new()
    ///     .base_path("/foo/")
    ///     // Ex: `/foo/handshake` -> `/foo/bar/handshake`
    ///     .handshake_base_path("/bar/")
    ///     .build(&context);
    /// ```
    #[inline(always)]
    pub fn handshake_base_path(self, path: &'static str) -> Self {
        Self {
            handshake_base_path: path,
            ..self
        }
    }

    /// Set connect base-path for routers.
    ///
    /// # Example
    /// ```rust
    /// use axum_cometd::RouterBuilder;
    /// let app = RouterBuilder::new()
    ///     .base_path("/foo/")
    ///     // Ex: `/foo/connect` -> `/foo/bar/connect`
    ///     .connect_base_path("/bar/")
    ///     .build(&context);
    /// ```
    #[inline(always)]
    pub fn connect_base_path(self, path: &'static str) -> Self {
        Self {
            connect_base_path: path,
            ..self
        }
    }

    /// Set disconnect base-path for routers.
    ///
    /// # Example
    /// ```rust
    /// use axum_cometd::RouterBuilder;
    /// let app = RouterBuilder::new()
    ///     .base_path("/foo/")
    ///     // Ex: `/foo/disconnect` -> `/foo/bar/disconnect`
    ///     .disconnect_base_path("/bar/")
    ///     .build(&context);
    /// ```
    #[inline(always)]
    pub fn disconnect_base_path(self, path: &'static str) -> Self {
        Self {
            disconnect_base_path: path,
            ..self
        }
    }
}
