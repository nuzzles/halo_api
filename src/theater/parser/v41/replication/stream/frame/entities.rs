//! Reference default production frame policy: message/entity/control classes and world admission.
use crate::theater::parser::v41::context::layout::RecordLayout;
use crate::theater::parser::v41::context::profile::DecodeProfile;
use crate::theater::parser::v41::registry::source::FilmRegistry;
use crate::theater::parser::v41::replication::source::components::field::ComponentField;
use crate::theater::parser::v41::replication::source::{
    DeltaRejectionReason, EntityRecord, EntityViewStop, ExternalDeltaSchemaUse, FrameViewStop,
    RecordKind,
};
use crate::theater::parser::v41::replication::state::{DeltaAdmission, ReplicationDecodeState};
use crate::theater::parser::v41::replication::stream::frame::controls::{
    decode_control_view_with_limit, decode_message_view_signed,
};
use crate::theater::parser::v41::replication::stream::frame::header::decode_frame_header_signed;
use crate::theater::parser::v41::replication::stream::frame::records::{
    RecordBinding, RecordReadContext, decode_entity_record,
};
use crate::theater::parser::wire::fields::RawBits;

use crate::theater::parser::v41::replication::source::stream::models::production_frame::{
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
                        defaults.status == crate::theater::parser::v41::replication::source::DefaultStateStatus::Complete
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
