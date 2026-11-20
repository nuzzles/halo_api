use std::future::Future;

use chrono::{DateTime, Duration, Utc};
use tokio::sync::Mutex;

pub(crate) struct CachedToken<T> {
    pub value: T,
    pub expires_at: DateTime<Utc>,
}

impl<T> CachedToken<T> {
    pub fn new(value: T, expires_at: DateTime<Utc>) -> Self {
        Self { value, expires_at }
    }
}

/// Serializes refreshes, including after a concurrent cache miss. A cancelled
/// refresh releases the lock so another caller can retry.
pub(crate) struct ExpiryTokenCache<T> {
    slot: Mutex<Option<CachedToken<T>>>,
}

impl<T: Clone> ExpiryTokenCache<T> {
    pub fn new() -> Self {
        Self {
            slot: Mutex::new(None),
        }
    }

    pub async fn get_or_refresh<E, F, Fut>(&self, fetch: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<CachedToken<T>, E>>,
    {
        let mut slot = self.slot.lock().await;
        if let Some(cached) = slot.as_ref() {
            if cached.expires_at > Utc::now() + Duration::minutes(1) {
                return Ok(cached.value.clone());
            }
        }
        *slot = None;
        let token = fetch().await?;
        let value = token.value.clone();
        *slot = Some(token);
        Ok(value)
    }

    pub async fn invalidate(&self) {
        *self.slot.lock().await = None;
    }
}
