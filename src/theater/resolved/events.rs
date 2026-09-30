//! Events support for ResolvedFilm.
use super::*;
/// A source packet and record/interpretation position. Positions are vector positions,
/// not manifest IDs. Summary positions address guarded reads owned by resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRef {
    pub chunk: usize,
    pub packet: usize,
    pub record: RecordRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordRef {
    Packet,
    Entity { index: usize },
    Keyframe(usize),
    Control { index: usize },
    Summary(usize),
    Event(usize),
}

/// Borrowed canonical payload or explicitly derived summary field read.
/// Summary candidates remain owned by interpretations, not Film.
#[derive(Debug, Clone, Copy)]
pub enum Record<'a> {
    ReplicationPacket(&'a ReplicationStreamPacket),
    SummaryPacket(&'a SummaryPacket),
    Entity(&'a EntityRecord),
    Keyframe(&'a KeyframeRecord),
    Control(&'a ControlEntry),
    Summary(&'a SummaryEventRead),
    Event(&'a EventRecord),
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
    RecordedEvent,
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
            Self::RecordedEvent => EventCategory::Action,
        }
    }
}

/// Index entry for a reference read, including incomplete reads. This is not a
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
    /// State effects are derived by accumulation, separately from the reference payload.
    pub change: Option<StateChange>,
}

/// Reference read status, independent of the derived state change. A complete
/// layout can still include explicitly opaque subfields in its reference payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    RecordedRead,
    PartialRead,
    /// Structural packet entry; inspect its body for opaque/refused regions.
    PacketEnvelope,
    /// Guarded marker association with an observed v41 identity-to-tail offset.
    /// This is not a structurally decoded summary record.
    DerivedSummary,
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
    if matches!(source.record, RecordRef::Summary(_)) {
        return Provenance::DerivedSummary;
    }
    let packet = chunk(film, source.chunk)
        .expect("indexed source chunk")
        .packet(source.packet)
        .expect("indexed source packet");
    let header = packet.header();
    let bounded = |start: i64, end: i64| {
        start >= 0 && end >= start && i128::from(end) <= header.payload_size as i128 * 8
    };
    match record(film, source) {
        Some(Record::Entity(r))
            if r.stop == EntityViewStop::Complete && bounded(r.header.start_bit, r.end_bit) =>
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
            if r.layout_complete && r.end_bit <= header.payload_size as usize * 8 =>
        {
            Provenance::RecordedRead
        }
        Some(Record::ReplicationPacket(_) | Record::SummaryPacket(_)) => Provenance::PacketEnvelope,
        _ => Provenance::PartialRead,
    }
}

pub(super) fn frame(packet: &ReplicationStreamPacket) -> Option<&ProductionFrame> {
    let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &packet.body
    else {
        return None;
    };
    frame.frame.decoded()
}

pub(super) fn replication_packet(
    film: &Film,
    source: SourceRef,
) -> Option<&ReplicationStreamPacket> {
    match chunk(film, source.chunk)?.packet(source.packet)? {
        PacketRef::Replication(packet) => Some(packet),
        PacketRef::Summary(_) => None,
    }
}

pub(super) fn record(film: &Film, source: SourceRef) -> Option<Record<'_>> {
    let packet = chunk(film, source.chunk)?.packet(source.packet)?;
    Some(match source.record {
        RecordRef::Packet => match packet {
            PacketRef::Replication(packet) => Record::ReplicationPacket(packet),
            PacketRef::Summary(packet) => Record::SummaryPacket(packet),
        },
        RecordRef::Summary(_) => return None,
        record => {
            let PacketRef::Replication(packet) = packet else {
                return None;
            };
            match record {
                RecordRef::Entity { index } => Record::Entity(frame(packet)?.records.get(index)?),
                RecordRef::Control { index } => Record::Control(
                    frame(packet)?
                        .controls
                        .as_ref()?
                        .control_entries
                        .get(index)?,
                ),
                RecordRef::Keyframe(index) => {
                    let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(
                        table,
                    )) = &packet.body
                    else {
                        return None;
                    };
                    Record::Keyframe(table.records.get(index)?.record.as_ref()?)
                }
                RecordRef::Event(index) => {
                    let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) =
                        &packet.body
                    else {
                        return None;
                    };
                    Record::Event(frame.events.records.get(index)?)
                }
                RecordRef::Packet | RecordRef::Summary(_) => unreachable!(),
            }
        }
    })
}

pub(super) fn index(
    film: &Film,
    players: Option<&crate::theater::resolved::identity::PlayerTable>,
    summaries: &[interpretation::SummaryPacketInterpretation],
) -> Vec<Event> {
    let mut events = Vec::new();
    for chunk in film.replication_chunks() {
        let chunk_index = chunk.source_position;
        for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
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
            if let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame_packet)) =
                &packet.body
            {
                for i in 0..frame_packet.events.records.len() {
                    push(RecordRef::Event(i), EventKind::RecordedEvent, None, None);
                }
            }
            {
                if let Some(frame) = frame(packet) {
                    for (i, record) in frame.records.iter().enumerate() {
                        let kind = match record.header.kind {
                            RecordKind::New => EventKind::EntityNew,
                            RecordKind::Delete => EventKind::EntityDelete,
                            RecordKind::Delta => EventKind::EntityDelta,
                            RecordKind::End => EventKind::EntityEnd,
                        };
                        push(RecordRef::Entity { index: i }, kind, record.header.id, None);
                    }
                    if let Some(controls) = &frame.controls {
                        for (i, entry) in controls.control_entries.iter().enumerate() {
                            push(
                                RecordRef::Control { index: i },
                                EventKind::Control,
                                None,
                                Some(usize::from(entry.index)),
                            );
                        }
                    }
                }
            }
            if let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
                &packet.body
            {
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
        }
    }
    for chunk in film.summary_chunks() {
        let chunk_index = chunk.source_position;
        for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
            events.push(Event {
                timestamp_us: packet.header.timestamp_us,
                order: 0,
                provenance: Provenance::PacketEnvelope,
                kind: EventKind::Packet,
                source: SourceRef {
                    chunk: chunk_index,
                    packet: packet_index,
                    record: RecordRef::Packet,
                },
                entity_id: None,
                player_index: None,
                change: None,
            });
            let Some(read) = summaries.iter().find(|read| {
                read.source.chunk == chunk_index && read.source.packet == packet_index
            }) else {
                continue;
            };
            for (i, summary) in read.events.iter().enumerate() {
                let player_index = (|| {
                    let xuid = summary.xuid;
                    let mut matches = players?.slots.iter().filter(|p| p.xuid == xuid);
                    let player = matches.next()?;
                    matches.next().is_none().then_some(player.film_index)
                })();
                events.push(Event {
                    timestamp_us: u64::from(summary.timestamp_ms) * 1000,
                    order: 0,
                    provenance: Provenance::PacketEnvelope,
                    kind: EventKind::Summary,
                    source: SourceRef {
                        chunk: chunk_index,
                        packet: packet_index,
                        record: RecordRef::Summary(i),
                    },
                    entity_id: None,
                    player_index,
                    change: None,
                });
            }
        }
    }
    events
}
