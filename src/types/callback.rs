use std::{
    fmt::{Debug, Formatter},
    future::Future,
    pin::Pin,
};

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;

pub(crate) enum Callback<T> {
    Empty,
    Sync(Box<dyn Fn(T) + Send + Sync + 'static>),
    Async(Box<dyn Fn(T) -> BoxedFuture + Send + Sync + 'static>),
}

impl<T> Default for Callback<T> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<T> Debug for Callback<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Callback::Empty => "Empty",
            Callback::Sync(_) => "Sync",
            Callback::Async(_) => "Async",
        };

        f.debug_struct("Callback").field("self", &name).finish()
    }
}

impl<T> Callback<T> {
    #[inline(always)]
    pub(crate) fn new_sync<F>(callback: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        Self::Sync(Box::new(callback))
    }

    #[inline(always)]
    pub(crate) fn new_async<F, Fut>(callback: F) -> Self
    where
        T: 'static,
        F: Fn(T) -> Fut + Sync + Send + 'static,
        Fut: Future<Output = ()> + Sync + Send + 'static,
    {
        Self::Async(Box::new(move |arg| Box::pin(callback(arg))))
    }

    pub(crate) async fn call(&self, argument: T) {
        match self {
            Callback::Empty => {}
            Callback::Sync(func) => func(argument),
            Callback::Async(afunc) => afunc(argument).await,
        }
    }
}
