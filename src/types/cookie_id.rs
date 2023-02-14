use crate::{error::ParseError, types::Id};
use core::fmt::{Debug, Display, Formatter};

pub(crate) const BAYEUX_BROWSER: &str = "BAYEUX_BROWSER";

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct CookieId(Id);

impl CookieId {
    #[inline(always)]
    pub(crate) fn gen() -> Self {
        Self(Id::gen())
    }

    #[inline(always)]
    pub(crate) fn parse(str: &str) -> Result<CookieId, ParseError<'_>> {
        Id::parse(str).map(Self)
    }
}

impl Debug for CookieId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for CookieId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
