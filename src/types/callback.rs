use core::{
    fmt::{Debug, Formatter},
    future::Future,
    pin::Pin,
};

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;

#[derive(Default)]
pub(crate) enum Callback<T> {
    #[default]
    Empty,
    Sync(Box<dyn Fn(T) + Send + Sync + 'static>),
    Async(Box<dyn Fn(T) -> BoxedFuture + Send + Sync + 'static>),
}

impl<T> Debug for Callback<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let name = match *self {
            Callback::Empty => "Empty",
            Callback::Sync(ref _callback) => "Sync",
            Callback::Async(ref _callback) => "Async",
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

    pub(crate) async fn call(&self, argument: T)
    where
        T: Send + Sync,
    {
        match *self {
            Callback::Empty => {}
            Callback::Sync(ref func) => func(argument),
            Callback::Async(ref afunc) => afunc(argument).await,
        }
    }
}
