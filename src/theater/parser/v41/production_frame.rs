//! Native default production frame policy: message/entity/control classes and world admission.
use super::*;
pub(crate) use crate::theater::film::chunks::replication::production_frame::{
    ProductionAdmissionDiagnostics, ProductionEntityEnd, ProductionFrame,
};

impl ProductionAdmissionDiagnostics {
    fn observe(&mut self, admission: &FilmViewAdmission) {
        match admission {
            FilmViewAdmission::Unbound => self.rejected_unbound += 1,
            FilmViewAdmission::OtherView => self.rejected_other_view += 1,
            FilmViewAdmission::Anticipated(d) => {
                *self.anticipated_bindings.entry(d.archetype).or_default() += 1
            }
            FilmViewAdmission::Allowed => {}
        }
    }
}

pub(crate) struct ProductionReaderContext<'a> {
    pub reader: Option<&'a NativeReaderContext>,
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
        header_diagnostics: Default::default(),
        admission_diagnostics: Some(ProductionAdmissionDiagnostics::default()),
        record_prefixes: Vec::new(),
        messages: None,
        records: vec![],
        controls: None,
        entity_end: None,
        views_completed: 0,
        end_bit: start_bit,
        padded_bits: 0,
    };
    if start_bit == context.preamble_bits && context.preamble_bits >= 1 {
        let messages = decode_message_view_signed(data, context.preamble_bits - 1);
        out.end_bit = messages.end_bit;
        let complete = messages.stop == FrameViewStop::Complete;
        out.messages = Some(messages);
        if !complete {
            out.padded_bits =
                super::bits::padded_from_native(out.end_bit, data.len().saturating_mul(8));
            return Some(out);
        }
        out.views_completed += 1;
    }
    world.current_view = 0;
    let mut hit_end = false;
    let mut capture_slot = 0;
    for _ in 0..8192 {
        if out.end_bit >= data.len().saturating_mul(8) as i64 {
            break;
        }
        if encoding.extra_fields {
            out.record_prefixes.push(ComponentField {
                name: "record.prefix".into(),
                bit: out.end_bit,
                width: 32,
                // Native skips this prefix. Retaining its available bits must
                // not introduce a read/panic before a prefix-repaired header.
                raw: native_bits_tolerant(data, out.end_bit, 32),
            });
        }
        let header_bit = out
            .end_bit
            .wrapping_add(if encoding.extra_fields { 32 } else { 0 });
        let header = super::records::decode_frame_header_signed(data, header_bit, encoding)?;
        if header.kind == RecordKind::End {
            out.end_bit = header.end_bit;
            out.entity_end = Some(ProductionEntityEnd::Marker(header));
            hit_end = true;
            break;
        }
        let id = header.id?;
        if header.kind == RecordKind::Delta {
            let reason = world.admit_delta(id, true);
            out.admission_diagnostics.as_mut().unwrap().observe(&reason);
            if matches!(
                reason,
                FilmViewAdmission::Unbound | FilmViewAdmission::OtherView
            ) {
                out.end_bit = header.end_bit;
                out.entity_end = Some(ProductionEntityEnd::Rejected { header, reason });
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
        let Some(record) = super::components::decode_entity_record_with_capture_slots(
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
        out.end_bit = record.end_bit;
        observer(&record, capture_slot, world);
        let complete = record.stop == EntityViewStop::Complete;
        if complete {
            match record.header.kind {
                RecordKind::New => {
                    if let Some(refusal) =
                        world.bind_native_new(id, record.archetype?, record.header.start_bit)
                    {
                        out.header_diagnostics.new_binding_refusals.push(refusal);
                    }
                }
                RecordKind::Delete => world.unbind(id & 0x3fff_ffff),
                _ => {}
            }
        } else {
            out.entity_end = Some(ProductionEntityEnd::Failure(record.stop.clone()));
        }
        out.records.push(record);
        if !complete {
            break;
        }
    }
    if hit_end {
        out.views_completed += 1;
        let controls = components::decode_control_view_contextual(
            data,
            out.end_bit,
            encoding.position.as_ref(),
            context.reader,
        );
        out.end_bit = controls.end_bit;
        out.views_completed += usize::from(controls.stop == FrameViewStop::Complete);
        out.controls = Some(controls);
    } else if out.entity_end.is_none() {
        out.entity_end = Some(if out.records.len() == 8192 {
            ProductionEntityEnd::RecordLimit
        } else {
            ProductionEntityEnd::PayloadBoundary
        });
    }
    out.padded_bits = super::bits::padded_from_native(out.end_bit, data.len().saturating_mul(8));
    Some(out)
}
