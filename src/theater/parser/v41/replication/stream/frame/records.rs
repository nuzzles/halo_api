//! Sequential replication views and generation-aware entity bindings.
use crate::theater::parser::v41::context::layout::RecordLayout;
use crate::theater::parser::v41::context::profile::DecodeProfile;
use crate::theater::parser::v41::reference::diagnostics::WidthPurpose;
use crate::theater::parser::v41::registry::source::FilmRegistry;
use crate::theater::parser::v41::replication::components::cursor::ComponentCursor as Cursor;
use crate::theater::parser::v41::replication::components::defaults;
use crate::theater::parser::v41::replication::components::m4b::required_runtime_field;
use crate::theater::parser::v41::replication::components::reader::{
    ComponentReader, ComponentWidths,
};
use crate::theater::parser::v41::replication::components::widths::{read_component, signed_skip};
use crate::theater::parser::v41::replication::source::stream::models::entity_records::{
    EntityComponentRead, EntityRecord, EntityViewStop,
};
use crate::theater::parser::v41::replication::source::{RecordHeader, RecordKind};

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
    header: RecordHeader,
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    binding: Option<RecordBinding>,
    simulation_complete: bool,
    slots: RecordReadContext<'_>,
) -> Option<EntityRecord> {
    if !encoding.valid() {
        return None;
    }
    // The frame reader already consumed the optional prefix and validated
    // this header against the source before passing its body layout here.
    debug_assert!(!encoding.extra_fields);
    let mut r = ComponentReader {
        category_references: encoding.category_references,
        snapshot: false,
        references: Vec::new(),
        reference_widths: slots.context.as_ref().map(|c| ComponentWidths {
            movement: &c.movement,
            mpp: c.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,
        context_error: None,

        live_grammar: slots.context.map(|c| &c.grammar),
        movement_slot: slots.movement,
        diagnostics: Default::default(),
        cursor: Cursor::signed(data, header.end_bit, None),
        fields: Vec::with_capacity(32),
        position_encoding: encoding.position.as_ref(),
    };
    read_entity_record(
        &mut r,
        header,
        data,
        registry,
        encoding,
        binding,
        simulation_complete,
    )
}
fn read_entity_record(
    r: &mut ComponentReader<'_>,
    header: RecordHeader,
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &RecordLayout,
    binding: Option<RecordBinding>,
    simulation_complete: bool,
) -> Option<EntityRecord> {
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
        rec.default_state = Some(crate::theater::parser::v41::replication::source::DefaultState {
            source: crate::theater::parser::v41::packet::BitRange {
                start: usize::try_from(default_start).ok()?,
                end: usize::try_from(r.cursor.position).ok()?,
            },
            fields: r.fields.drain(field_start..).collect(),
            status: match result {
                Some(true) if skip_default => crate::theater::parser::v41::replication::source::DefaultStateStatus::Opaque,
                Some(true) => crate::theater::parser::v41::replication::source::DefaultStateStatus::Complete,
                Some(false) => crate::theater::parser::v41::replication::source::DefaultStateStatus::Unsupported,
                None => crate::theater::parser::v41::replication::source::DefaultStateStatus::Truncated,
            },
        });
        if r.reference_configuration_unavailable() {
            rec.default_state.as_mut().unwrap().status =
                crate::theater::parser::v41::replication::source::DefaultStateStatus::ContextUnavailable;
            return Some(EntityViewStop::UnsupportedReferenceConfiguration);
        }
        if !result? {
            return Some(EntityViewStop::UnsupportedDefault { archetype: ti });
        }
    }
    // A clear creation gate ends the optional initial-component section. Some
    // creation types omit the gate itself; neither form carries a mask.
    let has_components = rec.header.kind != RecordKind::New
        || (creation_has_component_gate(ti) && r.bit("new.component_gate")?);
    let mask = if has_components {
        Some(r.mask()?)
    } else {
        None
    };
    rec.mask = mask;
    // The recorded NEW archetype is checked even when it has no components.
    let Some(arch) = arch else {
        return Some(EntityViewStop::InvalidArchetype { archetype: ti });
    };
    // The reference registry loop ignores mask bits beyond its component list.
    let component_count = arch.components.len();
    for index in ComponentMaskIndices::new(mask.unwrap_or(0), component_count) {
        let Some(registry_component) = arch.components.get(index) else {
            return Some(EntityViewStop::InvalidComponent { index });
        };
        let name = registry_component.name.as_str();
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
        let field_range = field_start..r.fields.len();
        rec.components.push(EntityComponentRead {
            index,
            name: crate::theater::parser::v41::replication::components::reader::intern_name(name),
            start_bit,
            end_bit: r.cursor.position,
            status: status.into(),
            field_range,
        });
        if r.reference_configuration_unavailable() {
            rec.components.last_mut().unwrap().status =
                crate::theater::parser::v41::replication::source::ComponentReadStatus::ContextUnavailable;
            return Some(EntityViewStop::UnsupportedReferenceConfiguration);
        }
        if let Some(field) = r.runtime_context_unavailable() {
            rec.components.last_mut().unwrap().status =
                crate::theater::parser::v41::replication::source::ComponentReadStatus::ContextUnavailable;
            return Some(EntityViewStop::RuntimeContextUnavailable {
                index,
                name: name.to_owned(),
                field: field.to_owned(),
            });
        }
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
        let Some(check) = encoding.corruption_check else {
            rec.components.last_mut().unwrap().status =
                crate::theater::parser::v41::replication::source::ComponentReadStatus::ContextUnavailable;
            return Some(EntityViewStop::MissingCorruptionCheckContext);
        };
        let check = if check {
            r.gate("component.corruption_check", 32, true)
        } else {
            Some(())
        };
        let end_bit = r.cursor.position;
        let component = rec.components.last_mut().unwrap();
        component.end_bit = end_bit;
        component.field_range.end = r.fields.len();
        if check.is_none() {
            component.status =
                crate::theater::parser::v41::replication::source::ComponentReadStatus::Truncated;
        }
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

/// These v41 creation forms have no initial-component section or gate.
fn creation_has_component_gate(archetype: u32) -> bool {
    !matches!(archetype, 7 | 13 | 14 | 20 | 23 | 28 | 29 | 46 | 47 | 49)
}

/// The source mask is 64 bits; schemas longer than that reuse those bits.
struct ComponentMaskIndices {
    mask: u64,
    remaining: u64,
    base: usize,
    count: usize,
}

impl ComponentMaskIndices {
    fn new(mask: u64, count: usize) -> Self {
        Self {
            mask,
            remaining: mask,
            base: 0,
            count,
        }
    }
}

impl Iterator for ComponentMaskIndices {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.base >= self.count {
                return None;
            }
            if self.remaining == 0 {
                self.base = self.base.saturating_add(64);
                self.remaining = self.mask;
                continue;
            }
            let index = self.base + self.remaining.trailing_zeros() as usize;
            self.remaining &= self.remaining - 1;
            if index >= self.count {
                return None;
            }
            return Some(index);
        }
    }
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

#[cfg(test)]
mod mask_tests {
    use super::ComponentMaskIndices;

    #[test]
    fn set_bits_match_the_recorded_wrapping_mask() {
        for count in [0, 1, 6, 63, 64, 65, 70, 129] {
            for mask in [0, 1, 0b100010, 1 << 63, u64::MAX] {
                let expected = (0..count)
                    .filter(|&index| mask & (1 << (index & 63)) != 0)
                    .collect::<Vec<_>>();
                assert_eq!(
                    ComponentMaskIndices::new(mask, count).collect::<Vec<_>>(),
                    expected
                );
            }
        }
    }
}
