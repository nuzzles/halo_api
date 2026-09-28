//! Chronological queries and materialized playback built only from decoded Film.
//!
//! This layer accumulates raw recorded component state. It does not infer physical
//! actions, interpolate movement, or promote recovery candidates into entities.
use super::film::{Film, NativeContinuationStatePolicy, NativeFilmPacket, NativeFilmPacketBody};
use super::parser::{
    ComponentField, EntityRecord, EntityViewStop, KeyframeRecord, KeyframeStop, NativeControlEntry,
    NativeEventRecord, PlayerTableSlot, ProductionFrame, RecordKind, SummaryEvent,
};
use std::{collections::BTreeMap, sync::Arc};

/// A stable path into the borrowed Film. Positions are vector positions, not manifest IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRef {
    pub chunk: usize,
    pub packet: usize,
    pub record: RecordRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordRef {
    Packet,
    Entity { continuation: bool, index: usize },
    Keyframe(usize),
    Control { continuation: bool, index: usize },
    Summary(usize),
    Event(usize),
}

/// Borrowed native payload. Independent diagnostic projections remain on Packet.
#[derive(Debug, Clone, Copy)]
pub enum Record<'a> {
    Packet(&'a NativeFilmPacket),
    Entity(&'a EntityRecord),
    Keyframe(&'a KeyframeRecord),
    Control(&'a NativeControlEntry),
    Summary(&'a SummaryEvent),
    Event(&'a NativeEventRecord),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventKind {
    Packet,
    EntityNew,
    EntityDelete,
    EntityDelta,
    EntityEnd,
    Keyframe,
    Control,
    Summary,
    NativeEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventCategory {
    Structure,
    Lifecycle,
    State,
    Input,
    Summary,
    Action,
}
impl EventKind {
    pub fn category(self) -> EventCategory {
        match self {
            Self::Packet | Self::EntityEnd => EventCategory::Structure,
            Self::EntityNew | Self::EntityDelete => EventCategory::Lifecycle,
            Self::EntityDelta | Self::Keyframe => EventCategory::State,
            Self::Control => EventCategory::Input,
            Self::Summary => EventCategory::Summary,
            Self::NativeEvent => EventCategory::Action,
        }
    }
}

/// Index entry for a native read, including incomplete reads. This is not a
/// claim that every indexed read describes a successfully completed action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Exact enclosing packet timestamp; a summary's own time remains in its payload.
    pub timestamp_us: u64,
    pub order: usize,
    pub kind: EventKind,
    pub provenance: Provenance,
    pub source: SourceRef,
    /// Raw record identity. Keyframe namespace bits are not runtime generations.
    pub entity_id: Option<u32>,
    /// Recorded control/player index, where explicitly present. No guessed attribution.
    pub player_index: Option<usize>,
    /// State effects are derived by accumulation, separately from the native payload.
    pub change: Option<StateChange>,
}

/// Native read status, independent of the derived state change. A complete
/// layout can still include explicitly opaque subfields in its native payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    RecordedRead,
    PartialRead,
    /// Structural packet entry; inspect its body for opaque/refused regions.
    PacketEnvelope,
}

/// A state derivation from a sequential record. Previous/new values share storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateChange {
    pub key: EntityKey,
    pub previous: Option<Arc<EntityState>>,
    pub new: Option<Arc<EntityState>>,
    pub derivation: &'static str,
}

/// Runtime entity view or an independent keyframe namespace. The first native
/// keyframe namespace seeds the entity view, following the decoder's binding policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityDomain {
    Runtime,
    Keyframe(u8),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityKey {
    pub domain: EntityDomain,
    pub slot: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentState {
    pub name: String,
    /// Ordered raw fields, including repeated names and quantized values.
    pub fields: Vec<ComponentField>,
    /// False means these are only the known prefix of the latest update.
    pub complete: bool,
    pub source: SourceRef,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityState {
    /// Runtime ID including generation, or None for keyframe-only slot evidence.
    pub id: Option<u32>,
    pub archetype: u32,
    /// NEW timestamp, never a fabricated spawn time for a keyframe observation.
    pub created_at_us: Option<u64>,
    /// Whether the native baseline reader completed; opaque/skipped component
    /// bodies remain individually marked and do not become decoded values.
    pub baseline_read_complete: bool,
    pub components: BTreeMap<usize, Arc<ComponentState>>,
    /// Raw fields of the last NEW/keyframe baseline, including header fields.
    pub initial_fields: Arc<Vec<ComponentField>>,
    pub last_source: SourceRef,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub entities: BTreeMap<EntityKey, Arc<EntityState>>,
}

/// Inclusive timestamp bounds. Filtering never alters playback state.
#[derive(Debug, Clone, Copy, Default)]
pub struct EventFilter {
    pub start_us: Option<u64>,
    pub end_us: Option<u64>,
    pub kind: Option<EventKind>,
    pub category: Option<EventCategory>,
    pub entity_id: Option<u32>,
    pub player_index: Option<usize>,
}

#[derive(Debug, Clone)]
struct Checkpoint {
    next_event: usize,
    world: WorldSnapshot,
}

#[derive(Debug, Default)]
struct QueryIndices {
    kinds: BTreeMap<EventKind, Vec<usize>>,
    categories: BTreeMap<EventCategory, Vec<usize>>,
    entities: BTreeMap<u32, Vec<usize>>,
    players: BTreeMap<usize, Vec<usize>>,
}
impl QueryIndices {
    fn new(events: &[Event]) -> Self {
        let mut out = Self::default();
        for (i, event) in events.iter().enumerate() {
            out.kinds.entry(event.kind).or_default().push(i);
            out.categories
                .entry(event.kind.category())
                .or_default()
                .push(i);
            if let Some(id) = event.entity_id {
                out.entities.entry(id).or_default().push(i);
            }
            if let Some(id) = event.player_index {
                out.players.entry(id).or_default().push(i);
            }
        }
        out
    }
}

/// Resolved recording, chronological index and independent playback cursor.
/// Conversion walks decoded records without reparsing or copying source bytes.
/// The Film must outlive this model.
#[derive(Debug, Clone)]
pub struct ResolvedFilm<'film> {
    film: &'film Film,
    events: Arc<Vec<Event>>,
    query_indices: Arc<QueryIndices>,
    checkpoints: Arc<Vec<Checkpoint>>,
    /// Recorded film player index -> position in `film.player_table.slots`.
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
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    /// O(1) borrowed access; iterating entities costs O(world size).
    pub fn current(&self) -> &WorldSnapshot {
        &self.world
    }
    /// None before any seek/advance, including for recordings starting at zero.
    pub fn timestamp_us(&self) -> Option<u64> {
        self.timestamp_us
    }
    /// Bootstrap identity by recorded film index. Dynamic roster evidence stays
    /// accessible through packet records; this does not guess entity ownership.
    pub fn player(&self, film_index: usize) -> Option<&PlayerTableSlot> {
        self.film
            .player_table
            .as_ref()?
            .slots
            .get(*self.player_slot_by_film_index.get(&film_index)?)
    }
    pub fn record(&self, source: SourceRef) -> Option<Record<'_>> {
        record(self.film, source)
    }
    /// Binary-search time bounds in the smallest applicable entity/player/kind/
    /// category index, then test the remaining predicates on those candidates.
    /// Without categorical filters, iterate the timestamp slice directly.
    pub fn query(&self, filter: EventFilter) -> impl Iterator<Item = &Event> {
        let start = filter
            .start_us
            .map_or(0, |t| self.events.partition_point(|e| e.timestamp_us < t));
        let end = filter.end_us.map_or(self.events.len(), |t| {
            self.events.partition_point(|e| e.timestamp_us <= t)
        });
        let start = start.min(end);
        let indices = &self.query_indices;
        let selected = [
            filter
                .kind
                .map(|v| indices.kinds.get(&v).map_or(&[][..], Vec::as_slice)),
            filter
                .category
                .map(|v| indices.categories.get(&v).map_or(&[][..], Vec::as_slice)),
            filter
                .entity_id
                .map(|v| indices.entities.get(&v).map_or(&[][..], Vec::as_slice)),
            filter
                .player_index
                .map(|v| indices.players.get(&v).map_or(&[][..], Vec::as_slice)),
        ]
        .into_iter()
        .flatten()
        .map(|ids| &ids[ids.partition_point(|&i| i < start)..ids.partition_point(|&i| i < end)])
        .min_by_key(|ids| ids.len());
        let mut selected = selected.map(|ids| ids.iter().copied());
        let mut range = start..end;
        std::iter::from_fn(move || {
            let index = match &mut selected {
                Some(ids) => ids.next(),
                None => range.next(),
            }?;
            Some(&self.events[index])
        })
        .filter(move |e| {
            filter.kind.is_none_or(|v| e.kind == v)
                && filter.category.is_none_or(|v| e.kind.category() == v)
                && filter.entity_id.is_none_or(|v| e.entity_id == Some(v))
                && filter
                    .player_index
                    .is_none_or(|v| e.player_index == Some(v))
        })
    }
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
    pub fn rewind(&mut self) {
        self.world = WorldSnapshot::default();
        self.next_event = 0;
        self.timestamp_us = None;
    }
    fn apply_until(&mut self, end: usize) {
        for event in &self.events[self.next_event..end] {
            apply(&mut self.world, event.change.as_ref());
        }
        self.next_event = end;
    }
}

const CHECKPOINT_INTERVAL: usize = 1024;

impl<'film> ResolvedFilm<'film> {
    pub(super) fn from_film(film: &'film Film) -> Self {
        let mut events = index(film);
        // Stable sort preserves source chunk/packet/record order for clock ties.
        events.sort_by_key(|e| e.timestamp_us);
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
        let player_slot_by_film_index = film
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

fn provenance(film: &Film, source: SourceRef) -> Provenance {
    let packet = &film.chunks[source.chunk].packets[source.packet];
    let bounded = |start: i64, end: i64| {
        start >= 0 && end >= start && i128::from(end) <= packet.header.payload_size as i128 * 8
    };
    match record(film, source) {
        Some(Record::Entity(r))
            if r.stop == EntityViewStop::Complete
                && r.padded_bits == 0
                && bounded(r.header.start_bit, r.end_bit) =>
        {
            Provenance::RecordedRead
        }
        Some(Record::Keyframe(r))
            if r.stop == KeyframeStop::Complete && bounded(r.start_bit, r.end_bit) =>
        {
            Provenance::RecordedRead
        }
        Some(Record::Control(r)) if bounded(r.start_bit, r.end_bit) => Provenance::RecordedRead,
        Some(Record::Summary(_)) => Provenance::RecordedRead,
        Some(Record::Event(r))
            if r.layout_complete && r.end_bit <= packet.header.payload_size.saturating_mul(8) =>
        {
            Provenance::RecordedRead
        }
        Some(Record::Packet(_)) => Provenance::PacketEnvelope,
        _ => Provenance::PartialRead,
    }
}

fn apply(world: &mut WorldSnapshot, change: Option<&StateChange>) {
    if let Some(change) = change {
        if let Some(new) = &change.new {
            world.entities.insert(change.key, new.clone());
        } else {
            world.entities.remove(&change.key);
        }
    }
}

fn frame(packet: &NativeFilmPacket, continuation: bool) -> Option<&ProductionFrame> {
    if continuation {
        packet.event_continuation.as_ref()?.frame.as_ref().ok()
    } else if let NativeFilmPacketBody::Frame(frame) = &packet.body {
        Some(frame)
    } else {
        None
    }
}

fn record(film: &Film, source: SourceRef) -> Option<Record<'_>> {
    let packet = film.chunks.get(source.chunk)?.packets.get(source.packet)?;
    Some(match source.record {
        RecordRef::Packet => Record::Packet(packet),
        RecordRef::Entity {
            continuation,
            index,
        } => Record::Entity(frame(packet, continuation)?.records.get(index)?),
        RecordRef::Control {
            continuation,
            index,
        } => Record::Control(
            frame(packet, continuation)?
                .controls
                .as_ref()?
                .control_entries
                .get(index)?,
        ),
        RecordRef::Keyframe(index) => {
            let NativeFilmPacketBody::Keyframes(table) = &packet.body else {
                return None;
            };
            Record::Keyframe(table.records.get(index)?.record.as_ref()?)
        }
        RecordRef::Summary(index) => {
            let NativeFilmPacketBody::Summary { events, .. } = &packet.body else {
                return None;
            };
            Record::Summary(events.get(index)?)
        }
        RecordRef::Event(index) => Record::Event(packet.event_list.as_ref()?.records.get(index)?),
    })
}

fn index(film: &Film) -> Vec<Event> {
    let mut events = Vec::new();
    for (chunk_index, chunk) in film.chunks.iter().enumerate() {
        for (packet_index, packet) in chunk.packets.iter().enumerate() {
            let mut push = |record, kind, entity_id, player_index| {
                events.push(Event {
                    timestamp_us: packet.header.timestamp_us,
                    order: 0,
                    provenance: Provenance::PacketEnvelope,
                    kind,
                    source: SourceRef {
                        chunk: chunk_index,
                        packet: packet_index,
                        record,
                    },
                    entity_id,
                    player_index,
                    change: None,
                })
            };
            push(RecordRef::Packet, EventKind::Packet, None, None);
            // Event lists precede the continuation's entity and control views.
            if let Some(list) = &packet.event_list {
                for i in 0..list.records.len() {
                    push(RecordRef::Event(i), EventKind::NativeEvent, None, None);
                }
            }
            for continuation in [false, true] {
                if let Some(frame) = frame(packet, continuation) {
                    for (i, record) in frame.records.iter().enumerate() {
                        let kind = match record.header.kind {
                            RecordKind::New => EventKind::EntityNew,
                            RecordKind::Delete => EventKind::EntityDelete,
                            RecordKind::Delta => EventKind::EntityDelta,
                            RecordKind::End => EventKind::EntityEnd,
                        };
                        push(
                            RecordRef::Entity {
                                continuation,
                                index: i,
                            },
                            kind,
                            record.header.id,
                            None,
                        );
                    }
                    if let Some(controls) = &frame.controls {
                        for (i, entry) in controls.control_entries.iter().enumerate() {
                            push(
                                RecordRef::Control {
                                    continuation,
                                    index: i,
                                },
                                EventKind::Control,
                                None,
                                Some(usize::from(entry.index)),
                            );
                        }
                    }
                }
            }
            match &packet.body {
                NativeFilmPacketBody::Keyframes(table) => {
                    for (i, attempt) in table.records.iter().enumerate() {
                        if attempt.record.is_some() {
                            push(
                                RecordRef::Keyframe(i),
                                EventKind::Keyframe,
                                Some(attempt.id),
                                None,
                            );
                        }
                    }
                }
                NativeFilmPacketBody::Summary { events, .. } => {
                    for (i, summary) in events.iter().enumerate() {
                        push(
                            RecordRef::Summary(i),
                            EventKind::Summary,
                            None,
                            summary.player.map(usize::from),
                        );
                    }
                }
                _ => {}
            }
        }
    }
    events
}

fn resolve_change(
    film: &Film,
    world: &WorldSnapshot,
    event: &Event,
    first_namespace: &mut Option<u8>,
) -> Option<StateChange> {
    match record(film, event.source)? {
        Record::Entity(record) => {
            if let RecordRef::Entity {
                continuation: true, ..
            } = event.source.record
            {
                let continuation = film.chunks[event.source.chunk].packets[event.source.packet]
                    .event_continuation
                    .as_ref()?;
                if continuation.state_policy != NativeContinuationStatePolicy::Applied {
                    return None;
                }
            }
            let packet = &film.chunks[event.source.chunk].packets[event.source.packet];
            if record.end_bit > packet.header.payload_size as i64 * 8 {
                return None;
            }
            if let RecordRef::Entity { continuation, .. } = event.source.record
                && frame(packet, continuation)?
                    .header_diagnostics
                    .new_binding_refusals
                    .iter()
                    .any(|refusal| refusal.record_bit == record.header.start_bit)
            {
                return None;
            }
            entity_change(world, event, record)
        }
        Record::Keyframe(record) => {
            let namespace = (record.id >> 30) as u8;
            let first = *first_namespace.get_or_insert(namespace);
            let key = EntityKey {
                domain: if first == namespace {
                    EntityDomain::Runtime
                } else {
                    EntityDomain::Keyframe(namespace)
                },
                slot: record.id & 0x3fff_ffff,
            };
            // Native padded reads remain visible but cannot become recorded state.
            let packet = &film.chunks[event.source.chunk].packets[event.source.packet];
            if record.start_bit < 0 || record.end_bit > packet.header.payload_size as i64 * 8 {
                return None;
            }
            let previous = world.entities.get(&key).cloned();
            let mut state = EntityState {
                id: None,
                archetype: record.archetype,
                created_at_us: None,
                baseline_read_complete: record.stop == KeyframeStop::Complete,
                components: BTreeMap::new(),
                initial_fields: Arc::new(record.fields.clone()),
                last_source: event.source,
            };
            for component in &record.attempts {
                let Some([start, end]) = component.field_range else {
                    continue;
                };
                let Some(fields) = record.fields.get(start..end) else {
                    continue;
                };
                state.components.insert(
                    component.index,
                    Arc::new(ComponentState {
                        name: component.name.clone(),
                        fields: fields.to_vec(),
                        complete: component.ported == Some(true),
                        source: event.source,
                    }),
                );
            }
            // Even partial keyframes explicitly replace the old baseline. Its
            // unread components stay absent instead of inheriting stale values.
            if record.stop == KeyframeStop::InvalidArchetype {
                return None;
            }
            Some(StateChange {
                key,
                previous,
                new: Some(Arc::new(state)),
                derivation: "sequential-keyframe-baseline-v1",
            })
        }
        _ => None,
    }
}

fn entity_change(
    world: &WorldSnapshot,
    event: &Event,
    record: &EntityRecord,
) -> Option<StateChange> {
    let id = record.header.id?;
    if record.padded_bits != 0
        || record.header.start_bit < 0
        || !record.diagnostics.new_binding_refusals.is_empty()
    {
        return None;
    }
    let key = EntityKey {
        domain: EntityDomain::Runtime,
        slot: id & 0x3fff_ffff,
    };
    let previous = world.entities.get(&key).cloned();
    if record.header.kind == RecordKind::Delete {
        let existing = previous.as_ref()?;
        if existing.id.is_some_and(|old| old != id) || record.stop != EntityViewStop::Complete {
            return None;
        }
        return Some(StateChange {
            key,
            previous,
            new: None,
            derivation: "recorded-delete-v1",
        });
    }
    // Match native admission: an incomplete NEW does not establish a binding.
    if record.header.kind == RecordKind::New && record.stop != EntityViewStop::Complete {
        return None;
    }
    let mut state = match record.header.kind {
        RecordKind::New => EntityState {
            id: Some(id),
            archetype: record.archetype?,
            created_at_us: Some(event.timestamp_us),
            baseline_read_complete: !record.default_state_fallback,
            components: BTreeMap::new(),
            initial_fields: Arc::new(record.fields.clone()),
            last_source: event.source,
        },
        RecordKind::Delta => {
            let previous = previous.as_ref()?;
            if previous.id.is_some_and(|old| old != id)
                || record.archetype.is_some_and(|a| a != previous.archetype)
            {
                return None;
            }
            let mut state = (**previous).clone();
            state.id = Some(id);
            state
        }
        _ => return None,
    };
    state.last_source = event.source;
    for component in &record.attempts {
        let Some(fields) = record
            .fields
            .get(component.field_start..component.field_end)
        else {
            continue;
        };
        state.components.insert(
            component.span.index,
            Arc::new(ComponentState {
                name: component.span.name.clone(),
                fields: fields.to_vec(),
                complete: component.status == Some(true),
                source: event.source,
            }),
        );
    }
    Some(StateChange {
        key,
        previous,
        new: Some(Arc::new(state)),
        derivation: "sequential-entity-components-v1",
    })
}

#[cfg(test)]
mod tests;
