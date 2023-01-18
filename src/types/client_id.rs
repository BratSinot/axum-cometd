use crate::types::{Id, ZERO_ID};
use core::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Serialize};

pub(crate) const ZERO_CLIENT_ID: ClientId = ClientId(ZERO_ID);

/// CometD ClientId.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct ClientId(Id);

impl ClientId {
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
