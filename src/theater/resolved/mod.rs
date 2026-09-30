//! Chronological queries and materialized playback built only from decoded Film.
//!
//! This layer accumulates raw recorded component state. It does not infer physical
//! actions, interpolate movement, or promote recovery candidates into entities.
pub mod interpretation;
use super::film::{
    ComponentField, ComponentReadStatus, ControlEntry, EntityRecord, EntityViewStop, EventRecord,
    KeyframeRecord, KeyframeStop, ProductionFrame, RecordKind,
};
use super::film::{
    Film, PacketRead, ReplicationStreamPacket, ReplicationStreamPacketBody, SummaryPacket,
};
use interpretation::{Interpretations, SummaryEventRead};
use std::{collections::BTreeMap, sync::Arc};

pub mod events;
pub mod identity;
pub mod playback;
pub mod query;
mod source;
pub mod summary;
use source::{PacketRef, chunk};
pub mod world;
pub use events::*;
use playback::{CHECKPOINT_INTERVAL, Checkpoint};
pub use query::EventFilter;
use query::QueryIndices;
pub use summary::{ResolvedSummary, SummaryDerivation, SummaryKind};
pub use world::*;
/// Resolved recording, chronological index and independent playback cursor.
/// Conversion walks decoded records and gathers explicitly marked interpretations
/// of opaque regions. Source chunk bytes remain borrowed without copying.
/// The Film must outlive this model.
#[derive(Debug, Clone)]
pub struct ResolvedFilm<'film> {
    film: &'film Film,
    interpretations: Interpretations,
    events: Arc<Vec<Event>>,
    summaries: Arc<Vec<ResolvedSummary>>,
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
    pub fn summaries(&self) -> &[ResolvedSummary] {
        &self.summaries
    }
    pub fn record(&self, source: SourceRef) -> Option<Record<'_>> {
        if let RecordRef::Summary(index) = source.record {
            let packet = self.interpretations.summary_packets.iter().find(|packet| {
                packet.source.chunk == source.chunk && packet.source.packet == source.packet
            })?;
            return Some(Record::Summary(packet.events.get(index)?));
        }
        record(self.film, source)
    }
}

impl<'film> ResolvedFilm<'film> {
    pub(super) fn from_film(film: &'film Film) -> Self {
        let interpretations = Interpretations::from_film(film);
        Self::from_interpretations(film, interpretations)
    }

    fn from_interpretations(film: &'film Film, interpretations: Interpretations) -> Self {
        let summaries = summary::resolve_summaries(
            &interpretations.summary_packets,
            interpretations.player_table.as_ref(),
        );
        let mut events = index(
            film,
            interpretations.player_table.as_ref(),
            &interpretations.summary_packets,
        );
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
            summaries: Arc::new(summaries),
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
    FilmIdentity, IdentityField, IdentityRead, IdentityValue, PlayerTable, PlayerTableError,
    PlayerTableReport, PlayerTableShorts, PlayerTableSlot,
};
