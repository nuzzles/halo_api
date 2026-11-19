//! Consume parsed packet evidence into finished film observations.
use super::{ParsedReplicationStream, event, source};
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
use source::{
    ComponentReadStatus, ControlPayload, DefaultState, DefaultStateStatus, EntityViewStop,
    KeyframeStop, RecordKind, ReplicationStreamPacketBody,
};

impl From<ParsedReplicationStream> for ReplicationStream {
    fn from(parsed: ParsedReplicationStream) -> Self {
        let mut observations = Vec::new();
        let schema_kinds = parsed.schema_kinds;
        for chunk in parsed.chunks {
            for packet in chunk.body.packets {
                let timestamp_us = packet.header.timestamp_us;
                let payload = packet.payload(&chunk.data);
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
                            let (Some(code), Some(payload)) = (record.code, payload) else {
                                continue;
                            };
                            let Some(start) = record.start_bit.checked_add(8) else {
                                continue;
                            };
                            let Some(bit_len) = record.end_bit.checked_sub(start) else {
                                continue;
                            };
                            let Some(event) = event::project(&record, payload).or_else(|| {
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
                        if let crate::theater::parser::v41::replication::source::FrameRead::Decoded(frame) = frame.frame {
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
                                    })
                                    .collect();
                                match record.header.kind {
                                    RecordKind::New => {
                                        if let Some(archetype) = record.archetype {
                                            emit(ReplicationChange::EntityCreated {
                                                id: EntityId::from_raw(id),
                                                schema: EntitySchemaId(archetype),
                                                schema_kind: schema_kinds
                                                    .get(&EntitySchemaId(archetype))
                                                    .copied()
                                                    .unwrap_or(crate::theater::film::EntitySchemaKind::Unknown),
                                                default_fields: default_fields(
                                                    record.default_state,
                                                ),
                                                fields: record_fields(record.fields),
                                                components,
                                            });
                                        }
                                    }
                                    RecordKind::Delta => emit(ReplicationChange::EntityUpdated {
                                        id: EntityId::from_raw(id),
                                        fields: record_fields(record.fields),
                                        components,
                                    }),
                                    RecordKind::Delete => {
                                        emit(ReplicationChange::EntityDeleted { id: EntityId::from_raw(id) })
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
                                schema: EntitySchemaId(record.archetype),
                                schema_kind: schema_kinds
                                    .get(&EntitySchemaId(record.archetype))
                                    .copied()
                                    .unwrap_or(crate::theater::film::EntitySchemaKind::Unknown),
                                default_fields: default_fields(record.default_state),
                                fields: record_fields(record.fields),
                                components: record
                                    .components
                                    .into_iter()
                                    .map(|component| ReplicationComponentChange {
                                        index: ComponentSlot(component.index),
                                        name: component.name,
                                        fields: project_fields(component.fields),
                                    })
                                    .collect(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        Self { observations }
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
                    let id = crate::theater::film::TagId(field.raw.low_u64() as u32);
                    super::tag::value(id)
                }
                _ => ReplicationValue::Bits(field.raw.into()),
            };
            ReplicationField {
                name: field.name,
                value,
            }
        })
        .collect()
}

impl From<RawBits> for BitValue {
    fn from(raw: RawBits) -> Self {
        Self {
            bit_len: raw.bit_len,
            bytes: raw.bytes,
        }
    }
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
