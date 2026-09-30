//! Owned Theater analysis and stateful playback over a canonical recording.
//!
//! Load once with [`TheaterRuntime::load`]. The resolved representation is private;
//! public events, evidence, and world models live in the supporting modules.

pub mod medals;
mod resolved;
use crate::theater::Film;
use resolved::ResolvedFilm;
use std::sync::Arc;

pub use resolved::EventFilter;
pub use resolved::events::*;
pub use resolved::identity::*;
pub use resolved::summary::*;
pub use resolved::world::*;
pub use resolved::{events, identity, interpretation, query, summary, world};

/// An owned canonical film, query indexes, and an independent playback cursor.
///
/// Cloning shares the film, events and checkpoints, but copies the current world
/// map in O(world size). Original chunk buffers are shared without copying; resolving derived state
/// can copy its field values.
#[derive(Debug, Clone)]
pub struct TheaterRuntime {
    resolved: ResolvedFilm,
}

impl TheaterRuntime {
    /// Take ownership of a canonical film and resolve its supported records.
    /// An `Arc<Film>` can be supplied to share the same recording across runtimes.
    /// Loading walks records, sorts events, and builds indexes/checkpoints; it is
    /// not constant time. Unknown data and interpretation uncertainty stay explicit.
    pub fn load(film: impl Into<Arc<Film>>) -> Self {
        Self {
            resolved: ResolvedFilm::from_film(film.into()),
        }
    }

    /// Borrow the unchanged canonical recording and its retained source bytes.
    pub fn film(&self) -> &Film {
        self.resolved.film()
    }
    /// Inspect explicitly labeled evidence derived from otherwise opaque regions.
    pub fn interpretations(&self) -> &interpretation::Interpretations {
        self.resolved.interpretations()
    }
    /// Borrow the complete chronological supported event stream, including state updates.
    pub fn events(&self) -> &[Event] {
        self.resolved.events()
    }
    /// Borrow typed chronological summaries with recorded actor fields and derivation.
    pub fn summary_events(&self) -> &[summary::SummaryEvent] {
        self.resolved.summary_events()
    }
    /// Compare each packet's declared summary count with recovered candidate counts.
    pub fn summary_reports(&self) -> &[SummaryReadReport] {
        self.resolved.summary_reports()
    }
    /// Filter indexed summary events without moving the playback cursor.
    pub fn query_summaries(&self, filter: SummaryFilter) -> impl Iterator<Item = &SummaryEvent> {
        self.resolved.query_summaries(filter)
    }
    /// Filter the full event stream without moving the playback cursor.
    pub fn query(&self, filter: EventFilter) -> impl Iterator<Item = &Event> {
        self.resolved.query(filter)
    }
    /// Follow a source reference to a structural record or an explicitly derived read.
    /// Invalid references return `None`; derived reads are not canonical Film records.
    pub fn record(&self, source: SourceRef) -> Option<Record<'_>> {
        self.resolved.record(source)
    }
    /// Look up an unambiguous bootstrap roster slot by recorded film index.
    pub fn player(&self, film_index: usize) -> Option<&PlayerTableSlot> {
        self.resolved.player(film_index)
    }

    /// O(1) borrowed access to the materialized snapshot. Enumerating/copying it
    /// costs O(world size); this method performs no seek or event application.
    pub fn current(&self) -> &WorldSnapshot {
        self.resolved.current()
    }
    /// Current requested playback time, or `None` before playback or after rewind.
    pub fn timestamp_us(&self) -> Option<u64> {
        self.resolved.timestamp_us()
    }
    /// Apply intervening updates; earlier timestamps use checkpoint seeking.
    pub fn advance_to(&mut self, timestamp_us: u64) -> &WorldSnapshot {
        self.resolved.advance_to(timestamp_us)
    }
    /// Binary-search events/checkpoints, copy the nearest checkpoint world, and
    /// apply at most 1,024 remaining events. Copy cost scales with world size.
    pub fn seek(&mut self, timestamp_us: u64) -> &WorldSnapshot {
        self.resolved.seek(timestamp_us)
    }
    /// Clear the current world and cursor while retaining shared indexes and evidence.
    pub fn rewind(&mut self) {
        self.resolved.rewind();
    }
}

#[cfg(test)]
mod tests;
