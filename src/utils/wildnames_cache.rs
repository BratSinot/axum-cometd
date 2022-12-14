use crate::{types::ChannelId, utils::get_wild_names};
use ahash::{AHashMap, AHashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub(crate) struct WildNamesCache {
    // TODO: Replace on Arc<[String]>
    cache: RwLock<AHashMap<ChannelId, Arc<Vec<String>>>>,
}

impl WildNamesCache {
    pub(crate) async fn fetch_wildnames(&self, name: &str) -> Arc<Vec<ChannelId>> {
        let read_guard = self.cache.read().await;
        if let Some(wildnames) = read_guard.get(name) {
            wildnames.clone()
        } else {
            drop(read_guard);
            let mut write_guard = self.cache.write().await;
            write_guard
                .entry(name.to_string())
                .or_insert_with(|| Arc::new(get_wild_names(name)))
                .clone()
        }
    }

    #[inline(always)]
    pub(crate) async fn remove_wildnames(&self, mut names: AHashSet<ChannelId>) {
        self.cache
            .write()
            .await
            .retain(|name, _| names.remove(name));
    }
}
