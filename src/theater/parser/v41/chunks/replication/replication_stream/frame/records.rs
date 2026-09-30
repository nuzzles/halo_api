//! Sequential replication views and generation-aware entity bindings.
use crate::theater::film::chunks::replication::replication_stream::models::entity_records::{
    EntityComponentRead, EntityRecord, EntityViewStop,
};
use crate::theater::film::{FilmRegistry, RecordKind};
use crate::theater::parser::v41::chunks::replication::components::cursor::ComponentCursor as Cursor;
use crate::theater::parser::v41::chunks::replication::components::defaults;
use crate::theater::parser::v41::chunks::replication::components::m4b::required_runtime_field;
use crate::theater::parser::v41::chunks::replication::components::reader::{
    ComponentReader, ComponentWidths,
};
use crate::theater::parser::v41::chunks::replication::components::widths::{
    read_component, signed_skip,
};
use crate::theater::parser::v41::context::layout::RecordLayout;
use crate::theater::parser::v41::context::profile::DecodeProfile;
use crate::theater::parser::v41::reference::diagnostics::WidthPurpose;

/// Schema selected by the enclosing chronological traversal for this record.
#[derive(Clone, Copy)]
pub(crate) struct RecordBinding {
    pub id: u32,
    pub archetype: u32,
}

/// Borrowed schema and inherited slots needed to read a single wire record.
pub(crate) struct RecordReadContext<'a> {
    pub context: Option<&'a DecodeProfile>,
    pub movement: Option<u32>,
}

pub(crate) fn decode_entity_record(
    data: &[u8],
    bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    binding: Option<RecordBinding>,
    simulation_complete: bool,
    slots: RecordReadContext<'_>,
) -> Option<EntityRecord> {
    let bit = bit.try_into().ok()?;
    if !encoding.valid() {
        return None;
    }
    let mut r = ComponentReader {
        references: Vec::new(),
        reference_widths: slots.context.as_ref().map(|c| ComponentWidths {
            movement: &c.movement,
            mpp: c.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,

        live_grammar: slots.context.as_ref().map(|c| c.grammar.clone()),
        movement_slot: slots.movement,
        diagnostics: Default::default(),
        cursor: Cursor::signed(data, bit, None),
        fields: vec![],
        position_encoding: encoding.position.as_ref(),
    };
    read_entity_record(
        &mut r,
        data,
        registry,
        encoding,
        binding,
        simulation_complete,
    )
}
fn read_entity_record(
    r: &mut ComponentReader<'_>,
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    binding: Option<RecordBinding>,
    simulation_complete: bool,
) -> Option<EntityRecord> {
    r.fields.clear();
    if encoding.extra_fields {
        // Reference skips this prefix without reading it. Retain source fields
        // where addressable, but do not turn a negative skip into a read panic.
        if r.cursor.position < 0 {
            signed_skip(
                r,
                "record.prefix",
                32,
                false,
                Some(WidthPurpose::RecordPrefix),
                ("record.prefix", "record.prefix"),
            )?;
        } else {
            r.r("record.prefix", 32)?;
        }
    }
    let width = encoding
        .reference_id_low_bits
        .or_else(|| i64::try_from(encoding.ids.low_bits).ok())?;
    if !r.cursor.fits_source(width) {
        return None;
    }
    let header = r.cursor.reference_header(width, encoding.ids.base);
    r.cursor.position = header.end_bit;
    let mut rec = EntityRecord {
        header,
        archetype: None,
        mask: None,
        fields: Vec::new(),
        default_state: None,
        components: Vec::new(),
        end_bit: r.cursor.position,
        stop: EntityViewStop::Truncated,
    };
    rec.stop = record_body(
        r,
        &mut rec,
        registry,
        encoding,
        binding,
        simulation_complete,
    )
    .unwrap_or(EntityViewStop::Truncated);
    let source_end = i64::try_from(data.len().saturating_mul(8)).ok()?;
    if r.cursor.position > source_end {
        rec.stop = EntityViewStop::Truncated;
    }
    rec.end_bit = r.cursor.position.min(source_end);
    rec.components.retain_mut(|component| {
        if component.start_bit >= source_end {
            return false;
        }
        component.end_bit = component.end_bit.min(source_end);
        true
    });
    rec.fields = std::mem::take(&mut r.fields);
    rec.fields.retain(|field| {
        !rec.components.iter().any(|component| {
            let Ok(start) = usize::try_from(component.start_bit) else {
                return false;
            };
            let Ok(end) = usize::try_from(component.end_bit) else {
                return false;
            };
            field.bit >= start && field.bit < end
        })
    });
    Some(rec)
}

fn record_body(
    r: &mut ComponentReader<'_>,
    rec: &mut EntityRecord,
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    binding: Option<RecordBinding>,
    simulation_complete: bool,
) -> Option<EntityViewStop> {
    if rec.header.kind == RecordKind::End {
        return Some(EntityViewStop::Complete);
    }
    let id = rec.header.id?;
    match rec.header.kind {
        RecordKind::New | RecordKind::Delete if encoding.extra_fields => {
            r.gate("record.extra", 8, true)?
        }
        _ => {}
    }
    if rec.header.kind == RecordKind::Delete {
        r.r("delete.word", 32)?;
        return Some(EntityViewStop::Complete);
    }
    let ti = if rec.header.kind == RecordKind::New {
        r.r("archetype", 6)? as u32
    } else {
        let Some(binding) = binding.filter(|binding| binding.id & 0x3fff_ffff == id & 0x3fff_ffff)
        else {
            return Some(EntityViewStop::MissingBinding { id });
        };
        if binding.id != id {
            return Some(EntityViewStop::GenerationMismatch {
                id,
                bound_id: binding.id,
            });
        }
        r.gate("baseline", 7, true)?;
        binding.archetype
    };
    rec.archetype = (rec.header.kind == RecordKind::New).then_some(ti);
    let arch = registry.archetype(ti as usize).filter(|_| ti < 50);
    if ti >= 50 || (arch.is_none() && rec.header.kind != RecordKind::New) {
        return Some(EntityViewStop::InvalidArchetype { archetype: ti });
    }
    if rec.header.kind == RecordKind::New {
        let skip_default = ti != 35
            && (!encoding.new_record.deserialize_defaults
                || !(ti == 41 || defaults::has_reference_deserializer(ti)));
        let default_start = r.cursor.position;
        let field_start = r.fields.len();
        let result = (|| {
            if skip_default {
                if let Some(width) = encoding.new_record.reference_fallback_default_bits {
                    read_reference_new_skip(
                        r,
                        width,
                        "new.default_skipped",
                        "new default state",
                        WidthPurpose::NewRecordDefault,
                    )?;
                } else {
                    read_new_raw_bits(
                        r,
                        "new.default_skipped",
                        encoding.new_record.fallback_default_bits,
                    )?;
                }
                Some(true)
            } else if ti == 41 {
                defaults::projectile(r, encoding.mpp_widths, true)
            } else {
                defaults::state(r, ti, encoding.mpp_widths)
            }
        })();
        rec.default_state = Some(crate::theater::film::DefaultState {
            source: crate::theater::film::BitRange {
                start: usize::try_from(default_start).ok()?,
                end: usize::try_from(r.cursor.position).ok()?,
            },
            fields: r.fields.drain(field_start..).collect(),
            status: match result {
                Some(true) if skip_default => crate::theater::film::DefaultStateStatus::Opaque,
                Some(true) => crate::theater::film::DefaultStateStatus::Complete,
                Some(false) => crate::theater::film::DefaultStateStatus::Unsupported,
                None => crate::theater::film::DefaultStateStatus::Truncated,
            },
        });
        if !result? {
            return Some(EntityViewStop::UnsupportedDefault { archetype: ti });
        }
        r.bit("new.component_gate")?;
    }
    let mask = r.mask()?;
    rec.mask = Some(mask);
    // Reference NEW reads its default state, gate and mask before registry lookup.
    let Some(arch) = arch else {
        return Some(EntityViewStop::InvalidArchetype { archetype: ti });
    };
    // The reference registry loop ignores mask bits beyond its component list.
    let component_count = arch.components.len();
    for index in 0..component_count {
        if mask & (1 << (index & 63)) == 0 {
            continue;
        }
        let Some(registry_component) = arch.components.get(index) else {
            return Some(EntityViewStop::InvalidComponent { index });
        };
        let name = registry_component.name()?;
        let start_bit = r.cursor.position;
        let field_start = r.fields.len();
        let status = read_component(
            r,
            name,
            registry_component.precision_level,
            ti,
            Some(&encoding.component_widths),
            (simulation_complete, None),
        );
        let mut component = EntityComponentRead {
            index,
            name: name.to_owned(),
            start_bit,
            end_bit: r.cursor.position,
            status: status.into(),
            fields: r.fields[field_start..].to_vec(),
        };
        rec.components.push(component.clone());
        if status.is_none()
            && let Some(adjustment) = r
                .diagnostics
                .width_adjustments
                .last()
                .filter(|a| a.end_bit.is_none())
        {
            return Some(EntityViewStop::InvalidWidthOverride {
                field: adjustment.component.clone(),
            });
        }
        if !status? {
            if let Some(field) = required_runtime_field(name) {
                return Some(EntityViewStop::RuntimeContextUnavailable {
                    index,
                    name: name.to_owned(),
                    field: field.to_owned(),
                });
            }
            return Some(EntityViewStop::UnsupportedComponent {
                index,
                name: name.to_owned(),
            });
        }
        let check = if encoding.corruption_check {
            r.gate("component.corruption_check", 32, true)
        } else {
            Some(())
        };
        component.end_bit = r.cursor.position;
        component.fields = r.fields[field_start..].to_vec();
        if check.is_none() {
            component.status = crate::theater::film::ComponentReadStatus::Truncated;
        }
        *rec.components.last_mut().unwrap() = component;
        check?;
    }
    if rec.header.kind == RecordKind::New {
        if let Some(grammar) = &r.live_grammar {
            let width = grammar.new_record_tail_bits;
            read_reference_new_tail(r, width)?;
        } else {
            read_new_raw_bits(r, "new.terminal", encoding.new_record.terminal_bits)?;
        }
    }
    Some(EntityViewStop::Complete)
}

fn read_reference_new_tail(r: &mut ComponentReader<'_>, width: i64) -> Option<()> {
    if width <= 0 {
        return Some(());
    }
    read_reference_new_skip(
        r,
        width,
        "new.terminal",
        "new record tail",
        WidthPurpose::NewRecordTail,
    )
}

fn read_reference_new_skip(
    r: &mut ComponentReader<'_>,
    width: i64,
    name: &str,
    _field: &'static str,
    purpose: WidthPurpose,
) -> Option<()> {
    signed_skip(
        r,
        name,
        width,
        false,
        Some(purpose),
        (name, &format!("{name}.tail")),
    )
}

fn read_new_raw_bits(r: &mut ComponentReader<'_>, name: &str, bits: usize) -> Option<()> {
    r.words(name, bits / 64, 64)?;
    if !bits.is_multiple_of(64) {
        r.r(&format!("{name}.tail"), bits % 64)?;
    }
    Some(())
}
