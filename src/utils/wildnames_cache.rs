use crate::{types::ChannelId, utils::get_wild_names};
use ahash::{AHashMap, AHashSet};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub(crate) struct WildNamesCache {
    // TODO: Replace on Arc<[String]>
    cache: RwLock<AHashMap<ChannelId, Arc<VecDeque<String>>>>,
}

impl WildNamesCache {
    pub(crate) async fn fetch_wildnames(&self, name: &str) -> Arc<VecDeque<ChannelId>> {
        let read_guard = self.cache.read().await;
        if let Some(wildnames) = read_guard.get(name).cloned() {
            wildnames
        } else {
            drop(read_guard);

            let mut write_guard = self.cache.write().await;
            write_guard.get(name).cloned().map_or_else(
                || {
                    Arc::clone(
                        write_guard
                            .entry(name.to_owned())
                            .or_insert_with(|| Arc::new(get_wild_names(name))),
                    )
                },
                |wildnames| wildnames,
            )
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
