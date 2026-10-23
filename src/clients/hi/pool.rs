//! Best-effort load balancing across independent Halo Infinite accounts.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use thiserror::Error;

use super::HaloInfiniteClient;

struct PoolState {
    clients: Vec<HaloInfiniteClient>,
    next: AtomicUsize,
}

/// A round-robin pool of independently authenticated Halo Infinite clients.
///
/// Call [`Self::next_client`] once per logical operation and use the returned
/// client throughout that operation. This keeps account-specific responses and
/// clearance credentials tied to one account. Each client retains its own
/// request pacing, token cache, and 429 retry state; clones of this pool share
/// the selection order within one process. Separate processes or Lambda
/// environments do not coordinate their request rates.
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
    /// Builds a pool from clients that are interchangeable for the operations
    /// you intend to distribute. Each client should use a distinct Halo account.
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

    /// Selects the next account and returns a cheap client clone whose token
    /// cache, pacing, and cooldown remain shared with that account's pool entry.
    ///
    /// Selection is round-robin within this process. It does not check whether
    /// an account is currently waiting on a 429 cooldown; the selected client's
    /// limiter and retry handling still apply before its outgoing requests.
    pub fn next_client(&self) -> HaloInfiniteClient {
        let index = self.state.next.fetch_add(1, Ordering::Relaxed) % self.state.clients.len();
        self.state.clients[index].clone()
    }

    /// Number of accounts in the pool.
    pub fn account_count(&self) -> usize {
        self.state.clients.len()
    }
}
