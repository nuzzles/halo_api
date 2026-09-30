//! Sequential replication views and generation-aware entity bindings.
use super::Cursor;
use super::{PositionEncoding, Reader, defaults};
pub(crate) use crate::theater::film::chunks::replication::replication_stream::models::entity_records::{
    EntityComponentRead, EntityRecord, EntityViewStop,
};
use crate::theater::parser::v41::{FilmRegistry, RecordIdLayout, RecordKind, decode_record_header};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum BindingOrigin {
    #[default]
    Supplied,
    SequentialKeyframe {
        bit: usize,
    },
    Creation {
        bit: i64,
    },
}

/// Reference New-record default routing and calibration widths. Neither skip is
/// a component length; terminal bits apply only after a successful New body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct NewRecordEncoding {
    pub deserialize_defaults: bool,
    pub fallback_default_bits: usize,
    /// Raw reference configuration, converted only if this New uses fallback state.
    /// None preserves the checked legacy encoding contract.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_fallback_default_bits: Option<i64>,
    pub terminal_bits: usize,
}
impl Default for NewRecordEncoding {
    fn default() -> Self {
        Self {
            deserialize_defaults: true,
            fallback_default_bits: 0,
            reference_fallback_default_bits: None,
            terminal_bits: 0,
        }
    }
}
impl NewRecordEncoding {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

/// Full-state keyframe framing, independent of New-record framing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct KeyframeLayout {
    pub header_bits: usize,
    pub size_word_bits: usize,
}
impl Default for KeyframeLayout {
    fn default() -> Self {
        Self {
            header_bits: 108,
            size_word_bits: 32,
        }
    }
}
impl KeyframeLayout {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
    pub(super) fn valid(&self) -> bool {
        self.header_bits >= 64 && self.size_word_bits <= 32
    }
}

/// Values sourced from bootstrap/map context, not guessed from record contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FrameEncoding {
    #[serde(default, skip_serializing_if = "KeyframeLayout::is_default")]
    pub keyframe_layout: KeyframeLayout,
    /// Explicit inherited simulation-state gate for configured reference readers. None
    /// preserves the legacy convenience API's completed-body policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyframe_simulation_complete: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "super::ComponentWidthOverrides::is_empty"
    )]
    pub component_widths: super::ComponentWidthOverrides,
    #[serde(default, skip_serializing_if = "NewRecordEncoding::is_default")]
    pub new_record: NewRecordEncoding,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_capture: Option<crate::theater::parser::v41::PositionCaptureEncoding>,
    /// Original signed reference ID width; admitted only after a non-End prefix.
    /// None uses ids.low_bits. Reference readers still use wrapping slot arithmetic;
    /// the public bounded header reader retains its checked contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id_low_bits: Option<i64>,
    pub ids: RecordIdLayout,
    pub mpp_widths: [usize; 2],
    pub position: Option<PositionEncoding>,
    pub extra_fields: bool,
    pub corruption_check: bool,
}

impl FrameEncoding {
    pub(in crate::theater) fn valid(&self) -> bool {
        (self.reference_id_low_bits.is_some()
            || (self.ids.low_bits <= 30 && self.ids.base <= 0x3fff_ffff))
            && self.mpp_widths.iter().all(|w| (1..=32).contains(w))
            && self.position.as_ref().is_none_or(PositionEncoding::valid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct EntityBinding {
    pub id: u32,
    /// The generation was not recorded by the evidence establishing this binding.
    /// Strict delta admission still accepts any recorded generation for this slot.
    #[serde(default, skip_serializing_if = "binding_false")]
    pub generation_any: bool,
    pub archetype: u32,
    #[serde(default)]
    pub origin: BindingOrigin,
}

/// Bindings from checked keyframes and completely decoded creation records.
/// Keys are slots; each binding retains the full ID including generation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct EntityBindings {
    pub slots: BTreeMap<u32, EntityBinding>,
}

impl EntityBindings {
    pub(crate) fn bind(&mut self, id: u32, archetype: u32) {
        self.bind_with_origin(id, archetype, BindingOrigin::Supplied);
    }
    pub(in crate::theater) fn bind_with_origin(
        &mut self,
        id: u32,
        archetype: u32,
        origin: BindingOrigin,
    ) {
        self.slots.insert(
            id & 0x3fff_ffff,
            EntityBinding {
                id,
                generation_any: false,
                archetype,
                origin,
            },
        );
    }
}

/// Reference locator trials suppress movement independently of position capture.
pub(crate) struct RecordCaptureSlots {
    pub context: Option<crate::theater::parser::v41::ReaderContext>,
    pub movement: Option<u32>,
    pub position: Option<u32>,
}

pub(crate) fn decode_entity_record_with_capture_slots(
    data: &[u8],
    bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &EntityBindings,
    simulation_complete: bool,
    slots: RecordCaptureSlots,
) -> Option<EntityRecord> {
    decode_entity_record_with_capture_slots_and_generation_policy(
        data,
        bit,
        registry,
        encoding,
        bindings,
        simulation_complete,
        slots,
        true,
    )
}

/// Reference decodeDelta resolves an archetype by slot, after reading the baseline
/// selector. Other record entry points may require a generation admission check.
#[allow(clippy::too_many_arguments)]
pub(crate) fn decode_entity_record_with_capture_slots_and_generation_policy(
    data: &[u8],
    bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &EntityBindings,
    simulation_complete: bool,
    slots: RecordCaptureSlots,
    generation_strict: bool,
) -> Option<EntityRecord> {
    let bit = bit.try_into().ok()?;
    if !encoding.valid() {
        return None;
    }
    let capture_map = encoding.position_capture.as_ref().map(|c| c.map());
    let mut r = Reader {
        references: Vec::new(),
        reference_widths: slots.context.as_ref().map(|c| super::ComponentWidths {
            movement: &c.profile.movement,
            mpp: c.profile.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,

        live_grammar: slots.context.as_ref().map(|c| c.profile.grammar.clone()),
        position_capture: encoding
            .position_capture
            .as_ref()
            .zip(capture_map.as_ref())
            .zip(slots.position)
            .map(|((c, map), slot)| c.reader(map, slot)),
        position_start: 0,
        position_slot: 0,
        position_fallback: false,
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
        bindings,
        true,
        ComponentWalkPolicy {
            capture_record_slot: false,
            slot_mirror: None,
            generation_strict,
            simulation_complete,
            stub: None,
        },
    )
}
#[derive(Clone, Copy)]
struct ComponentWalkPolicy<'a> {
    capture_record_slot: bool,
    slot_mirror: Option<&'a std::cell::Cell<u32>>,
    generation_strict: bool,
    simulation_complete: bool,
    stub: Option<(&'a str, usize)>,
}

fn read_entity_record(
    r: &mut Reader<'_>,
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &EntityBindings,
    reference_policy: bool,
    policy: ComponentWalkPolicy<'_>,
) -> Option<EntityRecord> {
    r.fields.clear();
    if encoding.extra_fields {
        // Reference skips this prefix without reading it. Retain source fields
        // where addressable, but do not turn a negative skip into a read panic.
        if reference_policy && r.cursor.position < 0 {
            super::widths::signed_skip(
                r,
                "record.prefix",
                32,
                false,
                Some(crate::theater::parser::v41::WidthPurpose::RecordPrefix),
                ("record.prefix", "record.prefix"),
            )?;
        } else {
            r.r("record.prefix", 32)?;
        }
    }
    let header = if reference_policy {
        let width = encoding
            .reference_id_low_bits
            .or_else(|| i64::try_from(encoding.ids.low_bits).ok())?;
        if !r.cursor.fits_source(width) {
            return None;
        }
        r.cursor.reference_header(width, encoding.ids.base)
    } else {
        decode_record_header(data, r.cursor.address()?, encoding.ids)?
    };
    r.cursor.position = header.end_bit;
    if let (Some(slot), Some(id)) = (policy.slot_mirror, header.id) {
        slot.set(id & 0x3fff_ffff);
    }
    if !reference_policy || policy.capture_record_slot {
        // Generic DecodeFrameRecords assigns every record's slot, including New.
        // Production/inference callers instead supply their inherited reader slot.
        r.movement_slot = header.id.map(|id| id & 0x3fff_ffff);
        if let Some(capture) = r.position_capture.as_mut() {
            capture.slot = header.id.unwrap_or(0) & 0x3fff_ffff;
        }
    }
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
        bindings,
        reference_policy,
        policy,
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
    r: &mut Reader<'_>,
    rec: &mut EntityRecord,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &EntityBindings,
    reference_policy: bool,
    policy: ComponentWalkPolicy<'_>,
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
        let Some(binding) = bindings.slots.get(&(id & 0x3fff_ffff)) else {
            if reference_policy && !policy.generation_strict {
                r.gate("baseline", 7, true)?;
            }
            return Some(EntityViewStop::MissingBinding { id });
        };
        if policy.generation_strict && !binding.generation_any && binding.id != id {
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
    if ti >= 50 || (arch.is_none() && !(reference_policy && rec.header.kind == RecordKind::New)) {
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
                        crate::theater::parser::v41::WidthPurpose::NewRecordDefault,
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
        rec.default_state = Some(super::DefaultState {
            source: super::BitRange {
                start: usize::try_from(default_start).ok()?,
                end: usize::try_from(r.cursor.position).ok()?,
            },
            fields: r.fields.drain(field_start..).collect(),
            status: match result {
                Some(true) if skip_default => super::DefaultStateStatus::Opaque,
                Some(true) => super::DefaultStateStatus::Complete,
                Some(false) => super::DefaultStateStatus::Unsupported,
                None => super::DefaultStateStatus::Truncated,
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
    let component_count = if reference_policy {
        arch.components.len()
    } else {
        64
    };
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
        let status = super::widths::read_component(
            r,
            name,
            registry_component.precision_level,
            ti,
            Some(&encoding.component_widths),
            (policy.simulation_complete, policy.stub),
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
            if let Some(field) = super::m4b::required_runtime_field(name) {
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
            component.status = super::ComponentReadStatus::Truncated;
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

fn read_reference_new_tail(r: &mut Reader<'_>, width: i64) -> Option<()> {
    if width <= 0 {
        return Some(());
    }
    read_reference_new_skip(
        r,
        width,
        "new.terminal",
        "new record tail",
        crate::theater::parser::v41::WidthPurpose::NewRecordTail,
    )
}

fn read_reference_new_skip(
    r: &mut Reader<'_>,
    width: i64,
    name: &str,
    _field: &'static str,
    purpose: crate::theater::parser::v41::WidthPurpose,
) -> Option<()> {
    super::widths::signed_skip(
        r,
        name,
        width,
        false,
        Some(purpose),
        (name, &format!("{name}.tail")),
    )
}

fn read_new_raw_bits(r: &mut Reader<'_>, name: &str, bits: usize) -> Option<()> {
    r.words(name, bits / 64, 64)?;
    if !bits.is_multiple_of(64) {
        r.r(&format!("{name}.tail"), bits % 64)?;
    }
    Some(())
}

fn binding_false(value: &bool) -> bool {
    !value
}

impl crate::theater::parser::v41::KeyframeReadLayout {
    /// Address-sized representation for legacy consumers; admission remains at use.
    pub(crate) fn bounded(self) -> Option<KeyframeLayout> {
        Some(KeyframeLayout {
            header_bits: self.header_bits.try_into().ok()?,
            size_word_bits: self.size_word_bits.try_into().ok()?,
        })
    }
}
