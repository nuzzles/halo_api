//! Sequential replication decoding: stream grammar, component grammar, and bindings.
use super::registry::ParsedRegistry;
use crate::theater::film::InflatedFilmChunk;
use crate::theater::film::replication::{
    ReplicationChange, ReplicationComponentChange, ReplicationControl, ReplicationEventBody,
    ReplicationEventKind, ReplicationEventWord, ReplicationField, ReplicationKill,
    ReplicationObservation, ReplicationStream, ReplicationValue,
};
use crate::theater::film::{ArchetypeId, ComponentSlot, ControlPlayerIndex, EntityId, KeyframeId};
pub(crate) mod source;

use crate::theater::film::ParseError;
use crate::theater::parser::packet::{PacketRead, PacketStream};
use crate::theater::parser::v41::chunks::replication::source::components::field::ComponentField;
use crate::theater::parser::v41::chunks::replication::source::{
    ComponentReadStatus, ControlPayload, DefaultState, DefaultStateStatus, EntityViewStop,
    EventFieldStage, EventFieldValue, KeyframeStop, RecordKind, ReplicationStreamPacket,
    ReplicationStreamPacketBody,
};
use crate::theater::parser::v41::context::V41DecodeConfig;
use crate::theater::parser::{
    DeltaRecordSite, FilmParseContext, ReplayTickSource, ReplayTickStatus, VehicleSnapshotSite,
};
use replication_stream::DecodeContext;

pub(crate) mod bindings;
pub(crate) mod components;
pub(crate) mod replication_stream;
pub(crate) mod state;

/// Read replication packet bodies with state retained across ordered chunks.
pub struct V41ReplicationStreamChunkReader;

/// Source-backed decoder result retained until admission and typed projection.
#[derive(Debug)]
pub(crate) struct ParsedReplicationStream(pub(crate) Vec<ParsedReplicationStreamChunk>);

impl std::ops::Deref for ParsedReplicationStream {
    type Target = [ParsedReplicationStreamChunk];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ParsedReplicationStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a ParsedReplicationStream {
    type Item = &'a ParsedReplicationStreamChunk;
    type IntoIter = std::slice::Iter<'a, ParsedReplicationStreamChunk>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut ParsedReplicationStream {
    type Item = &'a mut ParsedReplicationStreamChunk;
    type IntoIter = std::slice::IterMut<'a, ParsedReplicationStreamChunk>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[derive(Debug)]
pub(crate) struct ParsedReplicationStreamChunk {
    pub(crate) index: usize,
    pub(crate) start_ms: i64,
    pub(crate) data: Vec<u8>,
    pub(crate) body: PacketStream<ReplicationStreamPacketBody>,
}

impl ParsedReplicationStreamChunk {
    pub(crate) fn payload(&self, packet: &ReplicationStreamPacket) -> Option<&[u8]> {
        packet.payload(&self.data)
    }
}

impl ParsedReplicationStream {
    pub(crate) fn parse(
        registry: &ParsedRegistry,
        inputs: impl IntoIterator<Item = InflatedFilmChunk>,
        parse_context: &FilmParseContext,
        max_packets: usize,
        max_control_records: usize,
    ) -> Result<Self, ParseError> {
        for &archetype in parse_context.delta_record_archetypes.values() {
            if archetype >= 50
                || registry
                    .body
                    .registry
                    .archetype(archetype as usize)
                    .is_none()
            {
                return Err(ParseError::InvalidDeltaContextArchetype { archetype });
            }
        }
        let mut config = V41DecodeConfig::from_registry(&registry.body);
        config.control_record_limit = max_control_records;
        config.event_kill_tail = parse_context.kill_event_tail;
        let recorded_script_prefix =
            crate::theater::parser::v41::context::events::recorded_script_prefix(&registry.body);
        if recorded_script_prefix
            .zip(parse_context.script_event_prefix)
            .is_some_and(|(recorded, supplied)| recorded != supplied)
        {
            return Err(ParseError::ContradictoryScriptPrefixContext);
        }
        config.event_script_prefix = parse_context
            .script_event_prefix
            .filter(|_| recorded_script_prefix.is_none());
        let mut state = state::ReplicationDecodeState::default();
        let mut context = DecodeContext {
            config: &config,
            registry: &registry.body.registry,
            state: &mut state,
            event_gate15: recorded_script_prefix.or(parse_context.script_event_prefix),
        };
        let mut chunks = Vec::new();
        for input in inputs {
            context.state.current_chunk = input.index;
            chunks.push(V41ReplicationStreamChunkReader::read(
                input,
                &mut context,
                parse_context,
                max_packets,
            )?);
        }
        if chunks.is_empty() {
            return Err(ParseError::MissingReplication);
        }
        Ok(Self(chunks))
    }

    /// Source-backed tick admission, evaluated before dropping packet bytes.
    pub(crate) fn replay_tick_status(&self) -> ReplayTickStatus {
        let mut incomplete = false;
        for (chunk_index, chunk) in self.0.iter().enumerate() {
            incomplete |= !chunk.body.opaque.is_empty();
            for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
                let Some(payload) = chunk.payload(packet) else {
                    incomplete = true;
                    continue;
                };
                if packet.header.packet_type == 0
                    && let Some((payload_bit, _)) =
                        crate::theater::parser::ticks::tick_counter(payload)
                {
                    return ReplayTickStatus::Found(ReplayTickSource {
                        chunk: chunk_index,
                        packet: packet_index,
                        payload_bit,
                    });
                }
            }
        }
        if incomplete {
            ReplayTickStatus::Undetermined
        } else {
            ReplayTickStatus::Absent
        }
    }

    /// Consume source packets into owned, complete playback observations.
    pub(crate) fn into_typed(self) -> ReplicationStream {
        let mut observations = Vec::new();
        for chunk in self.0 {
            for packet in chunk.body.packets {
                let timestamp_us = packet.header.timestamp_us;
                let mut emit = |change| {
                    observations.push(ReplicationObservation {
                        timestamp_us,
                        change,
                    })
                };
                let PacketRead::Decoded(body) = packet.body else {
                    continue;
                };
                match body {
                    ReplicationStreamPacketBody::FramePacketBody(frame) => {
                        for record in frame.events.records {
                            if !record.layout_complete {
                                continue;
                            }
                            if let Some(code) = record.code {
                                let mut references = Vec::new();
                                let mut body = Vec::new();
                                for field in record.fields {
                                    let EventFieldValue::Scalar(value) = field.value else {
                                        continue;
                                    };
                                    let word = ReplicationEventWord {
                                        value,
                                        bit_len: field.width,
                                    };
                                    match field.stage {
                                        EventFieldStage::Header => {}
                                        EventFieldStage::References => references.push(word),
                                        EventFieldStage::Body => body.push(word),
                                        EventFieldStage::CorruptionCheck => {}
                                    }
                                }
                                let kind = ReplicationEventKind::from(code);
                                let body = match kind {
                                    ReplicationEventKind::Kill => project_kill(&body)
                                        .map(ReplicationEventBody::Kill)
                                        .unwrap_or(ReplicationEventBody::Fields(body)),
                                    _ => ReplicationEventBody::Fields(body),
                                };
                                emit(ReplicationChange::Event {
                                    code: kind,
                                    references,
                                    body,
                                });
                            }
                        }
                        if let crate::theater::parser::v41::chunks::replication::source::FrameRead::Decoded(frame) = frame.frame {
                            for record in frame.records {
                                if record.stop != EntityViewStop::Complete {
                                    continue;
                                }
                                let Some(id) = record.header.id else {
                                    continue;
                                };
                                let complete_components =
                                    record.components.iter().all(|component| {
                                        component.status == ComponentReadStatus::Complete
                                    });
                                if !complete_components {
                                    continue;
                                }
                                let components = record
                                    .components
                                    .into_iter()
                                    .map(|component| ReplicationComponentChange {
                                        index: ComponentSlot(component.index),
                                        name: component.name,
                                        fields: project_fields(component.fields),
                                        weapon_id: component.weapon_id,
                                    })
                                    .collect();
                                match record.header.kind {
                                    RecordKind::New => {
                                        if let Some(archetype) = record.archetype {
                                            emit(ReplicationChange::EntityCreated {
                                                id: EntityId(id),
                                                archetype: ArchetypeId(archetype),
                                                default_fields: default_fields(
                                                    record.default_state,
                                                ),
                                                fields: record_fields(record.fields),
                                                components,
                                            });
                                        }
                                    }
                                    RecordKind::Delta => emit(ReplicationChange::EntityUpdated {
                                        id: EntityId(id),
                                        fields: record_fields(record.fields),
                                        components,
                                    }),
                                    RecordKind::Delete => {
                                        emit(ReplicationChange::EntityDeleted { id: EntityId(id) })
                                    }
                                    RecordKind::End => {}
                                }
                            }
                            if let Some(controls) = frame.controls {
                                for entry in controls.control_entries {
                                    let control = match entry.payload {
                                        ControlPayload::Input(input) => ReplicationControl::Input {
                                            index: ControlPlayerIndex(input.index),
                                            short: input.short,
                                            analog: input.analog,
                                            third_analog: input.third_analog,
                                            extra: input.extra,
                                            flags: input.flags,
                                        },
                                        ControlPayload::Commands { index, records } => {
                                            ReplicationControl::Commands {
                                                index: ControlPlayerIndex(index),
                                                command_count: records.map(|records| records.len()),
                                            }
                                        }
                                        ControlPayload::Kind2 { value } => {
                                            ReplicationControl::Kind2 { value }
                                        }
                                    };
                                    emit(ReplicationChange::Control { control });
                                }
                            }
                        }
                    }
                    ReplicationStreamPacketBody::KeyframesPacketBody(table) => {
                        for attempt in table.records {
                            let Some(record) = attempt.record else {
                                continue;
                            };
                            if record.stop != KeyframeStop::Complete
                                || record.components.iter().any(|component| {
                                    component.status != ComponentReadStatus::Complete
                                })
                            {
                                continue;
                            }
                            emit(ReplicationChange::Keyframe {
                                id: KeyframeId(record.id),
                                archetype: ArchetypeId(record.archetype),
                                default_fields: default_fields(record.default_state),
                                fields: record_fields(record.fields),
                                components: record
                                    .components
                                    .into_iter()
                                    .map(|component| ReplicationComponentChange {
                                        index: ComponentSlot(component.index),
                                        name: component.name,
                                        fields: project_fields(component.fields),
                                        weapon_id: None,
                                    })
                                    .collect(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        ReplicationStream { observations }
    }
}

fn project_kill(words: &[ReplicationEventWord]) -> Option<ReplicationKill> {
    let mut cursor = 0;
    let victim = kill_actor(words, &mut cursor)?;
    let killer = kill_actor(words, &mut cursor)?;
    let killer_percent = take_event_word(words, &mut cursor, 32)? as u32;
    let flag = take_event_word(words, &mut cursor, 1)? == 1;
    let assist = kill_actor(words, &mut cursor)?;
    let assist_percent = take_event_word(words, &mut cursor, 32)? as u32;
    let suffix = &words[cursor..];
    if !suffix.is_empty() && suffix.iter().map(|w| w.bit_len).collect::<Vec<_>>() != [32, 32, 4] {
        return None;
    }
    Some(ReplicationKill {
        victim,
        killer,
        assist,
        killer_percent,
        assist_percent,
        flag,
        playback_suffix: suffix.to_vec(),
    })
}

fn take_event_word(
    words: &[ReplicationEventWord],
    cursor: &mut usize,
    width: usize,
) -> Option<u64> {
    let word = words.get(*cursor)?;
    if word.bit_len != width || (width < 64 && word.value >= (1_u64 << width)) {
        return None;
    }
    *cursor += 1;
    Some(word.value)
}

fn kill_actor(words: &[ReplicationEventWord], cursor: &mut usize) -> Option<Option<u8>> {
    if take_event_word(words, cursor, 1)? == 1 {
        Some(None)
    } else {
        Some(Some(take_event_word(words, cursor, 5)? as u8))
    }
}

fn project_fields(fields: Vec<ComponentField>) -> Vec<ReplicationField> {
    fields
        .into_iter()
        .filter(|field| {
            field.raw.bit_len == field.width
                && field.raw.bytes.len() == field.width.div_ceil(8)
                && !field.name.contains("corruption_check")
        })
        .map(|field| {
            let value = match field.width {
                0 => ReplicationValue::Empty,
                32 if field.name == "mpp.object_tag" => {
                    ReplicationValue::Tag(crate::theater::film::TagId(field.raw.low_u64() as u32))
                }
                1..=64 => ReplicationValue::Scalar {
                    value: field.raw.low_u64(),
                    bit_len: field.width,
                },
                _ => ReplicationValue::Wide {
                    bit_len: field.width,
                    bytes: field.raw.bytes,
                },
            };
            ReplicationField {
                name: field.name,
                value,
            }
        })
        .collect()
}

fn record_fields(fields: Vec<ComponentField>) -> Vec<ReplicationField> {
    project_fields(fields)
        .into_iter()
        .filter(|field| {
            !field.name.starts_with("record.")
                && !field.name.starts_with("new.")
                && !field.name.starts_with("delete.")
                && !field.name.starts_with("mask.")
                && field.name != "archetype"
                && field.name != "baseline"
        })
        .collect()
}

fn default_fields(state: Option<DefaultState>) -> Vec<ReplicationField> {
    state
        .filter(|state| state.status == DefaultStateStatus::Complete)
        .map(|state| project_fields(state.fields))
        .unwrap_or_default()
}

impl V41ReplicationStreamChunkReader {
    pub(crate) fn read(
        input: InflatedFilmChunk,
        context: &mut DecodeContext<'_>,
        parse_context: &FilmParseContext,
        max_packets: usize,
    ) -> Result<ParsedReplicationStreamChunk, crate::theater::film::ParseError> {
        let chunk_index = input.index;
        let body = input.read_packets(max_packets, |packet_index, header, payload| {
            let vehicle_gate = |id| {
                parse_context
                    .vehicle_snapshot_gates
                    .get(&VehicleSnapshotSite {
                        chunk_index,
                        packet_index,
                        id,
                    })
                    .copied()
            };
            let delta_schema = |start_bit, id| {
                parse_context
                    .delta_record_archetypes
                    .get(&DeltaRecordSite {
                        chunk_index,
                        packet_index,
                        start_bit,
                        id,
                    })
                    .copied()
            };
            match replication_stream::decode_with_context(
                header.packet_type,
                payload,
                context,
                vehicle_gate,
                delta_schema,
            ) {
                Ok(body) => PacketRead::Decoded(body),
                Err(reason) => PacketRead::Opaque { reason },
            }
        });
        Ok(ParsedReplicationStreamChunk {
            index: input.index,
            start_ms: input.start_ms,
            data: input.data,
            body: body?,
        })
    }
}

#[cfg(test)]
mod projection_tests {
    use super::*;
    use crate::theater::parser::packet::{ByteRange, FilmPacketHeader, Packet, PacketSource};
    use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;
    use crate::theater::parser::v41::chunks::replication::source::{
        ComponentReadStatus, ControlEntry, ControlInput, DecodedFrameView, EntityComponentRead,
        EntityRecord, EventField, EventListRead, EventListStop, EventRecord, FramePacket,
        FrameRead, FrameViewStop, ProductionFrame, RecordHeader,
    };

    fn field(name: &str, value: u64, width: usize) -> ComponentField {
        ComponentField {
            name: name.into(),
            bit: 17,
            width,
            raw: RawBits::from_low(value, width),
        }
    }

    #[test]
    fn kill_projection_requires_the_verified_field_shape() {
        let word = |value, bit_len| ReplicationEventWord { value, bit_len };
        let words = vec![
            word(0, 1),
            word(3, 5), // victim
            word(1, 1), // absent killer
            word(110, 32),
            word(1, 1),
            word(0, 1),
            word(12, 5), // assist
            word(45, 32),
            word(9, 32),
            word(10, 32),
            word(2, 4),
        ];
        let kill = project_kill(&words).unwrap();
        assert_eq!(
            (kill.victim, kill.killer, kill.assist),
            (Some(3), None, Some(12))
        );
        assert_eq!((kill.killer_percent, kill.assist_percent), (110, 45));
        assert!(kill.flag);
        assert_eq!(kill.playback_suffix, words[8..]);
        let mut wrong_width = words;
        wrong_width[3].bit_len = 31;
        assert!(project_kill(&wrong_width).is_none());
    }

    #[test]
    fn known_tag_and_unknown_scalar_keep_distinct_types() {
        let fields = project_fields(vec![
            field("mpp.object_tag", 0x1234_5678, 32),
            field("other.word", 0x75, 8),
        ]);
        assert_eq!(
            fields[0].value,
            ReplicationValue::Tag(crate::theater::film::TagId(0x1234_5678))
        );
        assert_eq!(
            fields[1].value,
            ReplicationValue::Scalar {
                value: 0x75,
                bit_len: 8
            }
        );
    }

    #[test]
    fn projection_preserves_observation_order_and_drops_source_coordinates() {
        let event = EventRecord {
            start_bit: 1,
            end_bit: 21,
            code: Some(7),
            body_start_bit: Some(9),
            fields: vec![
                EventField {
                    bit: 1,
                    width: 8,
                    stage: EventFieldStage::Header,
                    value: EventFieldValue::Scalar(7),
                },
                EventField {
                    bit: 9,
                    width: 8,
                    stage: EventFieldStage::Body,
                    value: EventFieldValue::Scalar(42),
                },
                EventField {
                    bit: 17,
                    width: 4,
                    stage: EventFieldStage::CorruptionCheck,
                    value: EventFieldValue::Scalar(15),
                },
            ],
            layout_complete: true,
        };
        let record = EntityRecord {
            header: RecordHeader {
                prefix: None,
                kind: RecordKind::Delta,
                id: Some(91),
                start_bit: 22,
                end_bit: 30,
            },
            archetype: None,
            mask: Some(1),
            fields: Vec::new(),
            default_state: None,
            components: vec![EntityComponentRead {
                index: 0,
                name: "health".into(),
                start_bit: 31,
                end_bit: 38,
                status: ComponentReadStatus::Complete,
                fields: vec![
                    field("empty_schema_field", 0, 0),
                    field("amount", 75, 7),
                    field("wide_flags", u64::MAX, 129),
                ],
                weapon_id: None,
            }],
            end_bit: 38,
            stop: EntityViewStop::Complete,
        };
        let control = ControlEntry {
            start_bit: 40,
            end_bit: 51,
            baseline: None,
            payload: ControlPayload::Input(ControlInput {
                index: 2,
                short: Some(3),
                analog: None,
                third_analog: None,
                extra: None,
                flags: Some(1),
                secondary: None,
            }),
        };
        let frame = FramePacket {
            configuration: Some(false),
            events: EventListRead {
                start_bit: 1,
                end_bit: 22,
                gate15: None,
                script_prefix_context_used: None,
                assumptions: Vec::new(),
                kill_tail_context_used: None,
                records: vec![event],
                terminator: None,
                stop: EventListStop::Terminator,
            },
            frame: FrameRead::Decoded(Box::new(ProductionFrame {
                messages: None,
                records: vec![record],
                controls: Some(DecodedFrameView {
                    control_entries: vec![control],
                    start_bit: 40,
                    end_bit: 51,
                    kinds: vec![0],
                    fields: Vec::new(),
                    stop: FrameViewStop::Complete,
                }),
                entity_end: None,
                end_bit: 51,
                external_delta_schemas: Vec::new(),
            })),
        };
        let packet = Packet {
            source: PacketSource {
                header: ByteRange { start: 0, end: 16 },
                payload: ByteRange { start: 16, end: 23 },
            },
            header: FilmPacketHeader {
                packet_type: 0,
                unknown_2: [0; 2],
                payload_size: 7,
                timestamp_us: 123,
            },
            body: PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(Box::new(
                frame,
            ))),
        };
        let stream = ParsedReplicationStream(vec![ParsedReplicationStreamChunk {
            index: 4,
            start_ms: 999,
            data: Vec::new(),
            body: PacketStream {
                packets: vec![packet],
                opaque: Vec::new(),
            },
        }])
        .into_typed();

        assert_eq!(stream.observations.len(), 3);
        assert!(stream.observations.iter().all(|o| o.timestamp_us == 123));
        assert!(matches!(
            &stream.observations[0].change,
            ReplicationChange::Event { code: crate::theater::film::ReplicationEventKind::Unknown(7), body: ReplicationEventBody::Fields(body), .. } if body == &[ReplicationEventWord { value: 42, bit_len: 8 }]
        ));
        assert!(matches!(
            &stream.observations[1].change,
            ReplicationChange::EntityUpdated { id: EntityId(91), components, .. }
                if components[0].fields[0].value == ReplicationValue::Empty
                    && components[0].fields[1].value == ReplicationValue::Scalar { value: 75, bit_len: 7 }
                    && matches!(&components[0].fields[2].value,
                        ReplicationValue::Wide { bit_len: 129, bytes } if bytes.len() == 17)
        ));
        assert!(matches!(
            &stream.observations[2].change,
            ReplicationChange::Control {
                control: ReplicationControl::Input {
                    index: ControlPlayerIndex(2),
                    ..
                }
            }
        ));
        let json = serde_json::to_string(&stream).unwrap();
        for absent in [
            "chunk_index",
            "packet_index",
            "start_bit",
            "end_bit",
            "raw",
            "header",
        ] {
            assert!(!json.contains(absent), "{absent} leaked into {json}");
        }
    }
}
