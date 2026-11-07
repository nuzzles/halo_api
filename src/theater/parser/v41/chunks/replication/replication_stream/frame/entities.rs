//! Reference default production frame policy: message/entity/control classes and world admission.
use crate::theater::film::{
    ComponentField, EntityRecord, EntityViewStop, FilmRegistry, FrameViewStop, RawBits, RecordKind,
};
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::controls::{
    decode_control_view_contextual, decode_message_view_signed,
};
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::header::decode_frame_header_signed;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::records::{
    RecordBinding, RecordReadContext, decode_entity_record,
};
use crate::theater::parser::v41::chunks::replication::state::{
    DeltaAdmission, ReplicationDecodeState,
};
use crate::theater::parser::v41::context::layout::RecordLayout;
use crate::theater::parser::v41::context::profile::DecodeProfile;

use crate::theater::film::chunks::replication::replication_stream::models::production_frame::{
    ProductionEntityEnd, ProductionFrame,
};

#[derive(Clone, Copy)]
pub(crate) struct ProductionReaderContext<'a> {
    pub reader: Option<&'a DecodeProfile>,
    pub preamble_bits: i64,
}

pub(crate) fn decode_production_frame_contextual(
    data: &[u8],
    start_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    world: &mut ReplicationDecodeState,
    context: ProductionReaderContext<'_>,
    observer: impl FnMut(&EntityRecord, u32, &ReplicationDecodeState),
) -> Option<ProductionFrame> {
    read_production_frame::<true>(
        data,
        start_bit.try_into().ok()?,
        registry,
        encoding,
        world,
        context,
        observer,
    )
}

fn read_production_frame<const ALLOW_REPLACEMENT: bool>(
    data: &[u8],
    start_bit: i64,
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    world: &mut ReplicationDecodeState,
    context: ProductionReaderContext<'_>,
    mut observer: impl FnMut(&EntityRecord, u32, &ReplicationDecodeState),
) -> Option<ProductionFrame> {
    if !encoding.valid() {
        return None;
    }
    let mut body_encoding = std::borrow::Cow::Borrowed(encoding);
    if encoding.extra_fields {
        body_encoding.to_mut().extra_fields = false;
    }
    let mut out = ProductionFrame {
        messages: None,
        records: vec![],
        controls: None,
        entity_end: None,
        end_bit: start_bit,
    };
    if start_bit == context.preamble_bits && context.preamble_bits >= 1 {
        let messages = decode_message_view_signed(data, context.preamble_bits - 1);
        out.end_bit = messages.end_bit;
        let complete = messages.stop == FrameViewStop::Complete;
        out.messages = Some(messages);
        if !complete {
            return Some(out);
        }
    }
    world.current_view = 0;
    let mut hit_end = false;
    let mut capture_slot = 0;
    for _ in 0..8192 {
        if out.end_bit >= data.len().saturating_mul(8) as i64 {
            break;
        }
        let prefix = if encoding.extra_fields {
            let start = usize::try_from(out.end_bit).ok()?;
            let Some(raw) = RawBits::from_source(data, start, 32) else {
                out.entity_end = Some(ProductionEntityEnd::Truncated);
                break;
            };
            Some(ComponentField {
                name: "record.prefix".into(),
                bit: start,
                width: 32,
                raw,
            })
        } else {
            None
        };
        let header_bit = out
            .end_bit
            .wrapping_add(if encoding.extra_fields { 32 } else { 0 });
        let Some(mut header) = decode_frame_header_signed(data, header_bit, encoding) else {
            out.entity_end = Some(ProductionEntityEnd::Truncated);
            break;
        };
        header.prefix = prefix.clone();
        if header.kind == RecordKind::End {
            out.end_bit = header.end_bit;
            out.entity_end = Some(ProductionEntityEnd::Marker(header));
            hit_end = true;
            break;
        }
        let id = header.id?;
        if header.kind == RecordKind::Delta {
            let mut reason = world.admit_delta(id, true);
            if matches!(reason, DeltaAdmission::Unbound)
                && let Some(archetype) = infer_unbound_archetype(
                    data,
                    header_bit,
                    id,
                    registry,
                    &body_encoding,
                    world,
                    context.reader,
                )
            {
                world.bind_inferred(id, archetype);
                reason = world.admit_delta(id, true);
            }
            if matches!(reason, DeltaAdmission::Unbound | DeltaAdmission::OtherView) {
                out.end_bit = header.end_bit;
                out.entity_end = Some(ProductionEntityEnd::Rejected { header });
                hit_end = true;
                break;
            }
        }
        if header.kind == RecordKind::Delta {
            capture_slot = id & 0x3fff_ffff;
        }
        let binding = world
            .archetype(id & 0x3fff_ffff)
            .map(|archetype| RecordBinding { id, archetype });
        let Some(mut record) = decode_entity_record(
            data,
            header_bit,
            registry,
            &body_encoding,
            binding,
            context.reader.is_none_or(|c| c.grammar.simulation_complete),
            RecordReadContext {
                context: context.reader,
                movement: Some(capture_slot),
            },
        ) else {
            out.entity_end = Some(ProductionEntityEnd::Truncated);
            break;
        };
        record.header.prefix = prefix;
        out.end_bit = record.end_bit;
        observer(&record, capture_slot, world);
        let complete = record.stop == EntityViewStop::Complete;
        // The NEW declaration establishes the grammar for later deltas once
        // its ID and archetype are recorded. A truncated component/default body
        // does not erase that declaration or make its missing state complete.
        // Resolution separately decides whether a lifetime/state is materialized.
        match record.header.kind {
            RecordKind::New
                if matches!(
                    record.stop,
                    EntityViewStop::Complete | EntityViewStop::Truncated
                ) =>
            {
                if let Some(archetype) = record
                    .archetype
                    .filter(|ti| *ti < 50 && mask_matches_schema(&record, *ti, registry))
                    && world
                        .bind_reference_new(id, archetype, record.header.start_bit)
                        .is_some()
                    && complete
                    && !record.components.is_empty()
                    && record.default_state.as_ref().is_some_and(|defaults| {
                        defaults.status == crate::theater::film::DefaultStateStatus::Complete
                    })
                    && ALLOW_REPLACEMENT
                    && world
                        .slots
                        .get(&(id & 0x3fff_ffff))
                        .is_some_and(|binding| !binding.generation_any && binding.full_id != id)
                {
                    // A new generation may reuse the slot with another schema.
                    // Require a decoded body and a clean continuation; an opaque,
                    // empty NEW can coincidentally end at the packet boundary.
                    let mut candidate = world.clone();
                    candidate.bind_full(id, archetype);
                    if read_production_frame::<false>(
                        data,
                        record.end_bit,
                        registry,
                        encoding,
                        &mut candidate,
                        context,
                        (|_, _, _| {}) as fn(&EntityRecord, u32, &ReplicationDecodeState),
                    )
                    .is_some_and(|frame| super::production_frame_closed(&frame, data))
                    {
                        world.bind_full(id, archetype);
                    }
                }
            }
            // A DELETE with a recorded header unbinds even if its metadata is
            // truncated. An unclosed frame can still prove a mismatched-ID
            // DELETE was speculative, so that binding is restored afterward.
            RecordKind::Delete => world.unbind_frame_delete(id),
            _ => {}
        }
        if !complete {
            out.entity_end = Some(ProductionEntityEnd::Failure(record.stop.clone()));
        }
        out.records.push(record);
        if !complete {
            break;
        }
    }
    if hit_end {
        let controls = decode_control_view_contextual(
            data,
            out.end_bit,
            encoding.position.as_ref(),
            context.reader,
        );
        out.end_bit = controls.end_bit;
        out.controls = Some(controls);
    } else if out.entity_end.is_none() {
        out.entity_end = Some(if out.records.len() == 8192 {
            ProductionEntityEnd::RecordLimit
        } else {
            ProductionEntityEnd::PayloadBoundary
        });
    }
    let source_end = i64::try_from(data.len().saturating_mul(8)).ok()?;
    if out.end_bit > source_end {
        out.end_bit = source_end;
        if let Some(controls) = &mut out.controls {
            controls.end_bit = controls.end_bit.min(source_end);
            controls.stop = FrameViewStop::Truncated;
        }
    }
    Some(out)
}

// Out-of-range mask bits may be traversable, but cannot establish a schema.
fn mask_matches_schema(record: &EntityRecord, archetype: u32, registry: &FilmRegistry) -> bool {
    registry
        .archetype(archetype as usize)
        .is_some_and(|schema| {
            record.mask.is_none_or(|mask| {
                schema.components.len() >= u64::BITS as usize
                    || mask >> schema.components.len() == 0
            })
        })
}

/// A single valid body or apparent packet closure is not enough: the next
/// record must be a clean delta with a known schema and an increasing slot.
fn infer_unbound_archetype(
    data: &[u8],
    start_bit: i64,
    id: u32,
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    world: &ReplicationDecodeState,
    context: Option<&DecodeProfile>,
) -> Option<u32> {
    let mut inferred = None;
    for archetype in 0..registry.archetypes.len().min(50) {
        let Some(record) = decode_entity_record(
            data,
            start_bit,
            registry,
            encoding,
            Some(RecordBinding {
                id,
                archetype: archetype as u32,
            }),
            context.is_none_or(|c| c.grammar.simulation_complete),
            RecordReadContext {
                context,
                movement: None,
            },
        ) else {
            continue;
        };
        if record.stop != EntityViewStop::Complete
            || record.components.is_empty()
            || !mask_matches_schema(&record, archetype as u32, registry)
        {
            continue;
        }
        let successor_bit = record.end_bit + if encoding.extra_fields { 32 } else { 0 };
        let Some(successor) = decode_frame_header_signed(data, successor_bit, encoding) else {
            continue;
        };
        let Some(next_id) = successor.id.filter(|_| successor.kind == RecordKind::Delta) else {
            continue;
        };
        // FUN_142f2e174 emits deltas in strictly increasing slot order.
        if next_id & 0x3fff_ffff <= id & 0x3fff_ffff {
            continue;
        }
        let Some(binding) = world.slots.get(&(next_id & 0x3fff_ffff)).filter(|binding| {
            !binding.soft && (binding.generation_any || binding.full_id == next_id)
        }) else {
            continue;
        };
        let Some(next_record) = decode_entity_record(
            data,
            successor_bit,
            registry,
            encoding,
            Some(RecordBinding {
                id: next_id,
                archetype: binding.archetype,
            }),
            context.is_none_or(|c| c.grammar.simulation_complete),
            RecordReadContext {
                context,
                movement: None,
            },
        ) else {
            continue;
        };
        if next_record.stop != EntityViewStop::Complete
            || next_record.components.is_empty()
            || !mask_matches_schema(&next_record, binding.archetype, registry)
        {
            continue;
        }
        if inferred.is_some() {
            return None;
        }
        inferred = Some(archetype as u32);
    }
    inferred
}

/// Derived traversal statistic for pinned reference-oracle checks only.
#[cfg(test)]
pub(crate) fn completed_views(frame: &ProductionFrame) -> usize {
    usize::from(
        frame
            .messages
            .as_ref()
            .is_some_and(|v| v.stop == FrameViewStop::Complete),
    ) + usize::from(matches!(
        frame.entity_end,
        Some(ProductionEntityEnd::Marker(_) | ProductionEntityEnd::Rejected { .. })
    )) + usize::from(
        frame
            .controls
            .as_ref()
            .is_some_and(|v| v.stop == FrameViewStop::Complete),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::film::{ByteRange, FilmArchetype, RegistryComponent};
    use crate::theater::parser::v41::context::V41DecodeConfig;

    #[test]
    fn generation_replacement_requires_a_closed_packet_and_never_replaces_the_same_id() {
        let range = ByteRange { start: 0, end: 0 };
        let registry = FilmRegistry {
            archetypes: (0..22)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 21 {
                        vec![RegistryComponent {
                            name_bytes: b"flock-emitting-component".to_vec(),
                            precision_level: 0,
                            source: range,
                        }]
                    } else {
                        vec![]
                    },
                    source: range,
                    padding: range,
                })
                .collect(),
        };
        let config = V41DecodeConfig::default();
        for (same_id, closed, has_component, archetype, expected) in [
            (false, true, true, 21, 21),
            (false, false, true, 21, 3),
            (true, true, true, 21, 3),
            (false, true, false, 19, 3),
            (false, true, false, 21, 3),
        ] {
            let mut bits = Vec::new();
            let mut put = |width: usize, value: u64| {
                bits.extend((0..width).rev().map(|shift| value & (1 << shift) != 0))
            };
            for (width, value) in [(3, 1), (13, 693), (2, 1), (6, archetype)] {
                put(width, value);
            }
            if archetype == 21 {
                put(18, 0);
            }
            put(1, 0); // NEW component gate
            put(1, 0); // sparse mask
            put(3, u64::from(has_component));
            if has_component {
                put(6, 0);
                put(1, 1);
            }
            if closed {
                put(4, 0);
            } else {
                put(1, 1);
                put(13, 77);
                put(2, 1);
            }
            let mut payload = vec![0_u8; bits.len().div_ceil(8)];
            for (bit, set) in bits.into_iter().enumerate() {
                payload[bit / 8] |= u8::from(set) << (7 - bit % 8);
            }
            let mut world = ReplicationDecodeState::default();
            world.bind_full((if same_id { 1 } else { 3 }) << 30 | 693, 3);
            let mut observed = 0;
            let frame = decode_production_frame_contextual(
                &payload,
                0,
                &registry,
                &config.record_layout(),
                &mut world,
                ProductionReaderContext {
                    reader: Some(&config.profile),
                    preamble_bits: 2,
                },
                |_, _, _| observed += 1,
            )
            .unwrap();
            assert_eq!(frame.records[0].header.kind, RecordKind::New);
            assert_eq!(frame.records[0].stop, EntityViewStop::Complete);
            assert_eq!(observed, 1);
            assert_eq!(world.archetype(693), Some(expected));
        }
    }

    #[test]
    fn out_of_schema_mask_cannot_poison_a_later_player_declaration() {
        let payload = include_bytes!(
            "../../../../../../fixtures/movement-schema-v41/invalid-flock-declaration.bin"
        );
        let range = ByteRange { start: 0, end: 0 };
        let mut registry = FilmRegistry {
            archetypes: (0..22)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    source: range,
                    padding: range,
                })
                .collect(),
        };
        registry.archetypes[21].components = (0..18)
            .map(|index| RegistryComponent {
                name_bytes: match index {
                    0 => b"flock-emitting-component".to_vec(),
                    1 => b"flock-destroying-component".to_vec(),
                    _ => b"flock-destination-component".to_vec(),
                },
                precision_level: 1,
                source: range,
            })
            .collect();
        let config = V41DecodeConfig::default();
        let mut world = ReplicationDecodeState::default();
        let frame = decode_production_frame_contextual(
            payload,
            1061,
            &registry,
            &config.record_layout(),
            &mut world,
            ProductionReaderContext {
                reader: Some(&config.profile),
                preamble_bits: 2,
            },
            |_, _, _| {},
        )
        .unwrap();
        assert_eq!(frame.records[0].header.id, Some(1073742495));
        assert_eq!(frame.records[0].mask, Some(25770065923));
        assert_eq!(frame.records[0].stop, EntityViewStop::Complete);
        assert_eq!(world.archetype(671), None);
        assert_eq!(world.bind_reference_new(1073742495, 35, 89), None);
        assert_eq!(world.archetype(671), Some(35));
    }

    #[test]
    fn inferred_schema_cannot_use_an_earlier_slot_as_its_successor() {
        let payload = include_bytes!(
            "../../../../../../fixtures/movement-schema-v41/empyrean-false-successor.bin"
        );
        let declarations: Vec<(usize, Vec<(String, u32)>)> = serde_json::from_str(include_str!(
            "../../../../../../fixtures/movement-schema-v41/empyrean-false-successor-registry.json"
        ))
        .unwrap();
        let registry = crate::theater::parser::test_support::component_registry(&declarations);
        let config = V41DecodeConfig::default();
        let layout = config.record_layout();
        let mut world = ReplicationDecodeState::default();
        world.bind_full(1073742339, 35);
        let guessed = decode_entity_record(
            payload,
            1252,
            &registry,
            &layout,
            Some(RecordBinding {
                id: 1073743428,
                archetype: 13,
            }),
            false,
            RecordReadContext {
                context: Some(&config.profile),
                movement: None,
            },
        )
        .unwrap();
        assert_eq!(guessed.stop, EntityViewStop::Complete);
        assert_eq!(guessed.end_bit, 1341);
        let successor = decode_entity_record(
            payload,
            1341,
            &registry,
            &layout,
            Some(RecordBinding {
                id: 1073742339,
                archetype: 35,
            }),
            false,
            RecordReadContext {
                context: Some(&config.profile),
                movement: None,
            },
        )
        .unwrap();
        assert_eq!(successor.stop, EntityViewStop::Complete);
        assert!(mask_matches_schema(&successor, 35, &registry));
        assert_eq!(
            infer_unbound_archetype(
                payload,
                1252,
                1073743428,
                &registry,
                &layout,
                &world,
                Some(&config.profile)
            ),
            None
        );
        assert!(world.archetype(1604).is_none());
    }

    #[test]
    fn apparent_packet_closure_does_not_establish_an_unbound_schema() {
        let registry = FilmRegistry {
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![RegistryComponent {
                    name_bytes: b"game-engine-round-condition-flags-component\0".to_vec(),
                    precision_level: 0,
                    source: ByteRange { start: 0, end: 0 },
                }],
                source: ByteRange { start: 0, end: 0 },
                padding: ByteRange { start: 0, end: 0 },
            }],
        };
        let mut bits = Vec::new();
        for (width, value) in [
            (1, 1_u64), // delta header
            (13, 17),
            (2, 0),
            (1, 0), // no baseline
            (1, 0), // sparse component mask
            (3, 1),
            (6, 0),
            (10, 0x155),
            (3, 0), // entity terminator
            (1, 0), // empty control view
        ] {
            bits.extend((0..width).rev().map(|shift| value & (1 << shift) != 0));
        }
        let mut packet = vec![0_u8; bits.len().div_ceil(8)];
        for (bit, set) in bits.into_iter().enumerate() {
            if set {
                packet[bit / 8] |= 1 << (7 - bit % 8);
            }
        }
        let config = V41DecodeConfig::default();
        let layout = config.record_layout();
        let world = ReplicationDecodeState::default();
        assert_eq!(
            infer_unbound_archetype(
                &packet,
                0,
                17,
                &registry,
                &layout,
                &world,
                Some(&config.profile)
            ),
            None
        );
    }
}
