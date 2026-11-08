//! Shared cooldowns after Halo Waypoint responds with 429.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::Instant;

/// Pauses requests to an origin after rate-limit responses.
///
/// Cloning shares cooldowns across requests from the same client.
#[derive(Debug, Clone)]
pub(crate) struct RateLimiter {
    /// Next instant a request is permitted, keyed by origin base URL.
    next_allowed: Arc<Mutex<HashMap<String, Instant>>>,
}

impl RateLimiter {
    pub(crate) fn new() -> Self {
        Self {
            next_allowed: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Waits for any active cooldown on `origin` to expire.
    pub(crate) async fn acquire(&self, origin: &str) {
        loop {
            let wait_until = self.next_allowed.lock().await.get(origin).copied();
            match wait_until {
                Some(until) if until > Instant::now() => tokio::time::sleep_until(until).await,
                _ => return,
            }
        }
    }

    /// Defers all subsequent requests to `origin` for at least `delay`.
    ///
    /// The delay is shared by all clones of this limiter.
    pub(crate) async fn backoff(&self, origin: &str, delay: Duration) {
        let blocked_until = Instant::now() + delay;
        let mut next_allowed = self.next_allowed.lock().await;
        next_allowed
            .entry(origin.to_string())
            .and_modify(|next| *next = (*next).max(blocked_until))
            .or_insert(blocked_until);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tracks_origins_independently() {
        let limiter = RateLimiter::new();
        limiter
            .backoff("https://skill.example", Duration::from_secs(1))
            .await;
        let start = Instant::now();
        limiter.acquire("https://stats.example").await;
        assert!(start.elapsed() < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn requests_without_a_cooldown_are_not_paced() {
        let limiter = RateLimiter::new();
        let start = Instant::now();

        for _ in 0..1000 {
            limiter.acquire("https://skill.example").await;
        }

        assert!(start.elapsed() < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn backoff_delays_every_request_to_the_same_origin() {
        let limiter = RateLimiter::new();
        limiter
            .backoff("https://skill.example", Duration::from_millis(50))
            .await;

        let start = Instant::now();
        limiter.acquire("https://skill.example").await;

        assert!(
            start.elapsed() >= Duration::from_millis(45),
            "backoff should delay requests to the affected origin"
        );
    }

    #[tokio::test]
    async fn extended_cooldown_is_observed_by_waiting_requests() {
        let limiter = RateLimiter::new();
        limiter
            .backoff("https://skill.example", Duration::from_millis(40))
            .await;
        let waiting = limiter.clone();
        let start = Instant::now();
        let request = tokio::spawn(async move { waiting.acquire("https://skill.example").await });
        tokio::time::sleep(Duration::from_millis(10)).await;
        limiter
            .backoff("https://skill.example", Duration::from_millis(50))
            .await;
        request.await.unwrap();
        assert!(start.elapsed() >= Duration::from_millis(55));
    }
}
