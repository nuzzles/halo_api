//! Chronological queries and materialized playback built only from decoded Film.
//!
//! This layer accumulates raw recorded component state. It does not infer physical
//! actions, interpolate movement, or promote recovery candidates into entities.
pub mod interpretation;
use super::film::{
    ComponentField, EntityRecord, EntityViewStop, KeyframeRecord, KeyframeStop, NativeControlEntry,
    NativeEventRecord, PlayerTableSlot, ProductionFrame, RecordKind, SummaryEvent,
};
use super::film::{Film, FilmPacket, FilmPacketBody, NativeContinuationStatePolicy};
use interpretation::Interpretations;
use std::{collections::BTreeMap, sync::Arc};

pub mod events;
pub mod identity;
pub mod playback;
pub mod query;
pub mod world;
pub use events::*;
use playback::{CHECKPOINT_INTERVAL, Checkpoint};
pub use query::EventFilter;
use query::QueryIndices;
pub use world::*;
/// Resolved recording, chronological index and independent playback cursor.
/// Conversion walks decoded records without reparsing or copying source bytes.
/// The Film must outlive this model.
#[derive(Debug, Clone)]
pub struct ResolvedFilm<'film> {
    film: &'film Film,
    interpretations: Interpretations,
    events: Arc<Vec<Event>>,
    query_indices: Arc<QueryIndices>,
    checkpoints: Arc<Vec<Checkpoint>>,
    /// Recorded film player index -> position in `interpretations.player_table.slots`.
    /// Recorded indices may differ from vector positions. This indexes the
    /// bootstrap roster without copying it; it does not resolve entity ownership.
    player_slot_by_film_index: BTreeMap<usize, usize>,
    world: WorldSnapshot,
    next_event: usize,
    timestamp_us: Option<u64>,
}

impl<'film> ResolvedFilm<'film> {
    pub fn film(&self) -> &Film {
        self.film
    }
    pub fn interpretations(&self) -> &Interpretations {
        &self.interpretations
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    pub fn record(&self, source: SourceRef) -> Option<Record<'_>> {
        record(self.film, source)
    }
}

impl<'film> ResolvedFilm<'film> {
    pub(super) fn from_film(film: &'film Film) -> Self {
        let interpretations = Interpretations::from_film(film);
        let mut events = index(film, interpretations.player_table.as_ref());
        // Stable sort preserves source chunk/packet/record order for clock ties.
        events.sort_by_key(|e| (e.timestamp_us, e.source.chunk, e.source.packet));
        let mut timestamp = None;
        let mut order = 0;
        let mut world = WorldSnapshot::default();
        let mut first_namespace = None;
        let mut checkpoints = vec![Checkpoint {
            next_event: 0,
            world: world.clone(),
        }];
        for (i, event) in events.iter_mut().enumerate() {
            if timestamp != Some(event.timestamp_us) {
                order = 0;
                timestamp = Some(event.timestamp_us);
            }
            event.provenance = provenance(film, event.source);
            event.order = order;
            order += 1;
            event.change = resolve_change(film, &world, event, &mut first_namespace);
            apply(&mut world, event.change.as_ref());
            if (i + 1) % CHECKPOINT_INTERVAL == 0 {
                checkpoints.push(Checkpoint {
                    next_event: i + 1,
                    world: world.clone(),
                });
            }
        }
        let player_slot_by_film_index = interpretations
            .player_table
            .as_ref()
            .map(|p| {
                p.slots
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (p.film_index, i))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            film,
            interpretations,
            query_indices: Arc::new(QueryIndices::new(&events)),
            events: Arc::new(events),
            checkpoints: Arc::new(checkpoints),
            player_slot_by_film_index,
            world: WorldSnapshot::default(),
            next_event: 0,
            timestamp_us: None,
        }
    }
}

#[cfg(test)]
mod tests;

pub use identity::{
    FilmIdentity, NativeIdentityField, NativeIdentityRead, NativeIdentityValue, PlayerTable,
    PlayerTableError, PlayerTableReport,
};
