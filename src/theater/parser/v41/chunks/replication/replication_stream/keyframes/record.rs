//! Full-state keyframe records use signed guards and all named components,
//! not the presence masks used by delta records.
use crate::theater::parser::v41::chunks::replication::source::replication_stream::models::keyframes::{
    KeyframeComponentRead, KeyframeRecord, KeyframeStop,
};
use crate::theater::parser::v41::chunks::replication::source::components::field::{ComponentField, RawBits};
use crate::theater::parser::v41::chunks::registry::source::{FilmRegistry};
use crate::theater::parser::v41::chunks::replication::components::cursor::ComponentCursor as Cursor;
use crate::theater::parser::v41::chunks::replication::components::defaults;
use crate::theater::parser::v41::chunks::replication::components::m4b::required_runtime_field;
use crate::theater::parser::v41::chunks::replication::components::reader::{
    ComponentReader, ComponentWidths,
};
use crate::theater::parser::v41::chunks::replication::components::widths::{
    ComponentWidthOverrides, read_component, signed_skip,
};
use crate::theater::parser::v41::chunks::replication::components::{m4b, vehicle};
use crate::theater::parser::v41::context::layout::RecordLayout;
use crate::theater::parser::v41::context::profile::DecodeProfile;
use crate::theater::parser::v41::reference_bits::{reference_bits_at, reference_bits_tolerant};

pub(crate) fn decode_reference_keyframe_record_contextual(
    data: &[u8],
    bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    context: Option<&DecodeProfile>,
) -> Option<KeyframeRecord> {
    decode_keyframe_record_inner(data, bit, registry, encoding, true, context)
}

/// Bounded native slot body; preserves the entire archetype word and never
/// adopts the historical record walker's six-bit/padded admission convention.
pub(crate) fn decode_keyframe_record_contextual(
    data: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    context: Option<&DecodeProfile>,
) -> Option<KeyframeRecord> {
    decode_keyframe_record_inner(data, bit, registry, encoding, false, context)
}

/// Two source-bounded diagnostic reads of one vehicle snapshot. Neither branch
/// is selected from successor-slot plausibility or published into Film.
pub(crate) fn vehicle_snapshot_branches(
    data: &[u8],
    canonical: &KeyframeRecord,
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    context: &DecodeProfile,
) -> Option<[KeyframeRecord; 2]> {
    let KeyframeStop::RuntimeContextUnavailable { index, field, .. } = &canonical.stop else {
        return None;
    };
    if canonical.archetype != 40
        || field != "vehicle+0x818"
        || encoding.snapshot_vehicle_type_gate.is_some()
    {
        return None;
    }
    let start = usize::try_from(canonical.start_bit).ok()?;
    let read = |gate| {
        let mut layout = encoding.clone();
        layout.snapshot_vehicle_type_gate = Some(gate);
        let branch =
            decode_keyframe_record_contextual(data, start, registry, &layout, Some(context))?;
        (branch.id == canonical.id
            && branch.archetype == canonical.archetype
            && branch.start_bit == canonical.start_bit
            && branch.end_bit >= canonical.end_bit
            && branch.fields == canonical.fields
            && branch.components.get(..*index) == canonical.components.get(..*index))
        .then_some(branch)
    };
    Some([read(false)?, read(true)?])
}

fn decode_keyframe_record_inner(
    data: &[u8],
    bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    record_layout: &RecordLayout,
    padded: bool,
    context: Option<&DecodeProfile>,
) -> Option<KeyframeRecord> {
    let component_widths = Some(&record_layout.component_widths);
    let simulation_complete = record_layout.keyframe_simulation_complete.unwrap_or(true);
    let layout = record_layout.keyframe_layout;
    let mpp_widths = record_layout.mpp_widths;
    let encoding = record_layout.position.as_ref();
    let corruption_check = record_layout.corruption_check;
    let bit = bit.try_into().ok()?;
    if context.is_none() && !layout.valid() {
        return None;
    }
    let header_bits = match context {
        Some(c) => c.keyframe.header_bits,
        None => i64::try_from(layout.header_bits).ok()?,
    };
    let size_word_bits = context.map_or(layout.size_word_bits as u64, |c| {
        c.keyframe.size_word_bits as u64
    });
    if mpp_widths.iter().any(|w| !(1..=32).contains(w)) || encoding.is_some_and(|e| !e.valid()) {
        return None;
    }
    let mut r = ComponentReader {
        category_references: record_layout.category_references,
        snapshot: true,
        references: Vec::new(),
        reference_widths: context.map(|c| ComponentWidths {
            movement: &c.movement,
            mpp: c.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,
        context_error: None,

        live_grammar: context.map(|c| c.grammar.clone()),
        // The reference full-state walker creates a fresh reader without setting a slot.
        movement_slot: Some(0),
        diagnostics: Default::default(),
        cursor: if padded {
            Cursor::signed(data, bit, None)
        } else {
            Cursor::new(data, usize::try_from(bit).ok()?)?
        },
        fields: Vec::new(),
        position_encoding: encoding,
    };
    let (id, archetype) = if padded {
        let archetype = reference_bits_at(data, bit.wrapping_add(58), 6) as u32;
        let id = reference_bits_tolerant(data, bit, 32) as u32;
        let mut remaining = header_bits.max(64);
        for (name, width) in [
            ("header.id", 32),
            ("header.archetype", 32),
            ("header.word", 32),
            ("header.kind", 4),
            ("header.byte", 8),
        ] {
            let width = width.min(remaining);
            if width > 0 {
                r.fields.push(ComponentField {
                    name: name.into(),
                    bit: usize::try_from(r.cursor.position).ok()?,
                    width: usize::try_from(width).ok()?,
                    raw: RawBits::from_source(
                        data,
                        usize::try_from(r.cursor.position).ok()?,
                        usize::try_from(width).ok()?,
                    )?,
                });
                r.cursor.skip_signed(width);
                remaining -= width;
            }
        }
        if remaining > 0 {
            signed_skip(
                &mut r,
                "header.extension",
                remaining,
                false,
                None,
                ("header.extension", "header.extension.tail"),
            )?;
        }
        if header_bits < 64 {
            signed_skip(
                &mut r,
                "header.layout_adjustment",
                header_bits.wrapping_sub(64),
                false,
                None,
                ("header.layout_adjustment", "header.layout_adjustment"),
            )?;
        }
        (id, archetype)
    } else {
        let id = r.r("header.id", 32)? as u32;
        let raw_archetype = r.r("header.archetype", 32)? as u32;
        // The native table already classified the whole archetype word.
        let archetype = raw_archetype;
        let mut remaining = usize::try_from(header_bits).ok()?.checked_sub(64)?;
        for (name, width) in [("header.word", 32), ("header.kind", 4), ("header.byte", 8)] {
            let width = width.min(remaining);
            if width > 0 {
                r.r(name, width)?;
                remaining -= width;
            }
        }
        r.words("header.extension", remaining / 64, 64)?;
        if !remaining.is_multiple_of(64) {
            r.r("header.extension.tail", remaining % 64)?;
        }
        (id, archetype)
    };
    let mut components = Vec::new();
    let mut default_state = None;
    let mut stop = body(
        &mut r,
        archetype,
        registry,
        mpp_widths,
        corruption_check,
        (&mut components, &mut default_state),
        (
            padded,
            component_widths,
            simulation_complete,
            size_word_bits,
            record_layout.snapshot_vehicle_type_gate,
        ),
    )
    .unwrap_or(KeyframeStop::Truncated);
    let source_end = i64::try_from(data.len().saturating_mul(8)).ok()?;
    if r.cursor.position > source_end {
        stop = KeyframeStop::Truncated;
    }
    components.retain_mut(|component| {
        if component.start_bit >= source_end {
            return false;
        }
        component.end_bit = component.end_bit.min(source_end);
        true
    });
    let mut fields = r.fields;
    fields.retain(|field| {
        !components.iter().any(|component: &KeyframeComponentRead| {
            let Ok(start) = usize::try_from(component.start_bit) else {
                return false;
            };
            let Ok(end) = usize::try_from(component.end_bit) else {
                return false;
            };
            field.bit >= start && field.bit < end
        })
    });
    Some(KeyframeRecord {
        start_bit: bit,
        end_bit: r.cursor.position.min(source_end),
        id,
        archetype,
        fields,
        default_state,
        external_vehicle_type_gate_used: record_layout.snapshot_vehicle_type_gate.filter(|_| {
            components
                .iter()
                .any(|c| required_runtime_field(&c.name).is_some())
        }),
        components,
        stop,
    })
}

fn body(
    r: &mut ComponentReader<'_>,
    ti: u32,
    registry: &FilmRegistry,
    widths: [usize; 2],
    check: Option<bool>,
    sections: (
        &mut Vec<KeyframeComponentRead>,
        &mut Option<crate::theater::parser::v41::chunks::replication::source::DefaultState>,
    ),
    policy: (
        bool,
        Option<&ComponentWidthOverrides>,
        bool,
        u64,
        Option<bool>,
    ),
) -> Option<KeyframeStop> {
    let (components, default_state) = sections;
    let (_reference_policy, component_widths, simulation_complete, size_word_bits, vehicle_gate) =
        policy;
    // The writer omits n1, n2 and the entire body for this sentinel archetype.
    if ti == u32::MAX {
        return Some(KeyframeStop::Complete);
    }
    let Some(arch) = registry.archetype(ti as usize).filter(|_| ti < 50) else {
        return Some(KeyframeStop::InvalidArchetype);
    };
    let n1 = r.r_wide("default_guard", size_word_bits)? as u32 as i32;
    if n1 > 0 {
        let start = r.cursor.position;
        let field_start = r.fields.len();
        let has_reader = defaults::has_reference_deserializer(ti);
        let result = (|| {
            if has_reader && !defaults::state(r, ti, widths)? {
                return Some(false);
            }
            if check == Some(true) {
                r.r_wide("default_corruption_check", size_word_bits)?;
            }
            Some(true)
        })();
        *default_state = Some(crate::theater::parser::v41::chunks::replication::source::DefaultState {
            source: crate::theater::parser::packet::BitRange {
                start: usize::try_from(start).ok()?,
                end: usize::try_from(r.cursor.position).ok()?,
            },
            fields: r.fields.drain(field_start..).collect(),
            status: match result {
                Some(true) if !has_reader => crate::theater::parser::v41::chunks::replication::source::DefaultStateStatus::Opaque,
                Some(true) => crate::theater::parser::v41::chunks::replication::source::DefaultStateStatus::Complete,
                Some(false) => crate::theater::parser::v41::chunks::replication::source::DefaultStateStatus::Unsupported,
                None => crate::theater::parser::v41::chunks::replication::source::DefaultStateStatus::Truncated,
            },
        });
        if r.reference_configuration_unavailable() {
            default_state.as_mut().unwrap().status =
                crate::theater::parser::v41::chunks::replication::source::DefaultStateStatus::ContextUnavailable;
            return Some(KeyframeStop::UnsupportedReferenceConfiguration);
        }
        if !result? {
            return Some(KeyframeStop::UnsupportedDefault);
        }
        if check.is_none() {
            default_state.as_mut().unwrap().status =
                crate::theater::parser::v41::chunks::replication::source::DefaultStateStatus::ContextUnavailable;
            return Some(KeyframeStop::MissingCorruptionCheckContext);
        }
    }
    let n2 = r.r_wide("components_guard", size_word_bits)? as u32 as i32;
    if n2 <= 0 {
        return Some(KeyframeStop::Complete);
    }
    for (index, registry_component) in arch.components.iter().enumerate() {
        let name = registry_component.name()?;
        let start_bit = r.cursor.position;
        let field_start = r.fields.len();
        let status = match (ti, name, vehicle_gate) {
            (40, "vehicle-type-state-component", Some(gate)) => {
                vehicle::snapshot_type_state(r, gate)
            }
            (40, "vehicle-type-physics-component", Some(gate)) => {
                m4b::snapshot_type_physics(r, gate)
            }
            _ => read_component(
                r,
                name,
                registry_component.precision_level,
                ti,
                component_widths,
                (simulation_complete, None),
            ),
        };
        components.push(KeyframeComponentRead {
            status: status.into(),
            index,
            name: name.to_owned(),
            start_bit,
            end_bit: r.cursor.position,
            fields: r.fields[field_start..].to_vec(),
        });
        if r.reference_configuration_unavailable() {
            components.last_mut().unwrap().status =
                crate::theater::parser::v41::chunks::replication::source::ComponentReadStatus::ContextUnavailable;
            return Some(KeyframeStop::UnsupportedReferenceConfiguration);
        }
        if let Some(field) = r.runtime_context_unavailable() {
            components.last_mut().unwrap().status =
                crate::theater::parser::v41::chunks::replication::source::ComponentReadStatus::ContextUnavailable;
            return Some(KeyframeStop::RuntimeContextUnavailable {
                index,
                name: name.to_owned(),
                field: field.to_owned(),
            });
        }
        if !status? {
            if let Some(field) = required_runtime_field(name) {
                return Some(KeyframeStop::RuntimeContextUnavailable {
                    index,
                    name: name.to_owned(),
                    field: field.to_owned(),
                });
            }
            return Some(KeyframeStop::UnsupportedComponent {
                index,
                name: name.to_owned(),
            });
        }
        let Some(check) = check else {
            components.last_mut().unwrap().status =
                crate::theater::parser::v41::chunks::replication::source::ComponentReadStatus::ContextUnavailable;
            return Some(KeyframeStop::MissingCorruptionCheckContext);
        };
        let check = if check {
            r.gate("component_corruption_check", 32, true)
        } else {
            Some(())
        };
        let component = components.last_mut().unwrap();
        component.end_bit = r.cursor.position;
        component.fields = r.fields[field_start..].to_vec();
        if check.is_none() {
            component.status = crate::theater::parser::v41::chunks::replication::source::ComponentReadStatus::Truncated;
        }
        check?;
    }
    Some(KeyframeStop::Complete)
}

#[cfg(test)]
mod vehicle_gate_tests {
    use super::*;
    use crate::theater::parser::test_support::component_registry;
    use crate::theater::parser::v41::chunks::replication::source::KeyframeStop;
    use crate::theater::parser::v41::context::V41DecodeConfig;

    fn bits(gate: bool, raw_physics: bool) -> Vec<u8> {
        let mut stream = Vec::new();
        let mut put = |width: usize, value: u64| {
            stream.extend((0..width).rev().map(|shift| value & (1 << shift) != 0));
        };
        put(32, 0x4000_0300); // full vehicle ID
        put(32, 40);
        put(32, 0);
        put(4, 3);
        put(8, 0);
        put(32, 0); // no creation/default body
        put(32, 1); // component state present
        if gate {
            put(2, 1); // type state
            put(6, 0x15);
            put(1, u64::from(raw_physics));
            if raw_physics {
                for word in 1..=9 {
                    put(32, word);
                }
            } else {
                put(1, 1); // default forward vector
                put(8, 0); // roll
                put(1, 1); // zero angular velocity
            }
        }
        let mut data = vec![0; stream.len().div_ceil(8)];
        for (i, bit) in stream.into_iter().enumerate() {
            data[i / 8] |= u8::from(bit) << (7 - i % 8);
        }
        data
    }

    #[test]
    fn explicit_vehicle_construction_gate_reads_both_snapshot_branches() {
        let registry = component_registry(&[(
            40,
            vec![
                ("vehicle-type-state-component".into(), 0),
                ("vehicle-type-physics-component".into(), 0),
            ],
        )]);
        let mut config = V41DecodeConfig::default();
        config.profile.grammar.corruption_check = false;
        let mut layout = config.record_layout();
        let unavailable = decode_keyframe_record_contextual(
            &bits(true, true),
            0,
            &registry,
            &layout,
            Some(&config.profile),
        )
        .unwrap();
        assert_eq!(unavailable.end_bit, 172);
        assert!(matches!(
            unavailable.stop,
            KeyframeStop::RuntimeContextUnavailable { ref field, .. } if field == "vehicle+0x818"
        ));
        let branches = vehicle_snapshot_branches(
            &bits(true, true),
            &unavailable,
            &registry,
            &layout,
            &config.profile,
        )
        .unwrap();
        assert_eq!(branches[0].end_bit, 172);
        assert_eq!(branches[1].end_bit, 469);
        assert_eq!(branches[0].stop, KeyframeStop::Complete);
        assert_eq!(branches[1].stop, KeyframeStop::Complete);
        layout.snapshot_vehicle_type_gate = Some(false);
        let omitted = decode_keyframe_record_contextual(
            &bits(false, true),
            0,
            &registry,
            &layout,
            Some(&config.profile),
        )
        .unwrap();
        assert_eq!(omitted.stop, KeyframeStop::Complete);
        assert_eq!(omitted.end_bit, 172);
        assert!(omitted.components.iter().all(|c| c.fields.is_empty()));
        layout.snapshot_vehicle_type_gate = Some(true);
        let present = decode_keyframe_record_contextual(
            &bits(true, true),
            0,
            &registry,
            &layout,
            Some(&config.profile),
        )
        .unwrap();
        assert_eq!(present.stop, KeyframeStop::Complete);
        assert_eq!(present.end_bit, 469);
        assert_eq!(present.components[0].start_bit, 172);
        assert_eq!(present.components[0].end_bit, 180);
        assert_eq!(present.components[1].start_bit, 180);
        assert_eq!(present.components[1].end_bit, 469);
        assert_eq!(present.components[1].fields.len(), 10);
        let source = bits(true, true);
        for component in &present.components {
            for field in &component.fields {
                assert_eq!(
                    field.raw,
                    RawBits::from_source(&source, field.bit, field.width).unwrap()
                );
            }
        }
        let truncated = decode_keyframe_record_contextual(
            &source[..source.len() - 1],
            0,
            &registry,
            &layout,
            Some(&config.profile),
        )
        .unwrap();
        assert_eq!(truncated.stop, KeyframeStop::Truncated);
        assert_eq!(
            truncated.components[0].status,
            crate::theater::parser::v41::chunks::replication::source::ComponentReadStatus::Complete
        );
    }

    #[test]
    fn compact_vehicle_physics_matches_upstream_snapshot_width() {
        // Upstream components::tests::vehicle_type_components_read_when_listed
        // consumes 19 bits for state 1 and this non-raw physics body.
        let registry = component_registry(&[(
            40,
            vec![
                ("vehicle-type-state-component".into(), 0),
                ("vehicle-type-physics-component".into(), 0),
            ],
        )]);
        let mut config = V41DecodeConfig::default();
        config.profile.grammar.corruption_check = false;
        let mut layout = config.record_layout();
        layout.snapshot_vehicle_type_gate = Some(true);
        let source = bits(true, false);
        let read = decode_keyframe_record_contextual(
            &source,
            0,
            &registry,
            &layout,
            Some(&config.profile),
        )
        .unwrap();
        assert_eq!(read.stop, KeyframeStop::Complete);
        assert_eq!(read.end_bit, 172 + 19);
        assert_eq!(read.components[0].end_bit, 180);
        assert_eq!(read.components[1].end_bit, 191);
        for component in &read.components {
            for field in &component.fields {
                assert_eq!(
                    field.raw,
                    RawBits::from_source(&source, field.bit, field.width).unwrap()
                );
            }
        }
    }
}
