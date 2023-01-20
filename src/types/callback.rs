use crate::LongPollingServiceContext;
use core::{
    fmt::{Debug, Formatter},
    future::Future,
    pin::Pin,
};
use std::sync::Arc;

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;
type SyncCallback<AdditionalData, T> =
    Box<dyn Fn(&Arc<LongPollingServiceContext<AdditionalData>>, T) + Send + Sync + 'static>;
type AsyncCallback<AdditionalData, T> = Box<
    dyn Fn(&Arc<LongPollingServiceContext<AdditionalData>>, T) -> BoxedFuture
        + Send
        + Sync
        + 'static,
>;

#[derive(Default)]
pub(crate) enum Callback<AdditionalData, T> {
    #[default]
    Empty,
    Sync(SyncCallback<AdditionalData, T>),
    Async(AsyncCallback<AdditionalData, T>),
}

impl<AdditionalData, T> Debug for Callback<AdditionalData, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let name = match *self {
            Callback::Empty => "Empty",
            Callback::Sync(ref _callback) => "Sync",
            Callback::Async(ref _callback) => "Async",
        };

        f.debug_struct("Callback").field("self", &name).finish()
    }
}

impl<AdditionalData, T> Callback<AdditionalData, T> {
    #[inline(always)]
    pub(crate) fn new_sync<F>(callback: F) -> Self
    where
        F: Fn(&Arc<LongPollingServiceContext<AdditionalData>>, T) + Send + Sync + 'static,
    {
        Self::Sync(Box::new(callback))
    }

    #[inline(always)]
    pub(crate) fn new_async<F, Fut>(callback: F) -> Self
    where
        T: 'static,
        F: Fn(&Arc<LongPollingServiceContext<AdditionalData>>, T) -> Fut + Sync + Send + 'static,
        Fut: Future<Output = ()> + Sync + Send + 'static,
    {
        Self::Async(Box::new(move |context, arg| {
            Box::pin(callback(context, arg))
        }))
    }

    pub(crate) async fn call(
        &self,
        context: &Arc<LongPollingServiceContext<AdditionalData>>,
        argument: T,
    ) {
        match *self {
            Callback::Empty => {}
            Callback::Sync(ref func) => func(context, argument),
            Callback::Async(ref afunc) => afunc(context, argument).await,
        }
    }
}
