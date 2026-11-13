//! Reference default production frame policy: message/entity/control classes and world admission.
use crate::theater::parser::v41::chunks::registry::source::FilmRegistry;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::controls::{
    decode_control_view_with_limit, decode_message_view_signed,
};
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::header::decode_frame_header_signed;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::records::{
    RecordBinding, RecordReadContext, decode_entity_record,
};
use crate::theater::parser::v41::chunks::replication::source::components::field::{
    ComponentField, RawBits,
};
use crate::theater::parser::v41::chunks::replication::source::{
    DeltaRejectionReason, EntityRecord, EntityViewStop, ExternalDeltaSchemaUse, FrameViewStop,
    RecordKind,
};
use crate::theater::parser::v41::chunks::replication::state::{
    DeltaAdmission, ReplicationDecodeState,
};
use crate::theater::parser::v41::context::layout::RecordLayout;
use crate::theater::parser::v41::context::profile::DecodeProfile;

use crate::theater::parser::v41::chunks::replication::source::replication_stream::models::production_frame::{
    ProductionEntityEnd, ProductionFrame,
};

/// Optional contextual recovery supplied by runtime; canonical parsing supplies none.
pub(crate) type SchemaRecoveryReader = fn(SchemaRecoveryRequest<'_>) -> Option<RecoveredSchema>;

pub(crate) struct SchemaRecoveryRequest<'a> {
    pub data: &'a [u8],
    pub start_bit: i64,
    pub id: u32,
    pub registry: &'a FilmRegistry,
    pub encoding: &'a RecordLayout,
    pub world: &'a ReplicationDecodeState,
    pub context: Option<&'a DecodeProfile>,
}

/// Hypothesis and corroborating successor extents, never a recorded declaration.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecoveredSchema {
    pub archetype: u32,
    pub end_bit: i64,
    pub successor_id: u32,
    pub successor_start_bit: i64,
    pub successor_end_bit: i64,
}

#[derive(Clone, Copy)]
pub(crate) struct ProductionReaderContext<'a> {
    pub reader: Option<&'a DecodeProfile>,
    pub schema_recovery: Option<SchemaRecoveryReader>,
    pub preamble_bits: i64,
    pub control_record_limit: usize,
    pub explicit_delta_schema: Option<&'a dyn Fn(i64, u32) -> Option<u32>>,
    /// Only the historical reference traversal reads controls after rejection.
    pub reference_controls_after_rejection: bool,
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
        external_delta_schemas: Vec::new(),
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
        let explicit_archetype = (header.kind == RecordKind::Delta)
            .then(|| {
                context
                    .explicit_delta_schema
                    .and_then(|select| select(header_bit, id))
            })
            .flatten()
            .filter(|archetype| {
                *archetype < 50 && registry.archetype(*archetype as usize).is_some()
            });
        if header.kind == RecordKind::Delta && explicit_archetype.is_none() {
            // A missed NEW may have reused this slot with another archetype.
            // Contextual replay may test that hypothesis against a complete
            // successor; the canonical reader has no recovery hook and keeps
            // its ordinary source walk unchanged.
            let generation_changed = world
                .slots
                .get(&(id & 0x3fff_ffff))
                .is_some_and(|binding| !binding.generation_any && binding.full_id != id);
            let mut reason = world.admit_delta(id, true);
            if (matches!(reason, DeltaAdmission::Unbound) || generation_changed)
                && let Some(recover) = context.schema_recovery
                && let Some(recovered) = recover(SchemaRecoveryRequest {
                    data,
                    start_bit: header_bit,
                    id,
                    registry,
                    encoding: &body_encoding,
                    world,
                    context: context.reader,
                })
            {
                world.bind_inferred(id, recovered.archetype);
                world.schema_recoveries.push((id, header_bit, recovered));
                reason = world.admit_delta(id, true);
            }
            if matches!(reason, DeltaAdmission::Unbound | DeltaAdmission::OtherView) {
                out.end_bit = header.end_bit;
                out.entity_end = Some(ProductionEntityEnd::Rejected {
                    header,
                    reason: match reason {
                        DeltaAdmission::Unbound => DeltaRejectionReason::Unbound,
                        DeltaAdmission::OtherView => DeltaRejectionReason::OtherView,
                        _ => unreachable!("only refused admissions reach this branch"),
                    },
                });
                // Rejection gives no boundary for the unread entity body.
                // Canonical controls require the recorded entity terminator.
                hit_end = context.reference_controls_after_rejection;
                break;
            }
        }
        if header.kind == RecordKind::Delta {
            capture_slot = id & 0x3fff_ffff;
        }
        let binding = explicit_archetype
            .or_else(|| world.archetype(id & 0x3fff_ffff))
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
        if let Some(archetype) = explicit_archetype {
            out.external_delta_schemas.push(ExternalDeltaSchemaUse {
                start_bit: header_bit,
                id,
                archetype,
            });
        }
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
                    EntityViewStop::Complete
                        | EntityViewStop::Truncated
                        | EntityViewStop::MissingCorruptionCheckContext
                        | EntityViewStop::UnsupportedReferenceConfiguration
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
                        defaults.status == crate::theater::parser::v41::chunks::replication::source::DefaultStateStatus::Complete
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
        let controls = decode_control_view_with_limit(
            data,
            out.end_bit,
            encoding.position.as_ref(),
            context.reader,
            encoding.category_references,
            context.control_record_limit,
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
pub(crate) fn mask_matches_schema(
    record: &EntityRecord,
    archetype: u32,
    registry: &FilmRegistry,
) -> bool {
    registry
        .archetype(archetype as usize)
        .is_some_and(|schema| {
            record.mask.is_none_or(|mask| {
                schema.components.len() >= u64::BITS as usize
                    || mask >> schema.components.len() == 0
            })
        })
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
    use crate::theater::parser::packet::ByteRange;
    use crate::theater::parser::v41::chunks::registry::source::{FilmArchetype, RegistryComponent};
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
            if has_component {
                put(1, 1); // NEW component gate
                put(1, 0); // sparse mask
                put(3, 1);
                put(6, 0);
                put(1, 1);
            } else {
                put(1, 0); // no initial components or mask
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
                    schema_recovery: None,
                    reader: Some(&config.profile),
                    preamble_bits: 2,
                    control_record_limit: config.control_record_limit,
                    explicit_delta_schema: None,
                    reference_controls_after_rejection: false,
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
        let mut payload = include_bytes!(
            "../../../../../../fixtures/movement-schema-v41/invalid-flock-declaration.bin"
        )
        .to_vec();
        // The captured walk is misaligned and has a clear NEW gate at bit 1103.
        // Set that one bit in a derived fixture to exercise the out-of-schema
        // mask guard under the corrected creation grammar.
        payload[1103 / 8] |= 1 << (7 - 1103 % 8);
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
            &payload,
            1061,
            &registry,
            &config.record_layout(),
            &mut world,
            ProductionReaderContext {
                schema_recovery: None,
                reader: Some(&config.profile),
                preamble_bits: 2,
                control_record_limit: config.control_record_limit,
                explicit_delta_schema: None,
                reference_controls_after_rejection: false,
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

    fn live_fire_region_fixture() -> (FilmRegistry, V41DecodeConfig) {
        let declarations: Vec<(usize, Vec<(String, u32)>)> = serde_json::from_str(include_str!(
            "../../../../../../fixtures/live-fire-region-3/registry.json"
        ))
        .unwrap();
        let registry = crate::theater::parser::test_support::component_registry(&declarations);
        let mut config = V41DecodeConfig::default();
        config.profile.movement.world_object.axis_bits = [12, 12, 11];
        config.profile.movement.world_object.index_bits = 2;
        config.profile.movement.world_object.region = 1;
        let map: crate::theater::film::catalog::maps::MapDetails =
            crate::theater::film::catalog::maps::known_maps::LIVE_FIRE.into();
        config.profile.movement.opaque_position_region_bits = map.opaque_position_region_bits;
        (registry, config)
    }

    #[test]
    fn live_fire_region_three_preserves_the_packet_boundary_without_a_team_update() {
        let payload =
            include_bytes!("../../../../../../fixtures/live-fire-region-3/false-team-packet.bin");
        let (registry, config) = live_fire_region_fixture();
        let mut world = ReplicationDecodeState::default();
        for (slot, archetype) in [
            (123, 4),
            (558, 35),
            (559, 35),
            (562, 35),
            (566, 35),
            (567, 35),
            (568, 35),
            (569, 35),
            (769, 40),
            (1962, 37),
            (1965, 42),
            (1967, 37),
            (1969, 37),
            (5365, 9),
        ] {
            world.bind_full((1 << 30) | slot, archetype);
        }
        let original = world.clone();
        let frame = decode_production_frame_contextual(
            payload,
            2,
            &registry,
            &config.record_layout(),
            &mut world,
            ProductionReaderContext {
                schema_recovery: None,
                reader: Some(&config.profile),
                preamble_bits: 2,
                control_record_limit: config.control_record_limit,
                explicit_delta_schema: None,
                reference_controls_after_rejection: false,
            },
            |_, _, _| {},
        )
        .unwrap();
        assert!(super::super::production_frame_closed(&frame, payload));
        assert_eq!(frame.end_bit, 1891);
        assert_eq!(frame.records.len(), 13);
        assert!(
            frame
                .records
                .iter()
                .all(|r| r.header.id != Some(1073747189))
        );
        let vehicle = &frame.records[8];
        assert_eq!(vehicle.end_bit, 1242);
        let position = &vehicle.components[0];
        let raw = position
            .fields
            .iter()
            .find(|f| f.name == "position.opaque_region_bits")
            .unwrap();
        assert_eq!((raw.bit, raw.width), (1145, 54));
        assert!(
            !position
                .fields
                .iter()
                .any(|f| f.name.starts_with("position["))
        );
        assert_eq!(
            vehicle.components[2]
                .fields
                .iter()
                .find(|f| f.name == "tick")
                .unwrap()
                .raw
                .low_u64(),
            39
        );
        // The old world-region fallback manufactured the team record even with
        // the correct managed-player binding. No runtime team guard is involved.
        let mut legacy = config.clone();
        legacy.profile.movement.opaque_position_region_bits.clear();
        let mut world = original;
        let old = decode_production_frame_contextual(
            payload,
            2,
            &registry,
            &legacy.record_layout(),
            &mut world,
            ProductionReaderContext {
                schema_recovery: None,
                reader: Some(&legacy.profile),
                preamble_bits: 2,
                control_record_limit: legacy.control_record_limit,
                explicit_delta_schema: None,
                reference_controls_after_rejection: false,
            },
            |_, _, _| {},
        )
        .unwrap();
        assert!(!super::super::production_frame_closed(&old, payload));
        let team = old
            .records
            .iter()
            .find(|r| r.header.id == Some(1073747189))
            .unwrap();
        assert_eq!(team.components[0].fields[0].raw.low_u64(), 6);
    }

    #[test]
    fn live_fire_region_three_keeps_command_ticks_aligned_across_component_masks() {
        #[derive(serde::Deserialize)]
        struct Sample {
            file: String,
            start_bit: i64,
            counter: u64,
        }
        let samples: Vec<Sample> = serde_json::from_str(include_str!(
            "../../../../../../fixtures/live-fire-region-3/samples.json"
        ))
        .unwrap();
        let (registry, config) = live_fire_region_fixture();
        for sample in samples {
            let payload = std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/theater/fixtures/live-fire-region-3")
                    .join(&sample.file),
            )
            .unwrap();
            let record = decode_entity_record(
                &payload,
                sample.start_bit,
                &registry,
                &config.record_layout(),
                Some(RecordBinding {
                    id: 1073742593,
                    archetype: 40,
                }),
                false,
                RecordReadContext {
                    context: Some(&config.profile),
                    movement: None,
                },
            )
            .unwrap();
            assert_eq!(record.stop, EntityViewStop::Complete, "{}", sample.file);
            let tick = record
                .components
                .iter()
                .flat_map(|c| &c.fields)
                .find(|f| f.name == "tick")
                .unwrap();
            assert_eq!(tick.raw.low_u64(), sample.counter, "{}", sample.file);
        }
    }
}
