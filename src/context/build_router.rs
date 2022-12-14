use crate::{handlers::*, LongPollingServiceContext};
use axum::{routing::post, Router};
use std::{fmt::Debug, sync::Arc, time::Duration};

/// A builder to construct `axum::Route` of CometD server.
#[derive(Debug)]
pub struct RouterBuilder {
    subscribe_base_path: &'static str,
    handshake_base_path: &'static str,
    connect_base_path: &'static str,
    disconnect_base_path: &'static str,
}

impl Default for RouterBuilder {
    #[inline(always)]
    fn default() -> Self {
        Self {
            subscribe_base_path: "/",
            handshake_base_path: "",
            connect_base_path: "",
            disconnect_base_path: "",
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
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new().build(&context);
    /// ```
    #[inline]
    pub fn build(self, context: &Arc<LongPollingServiceContext>) -> Router {
        let Self {
            subscribe_base_path,
            handshake_base_path,
            connect_base_path,
            disconnect_base_path,
        } = self;

        Router::new()
            .route(subscribe_base_path, post(subscribe))
            .route(&format!("{handshake_base_path}/handshake"), post(handshake))
            .route(&format!("{connect_base_path}/connect"), post(connect))
            .route(
                &format!("{disconnect_base_path}/disconnect"),
                post(disconnect),
            )
            .with_state(context.clone())
    }

    /// Set subscribe base-path for routers.
    ///
    /// # Example
    /// ```rust
    /// use axum_cometd::RouterBuilder;
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new()
    ///     // Ex: `/` -> `/bar`
    ///     .subscribe_base_path("/bar")
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
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new()
    ///     // Ex: `/handshake` -> `/bar/handshake`
    ///     .handshake_base_path("/bar")
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
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new()
    ///     // Ex: `/connect` -> `/bar/connect`
    ///     .connect_base_path("/bar")
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
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new()
    ///     // Ex: `/disconnect` -> `/bar/disconnect`
    ///     .disconnect_base_path("/bar")
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
