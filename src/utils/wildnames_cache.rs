use crate::{types::ChannelId, utils::get_wild_names};
use ahash::{AHashMap, AHashSet};
use std::collections::VecDeque;
use std::sync::{Arc, PoisonError, RwLock};

#[derive(Debug, Default)]
pub(crate) struct WildNamesCache {
    // TODO: Replace on Arc<[String]>
    cache: RwLock<AHashMap<ChannelId, Arc<VecDeque<String>>>>,
}

impl WildNamesCache {
    pub(crate) fn fetch_wildnames(&self, name: &str) -> Arc<VecDeque<ChannelId>> {
        if let Some(wildnames) = self
            .cache
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(name)
            .cloned()
        {
            return wildnames;
        }

        Arc::clone(
            self.cache
                .write()
                .unwrap_or_else(PoisonError::into_inner)
                .entry(name.to_owned())
                .or_insert_with(|| Arc::new(get_wild_names(name))),
        )
    }

    #[inline(always)]
    pub(crate) fn remove_wildnames(&self, mut names: AHashSet<ChannelId>) {
        self.cache
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|name, _| names.remove(name));
    }
}
