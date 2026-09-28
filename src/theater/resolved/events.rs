//! Events support for ResolvedFilm.
use super::*;
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
    /// Recorded summary timestamp, or enclosing packet timestamp for other records.
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

pub(super) fn provenance(film: &Film, source: SourceRef) -> Provenance {
    let packet = &film
        .chunk(source.chunk)
        .expect("indexed source chunk")
        .packets[source.packet];
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

pub(super) fn frame(packet: &NativeFilmPacket, continuation: bool) -> Option<&ProductionFrame> {
    if continuation {
        packet.event_continuation.as_ref()?.frame.as_ref().ok()
    } else if let NativeFilmPacketBody::Frame(frame) = &packet.body {
        Some(frame)
    } else {
        None
    }
}

pub(super) fn record(film: &Film, source: SourceRef) -> Option<Record<'_>> {
    let packet = film.chunk(source.chunk)?.packets.get(source.packet)?;
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

pub(super) fn index(
    film: &Film,
    players: Option<&crate::theater::parser::v41::PlayerTable>,
) -> Vec<Event> {
    let mut events = Vec::new();
    for chunk in film.chunks() {
        let chunk_index = chunk.source_position;
        for (packet_index, packet) in chunk.packets.iter().enumerate() {
            let mut push = |record, kind, entity_id, player_index| {
                events.push(Event {
                    timestamp_us: match record {
                        RecordRef::Summary(i) => match &packet.body {
                            NativeFilmPacketBody::Summary { events, .. } => events[i].time_us,
                            _ => packet.header.timestamp_us,
                        },
                        _ => packet.header.timestamp_us,
                    },
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
                            summary.player.map(usize::from).or_else(|| {
                                let xuid = summary.xuid.parse::<u64>().ok()?;
                                let mut matches = players?.slots.iter().filter(|p| p.xuid == xuid);
                                let player = matches.next()?;
                                matches.next().is_none().then_some(player.film_index)
                            }),
                        );
                    }
                }
                _ => {}
            }
        }
    }
    events
}
