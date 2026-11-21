//! Consume parsed packet evidence into finished film observations.
use super::{ParsedReplicationStream, ParsedReplicationStreamChunk, event, source};
use crate::theater::film::replication::{
    ReplicationChange, ReplicationComponentChange, ReplicationControl, ReplicationField,
    ReplicationObservation, ReplicationStream, ReplicationValue,
};
use crate::theater::film::{
    BitValue, ComponentSlot, ControlPlayerIndex, EntityId, EntitySchemaId, KeyframeId,
};
use crate::theater::parser::v41::packet::PacketRead;
use crate::theater::parser::wire::fields::RawBits;
use source::components::field::ComponentField;
use source::stream::models::entity_records::EntityComponentRead;
use source::{
    ComponentReadStatus, ControlPayload, DefaultState, DefaultStateStatus, EntityViewStop,
    KeyframeStop, RecordKind, ReplicationStreamPacketBody,
};

impl From<ParsedReplicationStream> for ReplicationStream {
    fn from(parsed: ParsedReplicationStream) -> Self {
        Self {
            observations: parsed.observations,
        }
    }
}

pub(super) fn project_chunks(
    chunks: Vec<ParsedReplicationStreamChunk>,
    schema_kinds: &std::collections::BTreeMap<
        EntitySchemaId,
        crate::theater::film::EntitySchemaKind,
    >,
    config: &crate::theater::parser::v41::context::V41DecodeConfig,
) -> Vec<ReplicationObservation> {
    let capacity = chunks
        .iter()
        .flat_map(|chunk| &chunk.body.packets)
        .map(|packet| match &packet.body {
            PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) => {
                frame.events.records.len()
                    + match &frame.frame {
                        crate::theater::parser::v41::replication::source::FrameRead::Decoded(
                            frame,
                        ) => {
                            frame.records.len()
                                + frame
                                    .controls
                                    .as_ref()
                                    .map_or(0, |controls| controls.control_entries.len())
                        }
                        _ => 0,
                    }
            }
            PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) => {
                table.records.len()
            }
            _ => 0,
        })
        .sum();
    let mut observations = Vec::with_capacity(capacity);
    for chunk in chunks {
        for packet in chunk.body.packets {
            let timestamp_us = packet.header.timestamp_us;
            let Some(payload) = packet.payload(&chunk.data) else {
                continue;
            };
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
                        let Some(code) = record.code else {
                            continue;
                        };
                        let Some(start) = record.start_bit.checked_add(8) else {
                            continue;
                        };
                        let Some(bit_len) = record.end_bit.checked_sub(start) else {
                            continue;
                        };
                        let Some(event) = event::project(&record, payload, config).or_else(|| {
                            RawBits::from_source(payload, start, bit_len).map(|raw| {
                                crate::theater::film::ReplicationEvent::Raw {
                                    kind: code.into(),
                                    payload: raw.into(),
                                }
                            })
                        }) else {
                            continue;
                        };
                        emit(ReplicationChange::Event { event });
                    }
                    if let crate::theater::parser::v41::replication::source::FrameRead::Decoded(
                        frame,
                    ) = frame.frame
                    {
                        for record in frame.records {
                            if record.stop != EntityViewStop::Complete {
                                continue;
                            }
                            let Some(id) = record.header.id else {
                                continue;
                            };
                            let complete_components = record
                                .components
                                .iter()
                                .all(|component| component.status == ComponentReadStatus::Complete);
                            if !complete_components {
                                continue;
                            }
                            let (fields, components) =
                                project_entity_fields(record.fields, record.components, payload);
                            match record.header.kind {
                                RecordKind::New => {
                                    if let Some(archetype) = record.archetype {
                                        emit(ReplicationChange::EntityCreated {
                                            id: EntityId::from_raw(id),
                                            schema: EntitySchemaId(archetype),
                                            schema_kind: schema_kinds
                                                .get(&EntitySchemaId(archetype))
                                                .copied()
                                                .unwrap_or(
                                                    crate::theater::film::EntitySchemaKind::Unknown,
                                                ),
                                            default_fields: default_fields(
                                                record.default_state,
                                                payload,
                                            ),
                                            fields,
                                            components,
                                        });
                                    }
                                }
                                RecordKind::Delta => emit(ReplicationChange::EntityUpdated {
                                    id: EntityId::from_raw(id),
                                    fields,
                                    components,
                                }),
                                RecordKind::Delete => emit(ReplicationChange::EntityDeleted {
                                    id: EntityId::from_raw(id),
                                }),
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
                            || record
                                .components
                                .iter()
                                .any(|component| component.status != ComponentReadStatus::Complete)
                        {
                            continue;
                        }
                        emit(ReplicationChange::Keyframe {
                            id: KeyframeId(record.id),
                            schema: EntitySchemaId(record.archetype),
                            schema_kind: schema_kinds
                                .get(&EntitySchemaId(record.archetype))
                                .copied()
                                .unwrap_or(crate::theater::film::EntitySchemaKind::Unknown),
                            default_fields: default_fields(record.default_state, payload),
                            fields: record_fields(record.fields, payload),
                            components: record
                                .components
                                .into_iter()
                                .map(|component| ReplicationComponentChange {
                                    index: ComponentSlot(component.index),
                                    name: component.name,
                                    fields: project_fields(component.fields, payload),
                                })
                                .collect(),
                        });
                    }
                }
                ReplicationStreamPacketBody::RosterPacketBody(read) => {
                    if let Some(players) = super::stream::roster::roster(&read) {
                        emit(ReplicationChange::Roster { players });
                    }
                }
                ReplicationStreamPacketBody::BotsPacketBody(read) => {
                    if let Some(players) = super::stream::roster::bots(&read) {
                        emit(ReplicationChange::Bots { players });
                    }
                }
                _ => {}
            }
        }
    }
    observations
}

fn project_fields(
    fields: impl IntoIterator<Item = ComponentField>,
    payload: &[u8],
) -> Vec<ReplicationField> {
    let fields = fields.into_iter();
    let (lower_bound, upper_bound) = fields.size_hint();
    // Record fields pass through a filter whose upper bound includes framing
    // fields. Only reserve eagerly when the iterator has a known exact size.
    let capacity = if upper_bound == Some(lower_bound) {
        lower_bound
    } else {
        0
    };
    let mut projected = Vec::with_capacity(capacity);
    for field in fields {
        if field.name.contains("corruption_check") {
            continue;
        }
        let value = match field.raw.bit_len {
            0 => ReplicationValue::Empty,
            32 if field.name.as_ref() == "mpp.object_tag" => {
                let id = crate::theater::film::TagId(field.raw.low_u64() as u32);
                super::tag::value(id)
            }
            _ => {
                let Some(bits) = field.materialize(payload) else {
                    continue;
                };
                ReplicationValue::Bits(bits.into())
            }
        };
        projected.push(ReplicationField {
            name: field.name,
            value,
        });
    }
    projected
}

impl From<RawBits> for BitValue {
    fn from(raw: RawBits) -> Self {
        Self {
            bit_len: raw.bit_len,
            bytes: raw.bytes,
        }
    }
}

fn record_fields(fields: Vec<ComponentField>, payload: &[u8]) -> Vec<ReplicationField> {
    project_fields(
        fields.into_iter().filter(|field| {
            !field.name.starts_with("record.")
                && !field.name.starts_with("new.")
                && !field.name.starts_with("delete.")
                && !field.name.starts_with("mask.")
                && field.name.as_ref() != "archetype"
                && field.name.as_ref() != "baseline"
        }),
        payload,
    )
}

fn project_entity_fields(
    fields: Vec<ComponentField>,
    components: Vec<EntityComponentRead>,
    payload: &[u8],
) -> (Vec<ReplicationField>, Vec<ReplicationComponentChange>) {
    let spans = components
        .iter()
        .filter_map(|component| {
            Some((
                usize::try_from(component.start_bit).ok()?,
                usize::try_from(component.end_bit).ok()?,
            ))
        })
        .collect::<smallvec::SmallVec<[(usize, usize); 4]>>();
    let mut source_fields = fields.into_iter();
    let mut consumed = 0;
    let mut scope_fields = Vec::new();
    let mut projected_components = Vec::with_capacity(components.len());
    for component in components {
        debug_assert!(component.field_range.start >= consumed);
        scope_fields.extend(
            source_fields
                .by_ref()
                .take(component.field_range.start - consumed),
        );
        let count = component.field_range.len();
        let current_span = (
            usize::try_from(component.start_bit).ok(),
            usize::try_from(component.end_bit).ok(),
        );
        let fields = project_fields(
            source_fields.by_ref().take(count).inspect(|field| {
                let within_current = current_span
                    .0
                    .zip(current_span.1)
                    .is_some_and(|(start, end)| field.bit >= start && field.bit < end);
                if !within_current
                    && !spans
                        .iter()
                        .any(|&(start, end)| field.bit >= start && field.bit < end)
                {
                    scope_fields.push(field.clone());
                }
            }),
            payload,
        );
        projected_components.push(ReplicationComponentChange {
            index: ComponentSlot(component.index),
            name: component.name,
            fields,
        });
        consumed = component.field_range.end;
    }
    scope_fields.extend(source_fields);
    scope_fields.retain(|field| {
        !spans
            .iter()
            .any(|&(start, end)| field.bit >= start && field.bit < end)
    });
    (record_fields(scope_fields, payload), projected_components)
}

fn default_fields(state: Option<DefaultState>, payload: &[u8]) -> Vec<ReplicationField> {
    state
        .filter(|state| state.status == DefaultStateStatus::Complete)
        .map(|state| project_fields(state.fields, payload))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{ComponentField, EntityComponentRead, project_entity_fields};
    use crate::theater::parser::v41::replication::source::ComponentReadStatus;
    use crate::theater::parser::v41::replication::source::components::field::FieldBits;

    #[test]
    fn component_ranges_preserve_out_of_range_record_fields() {
        let payload = [0xff; 10];
        let field = |name: &str, bit| ComponentField {
            name: name.into(),
            bit,
            raw: FieldBits::from_source(&payload, bit, 1).unwrap(),
        };
        let fields = vec![
            field("record.header", 0),
            field("header", 1),
            field("inside_a", 12),
            field("outside_a", 32),
            field("inside_b", 61),
            field("outside_b", 70),
        ];
        let components = [
            EntityComponentRead {
                index: 0,
                name: "a".into(),
                start_bit: 8,
                end_bit: 24,
                status: ComponentReadStatus::Complete,
                field_range: 2..4,
            },
            EntityComponentRead {
                index: 1,
                name: "b".into(),
                start_bit: 40,
                end_bit: 64,
                status: ComponentReadStatus::Complete,
                field_range: 4..6,
            },
        ];
        let (record, components) =
            project_entity_fields(fields.clone(), components.to_vec(), &payload);
        assert_eq!(
            record
                .iter()
                .map(|field| field.name.as_ref())
                .collect::<Vec<_>>(),
            ["header", "outside_a", "outside_b"]
        );
        assert_eq!(
            components[0]
                .fields
                .iter()
                .map(|field| field.name.as_ref())
                .collect::<Vec<_>>(),
            ["inside_a", "outside_a"]
        );
        assert_eq!(
            components[1]
                .fields
                .iter()
                .map(|field| field.name.as_ref())
                .collect::<Vec<_>>(),
            ["inside_b", "outside_b"]
        );

        let source_components = [
            EntityComponentRead {
                index: 0,
                name: "a".into(),
                start_bit: 8,
                end_bit: 24,
                status: ComponentReadStatus::Complete,
                field_range: 2..4,
            },
            EntityComponentRead {
                index: 1,
                name: "b".into(),
                start_bit: 24,
                end_bit: 64,
                status: ComponentReadStatus::Complete,
                field_range: 4..6,
            },
        ];
        let (record, _) = project_entity_fields(fields, source_components.to_vec(), &payload);
        assert_eq!(
            record
                .iter()
                .map(|field| field.name.as_ref())
                .collect::<Vec<_>>(),
            ["header", "outside_b"]
        );
    }
}
