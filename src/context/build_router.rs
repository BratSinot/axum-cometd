use crate::{handlers::*, LongPollingServiceContext};
use axum::{routing::post, Extension, Router};
use core::fmt::Debug;
use std::sync::Arc;

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
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use axum_cometd::RouterBuilder;
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new().build::<()>(Arc::clone(&context));
    /// ```
    #[inline(always)]
    pub fn build<CustomData>(
        self,
        context: Arc<LongPollingServiceContext<(), CustomData>>,
    ) -> Router
    where
        CustomData: Send + Sync + 'static,
    {
        self.build_with_additional_data(context)
            .layer(Extension(()))
    }

    /// Return a `axum::Router`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use axum::Extension;
    /// use axum_cometd::RouterBuilder;
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build::<ContextData, ()>();
    /// #[derive(Clone)]
    /// struct ContextData {
    ///     server_name: String,
    /// }
    ///
    /// let app = RouterBuilder::new()
    ///     .build_with_additional_data(Arc::clone(&context))
    ///     .layer(Extension(ContextData {
    ///         server_name: std::env::var("SERVER_NAME").unwrap_or_else(|_| "Skalica".to_owned()),
    ///     }));
    /// ```
    #[inline]
    pub fn build_with_additional_data<AdditionalData, CustomData>(
        self,
        context: Arc<LongPollingServiceContext<AdditionalData, CustomData>>,
    ) -> Router
    where
        AdditionalData: Clone + Send + Sync + 'static,
        CustomData: Send + Sync + 'static,
    {
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
            .with_state(context)
    }

    /// Set subscribe base-path for routers.
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use axum_cometd::RouterBuilder;
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new()
    ///     // Ex: `/` -> `/bar`
    ///     .subscribe_base_path("/bar")
    ///     .build::<()>(Arc::clone(&context));
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn subscribe_base_path(self, path: &'static str) -> Self {
        Self {
            subscribe_base_path: path,
            ..self
        }
    }

    /// Set handshake base-path for routers.
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use axum_cometd::RouterBuilder;
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new()
    ///     // Ex: `/handshake` -> `/bar/handshake`
    ///     .handshake_base_path("/bar")
    ///     .build::<()>(Arc::clone(&context));
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn handshake_base_path(self, path: &'static str) -> Self {
        Self {
            handshake_base_path: path,
            ..self
        }
    }

    /// Set connect base-path for routers.
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use axum_cometd::RouterBuilder;
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new()
    ///     // Ex: `/connect` -> `/bar/connect`
    ///     .connect_base_path("/bar")
    ///     .build::<()>(Arc::clone(&context));
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn connect_base_path(self, path: &'static str) -> Self {
        Self {
            connect_base_path: path,
            ..self
        }
    }

    /// Set disconnect base-path for routers.
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use axum_cometd::RouterBuilder;
    ///
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build();
    /// let app = RouterBuilder::new()
    ///     // Ex: `/disconnect` -> `/bar/disconnect`
    ///     .disconnect_base_path("/bar")
    ///     .build::<()>(Arc::clone(&context));
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn disconnect_base_path(self, path: &'static str) -> Self {
        Self {
            disconnect_base_path: path,
            ..self
        }
    }
}
