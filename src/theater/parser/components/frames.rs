//! Sequential replication views and generation-aware entity bindings.
use super::Cursor;
use super::{ComponentField, PositionEncoding, Reader, defaults};
use crate::theater::{
    FilmRegistry, RecordHeader, RecordIdLayout, RecordKind, decode_record_header,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Native New-record default routing and calibration widths. Neither skip is
/// a component length; terminal bits apply only after a successful New body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct NewRecordEncoding {
    pub deserialize_defaults: bool,
    pub fallback_default_bits: usize,
    /// Raw native configuration, converted only if this New uses fallback state.
    /// None preserves the checked legacy encoding contract.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_fallback_default_bits: Option<i64>,
    pub terminal_bits: usize,
}
impl Default for NewRecordEncoding {
    fn default() -> Self {
        Self {
            deserialize_defaults: true,
            fallback_default_bits: 0,
            native_fallback_default_bits: None,
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
    /// Explicit inherited simulation-state gate for configured native readers. None
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
    pub position_capture: Option<crate::theater::PositionCaptureEncoding>,
    /// Original signed native ID width; admitted only after a non-End prefix.
    /// None uses ids.low_bits. Native readers still use wrapping slot arithmetic;
    /// the public bounded header reader retains its checked contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_id_low_bits: Option<i64>,
    pub ids: RecordIdLayout,
    pub mpp_widths: [usize; 2],
    pub position: Option<PositionEncoding>,
    pub extra_fields: bool,
    pub corruption_check: bool,
}

impl FrameEncoding {
    pub(in crate::theater) fn valid(&self) -> bool {
        (self.native_id_low_bits.is_some()
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingOrigin {
    #[default]
    Supplied,
    SequentialKeyframe {
        bit: usize,
    },
    Creation {
        bit: i64,
    },
    RecoveredKeyframe {
        bit: usize,
    },
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityViewStop {
    Complete,
    Truncated,
    InvalidEncoding,
    InvalidWidthOverride {
        adjustment: Box<crate::theater::NativeWidthAdjustment>,
    },
    InvalidArchetype {
        archetype: u32,
    },
    MissingBinding {
        id: u32,
    },
    GenerationMismatch {
        id: u32,
        bound_id: u32,
    },
    UnsupportedDefault {
        archetype: u32,
    },
    InvalidComponent {
        index: usize,
    },
    UnsupportedComponent {
        index: usize,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityComponentSpan {
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    /// Component payload end, before any corruption check.
    pub end_bit: i64,
}

/// An attempted component, including known fields before an unsupported or truncated body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityComponentAttempt {
    pub span: EntityComponentSpan,
    /// Native returned variant, including zero for a calibrated skip and
    /// u32::MAX for no variant. None marks an incomplete read or older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<u32>,
    /// true = complete, false = unsupported, None = truncated.
    pub status: Option<bool>,
    /// Ordered diagnostic observations emitted by this attempt. Absent in older serialized data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_range: Option<[usize; 2]>,
    pub field_start: usize,
    pub field_end: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRecord {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<crate::theater::NativeUnitReference>,
    #[serde(
        default,
        skip_serializing_if = "crate::theater::FilmReadDiagnostics::is_empty"
    )]
    pub diagnostics: crate::theater::FilmReadDiagnostics,
    pub header: RecordHeader,
    pub archetype: Option<u32>,
    /// Configured signed width returned as native New Trace.DefaultBits, even
    /// when the selected archetype decoder ignores it. This is decoder context,
    /// not a recorded bit length. None denotes an older export, a non-New record,
    /// a traversal that did not reach the default-state trace, or a legacy width
    /// outside the native signed domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_state_bits: Option<i64>,
    /// A disabled/missing default decoder used the configured fallback. Verified
    /// zero-bit stubs with a zero configured skip leave this false.
    #[serde(default)]
    pub default_state_fallback: bool,
    pub binding_origin: Option<BindingOrigin>,
    pub mask: Option<u64>,
    pub fields: Vec<ComponentField>,
    pub components: Vec<EntityComponentSpan>,
    #[serde(default)]
    pub attempts: Vec<EntityComponentAttempt>,
    pub end_bit: i64,
    /// Tail bits supplied by the native padding convention, absent from the payload.
    #[serde(default)]
    pub padded_bits: usize,
    pub stop: EntityViewStop,
}

/// Native locator trials suppress movement independently of position capture.
pub(crate) struct RecordCaptureSlots {
    pub context: Option<crate::theater::NativeReaderContext>,
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

/// Native decodeDelta resolves an archetype by slot, after reading the baseline
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
        native_widths: slots
            .context
            .as_ref()
            .map(|c| super::NativeComponentWidths {
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
        references: Vec::new(),
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
    native_policy: bool,
    policy: ComponentWalkPolicy<'_>,
) -> Option<EntityRecord> {
    r.fields.clear();
    if encoding.extra_fields {
        // Native skips this prefix without reading it. Retain source fields
        // where addressable, but do not turn a negative skip into a read panic.
        if native_policy && r.cursor.position < 0 {
            super::widths::signed_skip(
                r,
                "record.prefix",
                32,
                false,
                Some(crate::theater::NativeWidthPurpose::RecordPrefix),
                ("record.prefix", "record.prefix"),
            )?;
        } else {
            r.r("record.prefix", 32)?;
        }
    }
    let header = if native_policy {
        let width = encoding
            .native_id_low_bits
            .or_else(|| i64::try_from(encoding.ids.low_bits).ok())?;
        r.cursor.native_header(width, encoding.ids.base)
    } else {
        decode_record_header(data, r.cursor.address()?, encoding.ids)?
    };
    r.cursor.position = header.end_bit;
    if let (Some(slot), Some(id)) = (policy.slot_mirror, header.id) {
        slot.set(id & 0x3fff_ffff);
    }
    if !native_policy || policy.capture_record_slot {
        // Generic DecodeFrameRecords assigns every record's slot, including New.
        // Production/inference callers instead supply their inherited reader slot.
        r.movement_slot = header.id.map(|id| id & 0x3fff_ffff);
        if let Some(capture) = r.position_capture.as_mut() {
            capture.slot = header.id.unwrap_or(0) & 0x3fff_ffff;
        }
    }
    let mut rec = EntityRecord {
        references: Vec::new(),
        diagnostics: Default::default(),
        header,
        archetype: None,
        default_state_bits: None,
        default_state_fallback: false,
        binding_origin: None,
        mask: None,
        fields: Vec::new(),
        components: Vec::new(),
        attempts: Vec::new(),
        end_bit: r.cursor.position,
        padded_bits: 0,
        stop: EntityViewStop::Truncated,
    };
    rec.stop = record_body(
        r,
        &mut rec,
        registry,
        encoding,
        bindings,
        native_policy,
        policy,
    )
    .unwrap_or(EntityViewStop::Truncated);
    rec.end_bit = r.cursor.position;
    rec.padded_bits =
        crate::theater::bits::padded_from_native(rec.end_bit, data.len().saturating_mul(8));
    rec.fields = std::mem::take(&mut r.fields);
    rec.references = std::mem::take(&mut r.references);
    rec.diagnostics = std::mem::take(&mut r.diagnostics);
    Some(rec)
}

fn record_body(
    r: &mut Reader<'_>,
    rec: &mut EntityRecord,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &EntityBindings,
    native_policy: bool,
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
        rec.default_state_bits = encoding
            .new_record
            .native_fallback_default_bits
            .or_else(|| i64::try_from(encoding.new_record.fallback_default_bits).ok());
        r.r("archetype", 6)? as u32
    } else {
        let Some(binding) = bindings.slots.get(&(id & 0x3fff_ffff)) else {
            if native_policy && !policy.generation_strict {
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
        rec.binding_origin = Some(binding.origin.clone());
        r.gate("baseline", 7, true)?;
        binding.archetype
    };
    rec.archetype = Some(ti);
    let arch = registry.archetype(ti as usize).filter(|_| ti < 50);
    if ti >= 50 || (arch.is_none() && !(native_policy && rec.header.kind == RecordKind::New)) {
        return Some(EntityViewStop::InvalidArchetype { archetype: ti });
    }
    if rec.header.kind == RecordKind::New {
        let skip_default = ti != 35
            && (!encoding.new_record.deserialize_defaults
                || !(ti == 41 || defaults::has_native_deserializer(ti)));
        rec.default_state_fallback = skip_default
            && (!encoding.new_record.deserialize_defaults
                || encoding
                    .new_record
                    .native_fallback_default_bits
                    .map_or(encoding.new_record.fallback_default_bits > 0, |bits| {
                        bits != 0
                    })
                || matches!(ti, 23 | 41 | 44));
        if skip_default {
            if let Some(width) = encoding.new_record.native_fallback_default_bits {
                read_native_new_skip(
                    r,
                    width,
                    "new.default_skipped",
                    "new default state",
                    crate::theater::NativeWidthPurpose::NewRecordDefault,
                )?;
            } else {
                read_new_raw_bits(
                    r,
                    "new.default_skipped",
                    encoding.new_record.fallback_default_bits,
                )?;
            }
        } else if !(if ti == 41 {
            defaults::projectile(r, encoding.mpp_widths, true)
        } else {
            defaults::state(r, ti, encoding.mpp_widths)
        })? {
            return Some(EntityViewStop::UnsupportedDefault { archetype: ti });
        }
        r.bit("new.component_gate")?;
    }
    let mask = r.mask()?;
    rec.mask = Some(mask);
    // Native NEW reads its default state, gate and mask before registry lookup.
    let Some(arch) = arch else {
        return Some(EntityViewStop::InvalidArchetype { archetype: ti });
    };
    // The native registry loop ignores mask bits beyond its component list.
    let component_count = if native_policy {
        arch.components.len()
    } else {
        64
    };
    for index in 0..component_count {
        if mask & (1 << (index & 63)) == 0 {
            continue;
        }
        let Some(name) = arch.components.get(index) else {
            return Some(EntityViewStop::InvalidComponent { index });
        };
        let start_bit = r.cursor.position;
        let field_start = r.fields.len();
        let observation_start = r.diagnostics.component_observations.len();
        let calibrated = super::widths::is_calibrated(r, name, Some(&encoding.component_widths));
        let status = super::widths::read_component(
            r,
            name,
            arch.levels.get(index).copied().unwrap_or(0),
            ti,
            Some(&encoding.component_widths),
            (policy.simulation_complete, policy.stub),
        );
        let span = EntityComponentSpan {
            index,
            name: name.clone(),
            start_bit,
            end_bit: r.cursor.position,
        };
        rec.attempts.push(EntityComponentAttempt {
            variant: status
                .map(|_| super::widths::result_variant(name, &r.fields[field_start..], calibrated)),
            span: span.clone(),
            status,
            field_start,
            field_end: r.fields.len(),
            observation_range: Some([
                observation_start,
                r.diagnostics.component_observations.len(),
            ]),
        });
        if status.is_none()
            && let Some(adjustment) = r
                .diagnostics
                .width_adjustments
                .last()
                .filter(|a| a.end_bit.is_none())
        {
            return Some(EntityViewStop::InvalidWidthOverride {
                adjustment: Box::new(adjustment.clone()),
            });
        }
        if !status? {
            return Some(EntityViewStop::UnsupportedComponent {
                index,
                name: name.clone(),
            });
        }
        rec.components.push(span);
        if encoding.corruption_check {
            r.gate("component.corruption_check", 32, true)?;
        }
    }
    if rec.header.kind == RecordKind::New {
        if let Some(grammar) = &r.live_grammar {
            let width = grammar.new_record_tail_bits;
            read_native_new_tail(r, width)?;
        } else {
            read_new_raw_bits(r, "new.terminal", encoding.new_record.terminal_bits)?;
        }
    }
    Some(EntityViewStop::Complete)
}

fn read_native_new_tail(r: &mut Reader<'_>, width: i64) -> Option<()> {
    if width <= 0 {
        return Some(());
    }
    read_native_new_skip(
        r,
        width,
        "new.terminal",
        "new record tail",
        crate::theater::NativeWidthPurpose::NewRecordTail,
    )
}

fn read_native_new_skip(
    r: &mut Reader<'_>,
    width: i64,
    name: &str,
    _field: &'static str,
    purpose: crate::theater::NativeWidthPurpose,
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
