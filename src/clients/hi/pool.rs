use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use thiserror::Error;

use super::HaloInfiniteClient;

struct PoolState {
    clients: Vec<HaloInfiniteClient>,
    next: AtomicUsize,
}

/// Selects Halo Infinite clients in round-robin order.
///
/// Clones share the selection cursor. Each client retains its own credentials,
/// rate limiter, and retries.
#[derive(Clone)]
pub struct HaloInfiniteClientPool {
    state: Arc<PoolState>,
}

/// Error constructing a Halo Infinite client pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum HaloInfiniteClientPoolError {
    #[error("a Halo Infinite client pool needs at least one account")]
    Empty,
}

impl HaloInfiniteClientPool {
    /// Creates a nonempty pool of clients for distinct Halo accounts.
    pub fn try_new(
        clients: impl IntoIterator<Item = HaloInfiniteClient>,
    ) -> Result<Self, HaloInfiniteClientPoolError> {
        let clients: Vec<_> = clients.into_iter().collect();
        if clients.is_empty() {
            return Err(HaloInfiniteClientPoolError::Empty);
        }
        Ok(Self {
            state: Arc::new(PoolState {
                clients,
                next: AtomicUsize::new(0),
            }),
        })
    }

    /// Returns the next client. Its clone shares credentials and pacing with
    /// the pool entry; clients in 429 backoff can still be selected.
    pub fn next_client(&self) -> HaloInfiniteClient {
        let index = self.state.next.fetch_add(1, Ordering::Relaxed) % self.state.clients.len();
        self.state.clients[index].clone()
    }

    /// Number of accounts in the pool.
    pub fn account_count(&self) -> usize {
        self.state.clients.len()
    }
}
