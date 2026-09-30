//! Reference default production frame policy: message/entity/control classes and world admission.
use super::*;
pub(crate) use crate::theater::film::chunks::replication::replication_stream::models::production_frame::{
    ProductionEntityEnd, ProductionFrame,
};

pub(crate) struct ProductionReaderContext<'a> {
    pub reader: Option<&'a ReaderContext>,
    pub preamble_bits: i64,
}

pub(crate) fn decode_production_frame_contextual(
    data: &[u8],
    start_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &mut FilmWorld,
    context: ProductionReaderContext<'_>,
    mut observer: impl FnMut(&EntityRecord, u32, &FilmWorld),
) -> Option<ProductionFrame> {
    let start_bit = start_bit.try_into().ok()?;
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
        let Some(mut header) =
            super::records::decode_frame_header_signed(data, header_bit, encoding)
        else {
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
            let reason = world.admit_delta(id, true);
            if matches!(
                reason,
                FilmViewAdmission::Unbound | FilmViewAdmission::OtherView
            ) {
                out.end_bit = header.end_bit;
                out.entity_end = Some(ProductionEntityEnd::Rejected { header });
                hit_end = true;
                break;
            }
        }
        if header.kind == RecordKind::Delta {
            capture_slot = id & 0x3fff_ffff;
        }
        let mut bindings = EntityBindings::default();
        if let Some(ti) = world.archetype(id & 0x3fff_ffff) {
            bindings.bind(id, ti);
        }
        let Some(mut record) = super::components::decode_entity_record_with_capture_slots(
            data,
            header_bit,
            registry,
            &body_encoding,
            &bindings,
            context
                .reader
                .is_none_or(|c| c.profile.grammar.simulation_complete),
            super::components::RecordCaptureSlots {
                context: context.reader.cloned(),
                movement: Some(capture_slot),
                position: Some(capture_slot),
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
                    .filter(|ti| *ti < 50 && registry.archetype(*ti as usize).is_some())
                {
                    let _ = world.bind_reference_new(id, archetype, record.header.start_bit);
                }
            }
            // DELETE's recorded header ends this parser declaration even if
            // its following metadata word is truncated. Keep that truncation
            // on the record rather than retaining a stale decoding schema.
            RecordKind::Delete => world.unbind(id & 0x3fff_ffff),
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
        let controls = components::decode_control_view_contextual(
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
