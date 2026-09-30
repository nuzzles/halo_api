//! Chronological queries and materialized playback built only from decoded Film.
//!
//! This layer accumulates raw recorded component state. It does not infer physical
//! actions, interpolate movement, or promote recovery candidates into entities.
pub mod interpretation;
use crate::theater::film::{
    ComponentField, ComponentReadStatus, ControlEntry, EntityRecord, EntityViewStop, EventRecord,
    KeyframeRecord, KeyframeStop, ProductionFrame, RecordKind,
};
use crate::theater::film::{
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
use query::{QueryIndices, SummaryIndices};
use summary::SummaryEvent;
#[cfg(test)]
use summary::{SummaryDerivation, SummaryKind};
pub use world::*;
/// Resolved recording, chronological index and independent playback cursor.
/// Conversion walks decoded records and gathers explicitly marked interpretations
/// of opaque regions. The Film is retained in shared ownership without copying source buffers.
#[derive(Debug)]
pub(super) struct ResolvedFilm {
    film: Arc<Film>,
    interpretations: Arc<Interpretations>,
    events: Arc<Vec<Event>>,
    summaries: Arc<Vec<SummaryEvent>>,
    summary_indices: Arc<SummaryIndices>,
    summary_reports: Arc<Vec<summary::SummaryReadReport>>,
    summary_packet_by_source: Arc<BTreeMap<(usize, usize), usize>>,
    query_indices: Arc<QueryIndices>,
    checkpoints: Arc<Vec<Checkpoint>>,
    /// Recorded film player index -> position in `interpretations.player_table.slots`.
    /// Recorded indices may differ from vector positions. This indexes the
    /// bootstrap roster without copying it; it does not resolve entity ownership.
    player_slot_by_film_index: Arc<BTreeMap<usize, usize>>,
    world: WorldSnapshot,
    next_event: usize,
    timestamp_us: Option<u64>,
}

impl ResolvedFilm {
    pub fn film(&self) -> &Film {
        &self.film
    }
    pub fn interpretations(&self) -> &Interpretations {
        &self.interpretations
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    pub fn summary_events(&self) -> &[SummaryEvent] {
        &self.summaries
    }
    pub fn summary_reports(&self) -> &[summary::SummaryReadReport] {
        &self.summary_reports
    }
    pub fn record(&self, source: SourceRef) -> Option<Record<'_>> {
        if let RecordRef::Summary(index) = source.record {
            let packet_index = *self
                .summary_packet_by_source
                .get(&(source.chunk, source.packet))?;
            let packet = self.interpretations.summary_packets.get(packet_index)?;
            return Some(Record::Summary(packet.events.get(index)?));
        }
        record(&self.film, source)
    }
}

impl ResolvedFilm {
    pub(super) fn from_film(film: Arc<Film>) -> Self {
        let interpretations = Interpretations::from_film(&film);
        Self::from_interpretations(film, interpretations)
    }

    fn from_interpretations(film: Arc<Film>, interpretations: Interpretations) -> Self {
        let mut summaries = summary::resolve_summaries(
            &interpretations.summary_packets,
            interpretations.player_table.as_ref(),
        );
        let summary_by_source: BTreeMap<_, _> = summaries
            .iter()
            .enumerate()
            .map(|(index, event)| {
                let RecordRef::Summary(record) = event.source.record else {
                    unreachable!("summary source")
                };
                ((event.source.chunk, event.source.packet, record), index)
            })
            .collect();
        let summary_packet_by_source: BTreeMap<_, _> = interpretations
            .summary_packets
            .iter()
            .enumerate()
            .map(|(index, packet)| ((packet.source.chunk, packet.source.packet), index))
            .collect();
        let summary_reports = film
            .summary_chunks()
            .flat_map(|chunk| {
                chunk
                    .body
                    .packets
                    .iter()
                    .enumerate()
                    .map(|(packet_index, packet)| {
                        let declared_events = match &packet.body {
                            PacketRead::Decoded(
                                crate::theater::film::SummaryPacketBody::Events {
                                    declared_events,
                                    ..
                                },
                            ) => Some(*declared_events),
                            PacketRead::Opaque { .. } => None,
                        };
                        let candidate_events = summary_packet_by_source
                            .get(&(chunk.source_position, packet_index))
                            .map_or(0, |index| {
                                interpretations.summary_packets[*index].events.len()
                            });
                        summary::SummaryReadReport {
                            source: SourceRef {
                                chunk: chunk.source_position,
                                packet: packet_index,
                                record: RecordRef::Packet,
                            },
                            declared_events,
                            candidate_events,
                        }
                    })
            })
            .collect();
        let mut events = index(
            &film,
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
            event.provenance = provenance(&film, event.source);
            event.order = order;
            if let RecordRef::Summary(record) = event.source.record {
                let index = summary_by_source[&(event.source.chunk, event.source.packet, record)];
                summaries[index].order = order;
                event.player_index = summaries[index].actor.roster_link.film_index();
            }
            order += 1;
            event.change = resolve_change(&film, &world, event, &mut first_namespace);
            apply(&mut world, event.change.as_ref());
            if (i + 1) % CHECKPOINT_INTERVAL == 0 {
                checkpoints.push(Checkpoint {
                    next_event: i + 1,
                    world: world.clone(),
                });
            }
        }
        let mut slots_by_index: BTreeMap<usize, Option<usize>> = BTreeMap::new();
        if let Some(table) = &interpretations.player_table {
            for (position, slot) in table.slots.iter().enumerate() {
                slots_by_index
                    .entry(slot.film_index)
                    .and_modify(|position| *position = None)
                    .or_insert(Some(position));
            }
        }
        let player_slot_by_film_index = slots_by_index
            .into_iter()
            .filter_map(|(index, position)| position.map(|position| (index, position)))
            .collect();
        Self {
            film,
            interpretations: Arc::new(interpretations),
            query_indices: Arc::new(QueryIndices::new(&events)),
            events: Arc::new(events),
            summary_indices: Arc::new(SummaryIndices::new(&summaries)),
            summaries: Arc::new(summaries),
            summary_reports: Arc::new(summary_reports),
            summary_packet_by_source: Arc::new(summary_packet_by_source),
            checkpoints: Arc::new(checkpoints),
            player_slot_by_film_index: Arc::new(player_slot_by_film_index),
            world: WorldSnapshot::default(),
            next_event: 0,
            timestamp_us: None,
        }
    }
}

#[cfg(test)]
mod tests;
