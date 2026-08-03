//! Client-side request pacing for the undocumented Halo Infinite API.
//!
//! Halo Waypoint is an unofficial target, so bursts of traffic risk throttling or account
//! flagging. This limiter spaces requests evenly rather than allowing bursts, and tracks each
//! origin independently so a flurry of calls to one service does not stall another.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::Instant;

/// Evenly paces outgoing requests to at most `requests_per_second` per origin and applies
/// server-requested cooldowns after rate-limit responses.
///
/// Cloning shares the same underlying schedule, so every clone of a client draws from one budget
/// per origin.
#[derive(Debug, Clone)]
pub(crate) struct RateLimiter {
    /// Minimum spacing between two requests to the same origin, or `None` when disabled.
    interval: Option<Duration>,
    /// Next instant a request is permitted, keyed by origin base URL.
    next_allowed: Arc<Mutex<HashMap<String, Instant>>>,
}

impl RateLimiter {
    /// Builds a limiter that allows `requests_per_second` requests per origin. `0` disables pacing.
    pub(crate) fn per_second(requests_per_second: u32) -> Self {
        let interval = (requests_per_second > 0)
            .then(|| Duration::from_secs_f64(1.0 / f64::from(requests_per_second)));
        Self {
            interval,
            next_allowed: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Waits until a request to `origin` is permitted, reserving that slot for this caller.
    ///
    /// Reservations stack: concurrent callers targeting the same origin each claim the next slot
    /// under the lock, so they wake evenly spaced rather than all at once.
    pub(crate) async fn acquire(&self, origin: &str) {
        let now = Instant::now();
        let wait_until = {
            let mut next_allowed = self.next_allowed.lock().await;
            let slot = match next_allowed.get(origin) {
                Some(&prev) if prev > now => prev,
                _ => now,
            };
            let next = self.interval.map_or(slot, |interval| slot + interval);
            next_allowed.insert(origin.to_string(), next);
            slot
        };
        if wait_until > now {
            tokio::time::sleep_until(wait_until).await;
        }
    }

    /// Defers all subsequent requests to `origin` for at least `delay`.
    ///
    /// The delay is shared by all clones of this limiter. Existing reservations are preserved if
    /// they already extend beyond the requested cooldown.
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
    async fn spaces_same_origin_requests_by_the_interval() {
        // 100/s → 10ms spacing. Five calls incur four gaps ≈ 40ms; assert a safe lower bound.
        let limiter = RateLimiter::per_second(100);
        let start = Instant::now();

        for _ in 0..5 {
            limiter.acquire("https://skill.example").await;
        }

        assert!(
            start.elapsed() >= Duration::from_millis(35),
            "five paced requests should take at least ~40ms, took {:?}",
            start.elapsed()
        );
    }

    #[tokio::test]
    async fn tracks_origins_independently() {
        // Each origin's first request is immediate regardless of traffic to the other, so two
        // distinct origins complete promptly even at a slow rate.
        let limiter = RateLimiter::per_second(1);
        let start = Instant::now();

        limiter.acquire("https://skill.example").await;
        limiter.acquire("https://stats.example").await;

        assert!(start.elapsed() < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn zero_rate_never_delays() {
        let limiter = RateLimiter::per_second(0);
        let start = Instant::now();

        for _ in 0..1000 {
            limiter.acquire("https://skill.example").await;
        }

        assert!(start.elapsed() < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn backoff_delays_every_request_to_the_same_origin() {
        let limiter = RateLimiter::per_second(0);
        limiter
            .backoff("https://skill.example", Duration::from_millis(50))
            .await;

        let start = Instant::now();
        limiter.acquire("https://skill.example").await;

        assert!(
            start.elapsed() >= Duration::from_millis(45),
            "backoff should delay requests even when normal pacing is disabled"
        );
    }
}
