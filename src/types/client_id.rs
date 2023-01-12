use crate::types::Id;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
