use core::time::Duration;

pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(20);
pub(crate) const DEFAULT_INTERVAL: Duration = Duration::new(0, 0);
pub(crate) const DEFAULT_MAX_INTERVAL: Duration = Duration::from_secs(60);
pub(crate) const DEFAULT_CHANNEL_CAPACITY: usize = 500;
pub(crate) const DEFAULT_STORAGE_CAPACITY: usize = 10_000;
