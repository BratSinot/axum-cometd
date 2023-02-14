use crate::types::Id;
use core::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Serialize};

/// CometD ClientId.
#[derive(Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct ClientId(Id);

impl ClientId {
    #[inline(always)]
    pub(crate) fn zero() -> Self {
        Self(Id::zero())
    }

    #[inline(always)]
    pub(crate) fn gen() -> Self {
        Self(Id::gen())
    }
}

impl Debug for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
