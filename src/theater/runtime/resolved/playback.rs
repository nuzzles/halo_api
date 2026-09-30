//! Playback support for ResolvedFilm.
use super::*;
#[derive(Debug, Clone)]
pub(super) struct Checkpoint {
    pub(super) next_event: usize,
    pub(super) world: WorldSnapshot,
}

pub(super) const CHECKPOINT_INTERVAL: usize = 1024;

impl ResolvedFilm {
    /// O(1) borrowed access; iterating entities costs O(world size).
    pub fn current(&self) -> &WorldSnapshot {
        &self.world
    }
}

impl ResolvedFilm {
    /// None before any seek/advance, including for recordings starting at zero.
    pub fn timestamp_us(&self) -> Option<u64> {
        self.timestamp_us
    }
}

impl ResolvedFilm {
    /// Forward advancement applies intervening updates. Earlier timestamps seek.
    pub fn advance_to(&mut self, timestamp_us: u64) -> &WorldSnapshot {
        if self.timestamp_us.is_some_and(|t| timestamp_us < t) {
            return self.seek(timestamp_us);
        }
        let end = self
            .events
            .partition_point(|e| e.timestamp_us <= timestamp_us);
        self.apply_until(end);
        self.timestamp_us = Some(timestamp_us);
        &self.world
    }
}

impl ResolvedFilm {
    /// Restore the nearest checkpoint, then apply at most CHECKPOINT_INTERVAL
    /// indexed events. Lookup is O(log events); restoration is O(entity count).
    pub fn seek(&mut self, timestamp_us: u64) -> &WorldSnapshot {
        let end = self
            .events
            .partition_point(|e| e.timestamp_us <= timestamp_us);
        let c = self.checkpoints.partition_point(|c| c.next_event <= end) - 1;
        self.world = self.checkpoints[c].world.clone();
        self.next_event = self.checkpoints[c].next_event;
        self.apply_until(end);
        self.timestamp_us = Some(timestamp_us);
        &self.world
    }
}

impl ResolvedFilm {
    pub fn rewind(&mut self) {
        self.world = WorldSnapshot::default();
        self.next_event = 0;
        self.timestamp_us = None;
    }
}

impl ResolvedFilm {
    fn apply_until(&mut self, end: usize) {
        for event in &self.events[self.next_event..end] {
            apply(&mut self.world, event.change.as_ref());
        }
        self.next_event = end;
    }
}
