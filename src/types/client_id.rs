use crate::types::Id;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

/// CometD ClientId.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct ClientId(Id);

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

#[derive(Debug)]
pub(crate) struct ClientIdGen(ClientId);

impl ClientIdGen {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self(ClientId(Id::rand()))
    }

    #[inline]
    pub(crate) fn next(&mut self) -> ClientId {
        let ret = self.0;
        self.0 .0.rotr();
        ret
    }
}
