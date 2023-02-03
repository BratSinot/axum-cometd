pub(crate) trait CheckExt<T: ?Sized> {
    fn check<E>(&self, other: &T, error: E) -> Result<(), E>;

    fn check_or<E>(&self, other: &T, error: impl FnOnce() -> E) -> Result<(), E>;
}

impl<T> CheckExt<T> for T
where
    T: PartialEq,
{
    fn check<E>(&self, other: &T, error: E) -> Result<(), E> {
        self.eq(other).then_some(()).ok_or(error)
    }

    fn check_or<E>(&self, other: &T, error: impl FnOnce() -> E) -> Result<(), E> {
        self.eq(other).then_some(()).ok_or_else(error)
    }
}

impl CheckExt<str> for Option<String> {
    fn check<E>(&self, other: &str, error: E) -> Result<(), E> {
        self.as_deref().eq(&Some(other)).then_some(()).ok_or(error)
    }

    fn check_or<E>(&self, other: &str, error: impl FnOnce() -> E) -> Result<(), E> {
        self.as_deref()
            .eq(&Some(other))
            .then_some(())
            .ok_or_else(error)
    }
}
