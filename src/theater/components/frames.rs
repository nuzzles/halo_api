//! Sequential replication views and generation-aware entity bindings.
use super::Cursor;
use super::{
    ComponentField, DecodedFrameView, FrameViewStop, PositionEncoding, Reader, decode_message_view,
    defaults,
};
use crate::theater::{
    FilmRegistry, RecordHeader, RecordIdLayout, RecordKind, decode_record_header,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Native New-record default routing and calibration widths. Neither skip is
/// a component length; terminal bits apply only after a successful New body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NewRecordEncoding {
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
pub struct KeyframeLayout {
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
pub struct FrameEncoding {
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
pub struct EntityBinding {
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
pub struct EntityBindings {
    pub slots: BTreeMap<u32, EntityBinding>,
}

fn binding_false(value: &bool) -> bool {
    !value
}

impl EntityBindings {
    /// Bind an archetype established without a known entity generation.
    /// Like native BindWildcard, discard namespace bits and retain uncertainty
    /// until a successfully decoded New record replaces the binding.
    pub fn bind_wildcard(&mut self, slot: u32, archetype: u32) {
        let slot = slot & 0x3fff_ffff;
        self.bind(slot, archetype);
        self.slots.get_mut(&slot).unwrap().generation_any = true;
    }
    pub fn bind(&mut self, id: u32, archetype: u32) {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedEntityView {
    pub start_bit: i64,
    pub end_bit: i64,
    /// Includes End, including its optional prefix word.
    pub records: Vec<EntityRecord>,
    /// Fields read before an incomplete next header (such as its prefix word).
    pub trailing_fields: Vec<ComponentField>,
    pub stop: EntityViewStop,
}

/// Walk view B from its known start until End or the first unparseable record.
/// Fully decoded earlier records update bindings even if a later record fails.
/// Failed creation records never install speculative bindings.
pub fn decode_entity_view(
    data: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &mut EntityBindings,
) -> DecodedEntityView {
    decode_entity_view_with_generation_policy(data, bit, registry, encoding, bindings, true)
}

/// Generic record loop with native `GenerationStricte` policy.
/// When false, deltas resolve the bound archetype by slot despite a different
/// generation. The recorded ID is retained and the existing binding is unchanged.
/// This does not relax missing-binding checks or recover malformed records.
pub fn decode_entity_view_with_generation_policy(
    data: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &mut EntityBindings,
    generation_strict: bool,
) -> DecodedEntityView {
    decode_entity_view_impl(
        data,
        bit as i64,
        registry,
        encoding,
        bindings,
        generation_strict,
        EntityViewMode {
            native: false,
            runtime: None,
            cursor_mirror: None,
        },
    )
    .0
}

/// Generic native record loop starting at a caller-supplied record boundary.
/// Reads beyond the buffer produce synthetic zero bits. Every record, including
/// End, reports `padded_bits`; source bytes are never extended or replaced.
/// Like the bounded API, this retains End and updates entity bindings. It does
/// not consume a packet preamble or provide the native World's auxiliary caches.
/// The encoding's simulation-completion policy is honored; None uses the native
/// standalone default (false), matching direct native component dispatch.
pub fn decode_native_entity_view(
    data: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &mut EntityBindings,
    generation_strict: bool,
) -> DecodedEntityView {
    decode_entity_view_impl(
        data,
        bit as i64,
        registry,
        encoding,
        bindings,
        generation_strict,
        EntityViewMode {
            native: true,
            runtime: None,
            cursor_mirror: None,
        },
    )
    .0
}

/// Decode a native generic record loop using a caller-owned reader.
/// Positive preamble widths are consumed only when the reader starts at bit zero.
/// Returns all records including End, and leaves the reader at the consumed end
/// even on a record desynchronization or panic during component traversal. Earlier
/// binding mutations are retained. Signed starting positions follow native read
/// panic behavior. Native observer and
/// capture state are represented by the returned record diagnostics, not stored
/// on this source-only bit reader.
pub fn decode_native_frame_records(
    reader: &mut crate::theater::NativeFilmBits<'_>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &mut EntityBindings,
    generation_strict: bool,
    packet_preamble_bits: i64,
) -> Option<DecodedEntityView> {
    let mut start = reader.position();
    if start == 0 && packet_preamble_bits > 0 {
        start = packet_preamble_bits;
    }
    let data = reader.octets();
    let position = std::cell::Cell::new(start);
    struct Restore<'r, 'd> {
        reader: &'r mut crate::theater::NativeFilmBits<'d>,
        position: &'r std::cell::Cell<i64>,
    }
    impl Drop for Restore<'_, '_> {
        fn drop(&mut self) {
            self.reader.set_position(self.position.get());
        }
    }
    let _restore = Restore {
        reader,
        position: &position,
    };
    let view = decode_entity_view_impl(
        data,
        start,
        registry,
        encoding,
        bindings,
        generation_strict,
        EntityViewMode {
            native: true,
            runtime: None,
            cursor_mirror: Some(&position),
        },
    )
    .0;
    position.set(view.end_bit);
    Some(view)
}

pub(crate) struct NativeFrameRuntime<'a> {
    pub context: crate::theater::NativeReaderContext,
    pub world: Option<&'a mut crate::theater::FilmWorld>,
    pub accumulator: Option<&'a mut crate::theater::FilmWorld>,
    pub accumulate_in_world: bool,
    pub cursor_mirror: Option<&'a std::cell::Cell<i64>>,
    pub slot_mirror: Option<&'a std::cell::Cell<u32>>,
}

pub(crate) fn decode_native_entity_view_live(
    data: &[u8],
    bit: i64,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &mut EntityBindings,
    runtime: NativeFrameRuntime<'_>,
) -> (DecodedEntityView, Option<&'static str>) {
    let strict = runtime.context.profile.grammar.generation_strict;
    decode_entity_view_impl(
        data,
        bit,
        registry,
        encoding,
        bindings,
        strict,
        EntityViewMode {
            native: true,
            cursor_mirror: runtime.cursor_mirror,
            runtime: Some(runtime),
        },
    )
}

struct EntityViewMode<'a> {
    native: bool,
    runtime: Option<NativeFrameRuntime<'a>>,
    cursor_mirror: Option<&'a std::cell::Cell<i64>>,
}

fn decode_entity_view_impl(
    data: &[u8],
    bit: i64,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &mut EntityBindings,
    generation_strict: bool,
    mode: EntityViewMode<'_>,
) -> (DecodedEntityView, Option<&'static str>) {
    let EntityViewMode {
        native,
        mut runtime,
        cursor_mirror,
    } = mode;
    let mut out = DecodedEntityView {
        start_bit: bit,
        end_bit: bit,
        records: Vec::new(),
        trailing_fields: Vec::new(),
        stop: EntityViewStop::Truncated,
    };
    if !encoding.valid() {
        out.stop = EntityViewStop::InvalidEncoding;
        return (out, None);
    }
    let cursor = if native {
        Cursor::signed(data, bit, cursor_mirror)
    } else {
        let Some(cursor) = usize::try_from(bit)
            .ok()
            .and_then(|bit| Cursor::new(data, bit))
        else {
            return (out, None);
        };
        cursor
    };
    let capture_map = encoding.position_capture.as_ref().map(|c| c.map());
    let movement = runtime.as_ref().map(|r| r.context.profile.movement.clone());
    let mut r = Reader {
        native_widths: movement
            .as_ref()
            .map(|movement| super::NativeComponentWidths {
                movement,
                mpp: runtime.as_ref().unwrap().context.profile.mpp,
                maximum: u64::MAX,
            }),
        width_error: None,
        live_observer: runtime.as_ref().and_then(|r| r.context.observer.clone()),
        live_grammar: runtime.as_ref().map(|r| r.context.profile.grammar.clone()),
        position_capture: encoding
            .position_capture
            .as_ref()
            .zip(capture_map.as_ref())
            .map(|(c, map)| {
                let mut capture = c.reader(map, 0);
                capture.world = runtime.as_mut().and_then(|r| {
                    if r.accumulate_in_world {
                        r.world.take()
                    } else {
                        r.accumulator.take()
                    }
                });
                capture
            }),
        position_start: 0,
        position_slot: 0,
        position_fallback: false,
        movement_slot: None,
        references: Vec::new(),
        diagnostics: Default::default(),
        cursor,
        fields: Vec::new(),
        position_encoding: encoding.position.as_ref(),
    };
    while let Some(rec) = read_entity_record(
        &mut r,
        data,
        registry,
        encoding,
        bindings,
        native,
        ComponentWalkPolicy {
            capture_record_slot: true,
            slot_mirror: runtime.as_ref().and_then(|r| r.slot_mirror),
            generation_strict,
            simulation_complete: if native {
                encoding.keyframe_simulation_complete.unwrap_or(false)
            } else {
                true
            },
            stub: None,
        },
    ) {
        out.stop = rec.stop.clone();
        let end = rec.header.kind == RecordKind::End;
        if rec.stop == EntityViewStop::Complete {
            if let Some(runtime) = runtime.as_mut() {
                // A shared traversal/capture world has one mutable owner: the
                // position reader between records, then this commit operation.
                let world = runtime
                    .world
                    .as_deref_mut()
                    .or_else(|| {
                        r.position_capture
                            .as_mut()
                            .and_then(|c| c.world.as_deref_mut())
                    })
                    .expect("live frame traversal world");
                match (rec.header.kind, rec.header.id) {
                    (RecordKind::New, Some(id)) => world.bind_full(id, rec.archetype.unwrap()),
                    (RecordKind::Delete, Some(id)) => world.unbind(id & 0x3fff_ffff),
                    _ => {}
                }
            }
            match (rec.header.kind, rec.header.id) {
                (RecordKind::New, Some(id)) => bindings.bind_with_origin(
                    id,
                    rec.archetype.expect("complete creation has archetype"),
                    BindingOrigin::Creation {
                        bit: rec.header.start_bit,
                    },
                ),
                (RecordKind::Delete, Some(id)) => {
                    bindings.slots.remove(&(id & 0x3fff_ffff));
                }
                _ => {}
            }
        }
        out.records.push(rec);
        if end || out.stop != EntityViewStop::Complete {
            out.end_bit = r.cursor.position;
            return (out, r.width_error);
        }
    }
    out.end_bit = r.cursor.position;
    out.trailing_fields = r.fields;
    out.stop = EntityViewStop::Truncated;
    (out, r.width_error)
}

/// Decode one record without mutating bindings; production world policy controls commitment.
pub(crate) fn decode_entity_record_at(
    data: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &EntityBindings,
) -> Option<EntityRecord> {
    decode_entity_record_at_with_simulation_policy(data, bit, registry, encoding, bindings, true)
}

pub(crate) fn decode_entity_record_at_with_simulation_policy(
    data: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &EntityBindings,
    simulation_complete: bool,
) -> Option<EntityRecord> {
    decode_entity_record_with_movement(
        data,
        bit,
        registry,
        encoding,
        bindings,
        simulation_complete,
        None,
    )
}

pub(crate) fn decode_entity_record_with_movement(
    data: &[u8],
    bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &EntityBindings,
    simulation_complete: bool,
    movement_slot: Option<u32>,
) -> Option<EntityRecord> {
    decode_entity_record_with_capture_slots(
        data,
        bit,
        registry,
        encoding,
        bindings,
        simulation_complete,
        RecordCaptureSlots {
            context: None,
            movement: movement_slot,
            position: movement_slot,
        },
    )
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
        live_observer: slots.context.as_ref().and_then(|c| c.observer.clone()),
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

/// Re-decode one record with an explicit extra width for an unsupported component.
/// The inferred tail remains raw data with exact bit positions.
pub(crate) fn decode_entity_record_with_stub(
    data: &[u8],
    record: &EntityRecord,
    registry: &FilmRegistry,
    profile: &crate::theater::KillWalkProfile,
    bindings: &EntityBindings,
    stub: (&str, usize),
    context: Option<&crate::theater::NativeReaderContext>,
) -> Option<EntityRecord> {
    // Native repair starts after the inference loop's prefix/header, and NEW
    // bodies do not carry the generic reader's optional extra-byte field.
    let mut encoding = profile.encoding.clone();
    encoding.extra_fields = false;
    encoding.component_widths.stubs.clear();
    let slot = if record.header.kind == RecordKind::Delta {
        record.header.id.unwrap_or(0) & 0x3fff_ffff
    } else {
        0
    };
    let capture_map = encoding.position_capture.as_ref().map(|c| c.map());
    let mut r = Reader {
        native_widths: context.map(|c| super::NativeComponentWidths {
            movement: &c.profile.movement,
            mpp: c.profile.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,
        live_observer: context.and_then(|c| c.observer.clone()),
        live_grammar: context.map(|c| c.profile.grammar.clone()),
        position_capture: context.and_then(|_| {
            encoding
                .position_capture
                .as_ref()
                .zip(capture_map.as_ref())
                .map(|(c, m)| c.reader(m, slot))
        }),
        position_start: 0,
        position_slot: slot,
        position_fallback: false,
        movement_slot: context.map(|_| slot),
        references: Vec::new(),
        diagnostics: Default::default(),
        cursor: Cursor::signed(data, record.header.start_bit, None),
        fields: vec![],
        position_encoding: profile.encoding.position.as_ref(),
    };
    read_entity_record(
        &mut r,
        data,
        registry,
        &encoding,
        bindings,
        true,
        ComponentWalkPolicy {
            capture_record_slot: false,
            slot_mirror: None,
            generation_strict: true,
            simulation_complete: profile.simulation_complete,
            stub: context.is_none().then_some(stub),
        },
    )
}

/// Native chain-inference body trial with retained read diagnostics. The supplied offset is
/// the mask start; this deliberately does not consume a baseline selector.
pub(crate) fn chain_delta_body_trial_contextual(
    data: &[u8],
    bit: i64,
    registry: &FilmRegistry,
    ti: u32,
    encoding: &FrameEncoding,
    simulation_complete: bool,
    context: Option<&crate::theater::NativeReaderContext>,
) -> (Option<(i64, usize)>, crate::theater::FilmReadDiagnostics) {
    let Some(arch) = registry.archetype(ti as usize) else {
        return (None, Default::default());
    };
    let capture_map = encoding.position_capture.as_ref().map(|c| c.map());
    let mut r = Reader {
        native_widths: context.map(|c| super::NativeComponentWidths {
            movement: &c.profile.movement,
            mpp: c.profile.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,
        live_observer: context.and_then(|c| c.observer.clone()),
        live_grammar: context.map(|c| c.profile.grammar.clone()),
        position_capture: context.and_then(|_| {
            encoding
                .position_capture
                .as_ref()
                .zip(capture_map.as_ref())
                .map(|(c, m)| c.reader(m, 0))
        }),
        position_start: 0,
        position_slot: 0,
        position_fallback: false,
        movement_slot: context.map(|_| 0),
        references: Vec::new(),
        diagnostics: Default::default(),
        cursor: Cursor::signed(data, bit, None),
        fields: vec![],
        position_encoding: encoding.position.as_ref(),
    };
    let result = (|| {
        let mask = r.mask()?;
        let mut count = 0;
        for (index, name) in arch.components.iter().enumerate() {
            if mask & (1 << (index & 63)) == 0 {
                continue;
            }
            if !super::widths::read_component(
                &mut r,
                name,
                arch.levels.get(index).copied().unwrap_or(0),
                ti,
                Some(&encoding.component_widths),
                (simulation_complete, None),
            )? {
                return None;
            }
            count += 1;
            if encoding.corruption_check {
                r.gate("component.corruption_check", 32, true)?;
            }
        }
        (r.cursor.position <= (data.len() * 8) as i64).then_some((r.cursor.position, count))
    })();
    (result, r.diagnostics)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedReplicationFrame {
    pub configuration: bool,
    pub messages: DecodedFrameView,
    pub entities: Option<DecodedEntityView>,
    pub controls: Option<DecodedFrameView>,
    pub end_bit: i64,
}

/// Compose the frame configuration bit and views A, B and C, stopping before
/// downstream views whenever an earlier view cannot reach its terminator.
pub fn decode_replication_frame(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    bindings: &mut EntityBindings,
) -> Option<DecodedReplicationFrame> {
    if !encoding.valid() {
        return None;
    }
    let configuration = Cursor::new(data, 0)?.bit()?;
    let messages = decode_message_view(data, 1);
    let mut frame = DecodedReplicationFrame {
        configuration,
        end_bit: messages.end_bit,
        messages,
        entities: None,
        controls: None,
    };
    if frame.messages.stop != FrameViewStop::Complete {
        return Some(frame);
    }
    let entities = decode_entity_view(
        data,
        usize::try_from(frame.end_bit).ok()?,
        registry,
        encoding,
        bindings,
    );
    frame.end_bit = entities.end_bit;
    let complete = entities.stop == EntityViewStop::Complete;
    frame.entities = Some(entities);
    if complete {
        let controls = super::views::decode_control_view_contextual(
            data,
            frame.end_bit,
            encoding.position.as_ref(),
            None,
        );
        frame.end_bit = controls.end_bit;
        frame.controls = Some(controls);
    }
    Some(frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::FilmArchetype;
    use std::io::Read;

    fn registry() -> FilmRegistry {
        let mut reg = FilmRegistry {
            archetypes: (0..4)
                .map(|index| FilmArchetype {
                    index,
                    components: Vec::new(),
                    levels: Vec::new(),
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        reg.archetypes[3].components = vec!["high-frequency".into()];
        reg.archetypes[3].levels = vec![1];
        reg
    }

    #[test]
    fn record_chains_bind_reuse_and_stop_like_reference() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("../fixtures/frames-levelup-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 32);
        for case in cases {
            let hex = case["hex"].as_str().unwrap();
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 5,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: case["extra"] == true,
                corruption_check: case["check"] == true,
            };
            if !encoding.extra_fields {
                let mut world = crate::theater::FilmWorld::default();
                let production = crate::theater::decode_production_frame(
                    &data,
                    2,
                    &registry(),
                    &encoding,
                    &mut world,
                )
                .unwrap();
                assert_eq!(production.end_bit, case["production_end"], "{case}");
                assert_eq!(
                    production.views_completed, case["production_views"],
                    "{case}"
                );
                let expected = case["production"].as_array().unwrap();
                assert_eq!(production.records.len(), expected.len());
                for (record, expected) in production.records.iter().zip(expected) {
                    assert_eq!(
                        record.header.id,
                        Some(expected["ID"].as_u64().unwrap() as u32)
                    );
                    if record.header.kind != RecordKind::Delete {
                        assert_eq!(record.end_bit, expected["Trace"]["EndBit"]);
                    }
                }
                if !encoding.extra_fields {
                    assert_eq!(world.slots[&1].full_id, 0x80000001);
                }
            }
            for simulation in case["simulations"].as_array().unwrap() {
                let mut sim_registry = registry();
                sim_registry.archetypes[3].components = vec!["simulation-state".into()];
                let mut sim_encoding = encoding.clone();
                sim_encoding.position = crate::theater::native_keyframe_queue_encoding().position;
                sim_encoding.keyframe_simulation_complete =
                    Some(simulation["policy"].as_bool().unwrap());
                let mut reader = crate::theater::NativeFilmBits::new(&data);
                let view = decode_native_frame_records(
                    &mut reader,
                    &sim_registry,
                    &sim_encoding,
                    &mut EntityBindings::default(),
                    true,
                    2,
                )
                .unwrap();
                assert_eq!(
                    reader.position(),
                    simulation["end"],
                    "simulation {simulation}"
                );
                assert_eq!(
                    view.stop == EntityViewStop::Complete,
                    simulation["complete"]
                );
                let records: Vec<_> = view
                    .records
                    .iter()
                    .filter(|r| r.header.kind != RecordKind::End)
                    .collect();
                let expected = simulation["records"].as_array().unwrap();
                assert_eq!(records.len(), expected.len());
                for (r, e) in records.iter().zip(expected) {
                    assert_eq!(r.header.id, Some(e["ID"].as_u64().unwrap() as u32));
                    if r.header.kind != RecordKind::Delete {
                        assert_eq!(r.end_bit, e["Trace"]["EndBit"]);
                    }
                }
                if simulation["policy"] == false {
                    sim_encoding.keyframe_simulation_complete = None;
                    let default = decode_native_entity_view(
                        &data,
                        2,
                        &sim_registry,
                        &sim_encoding,
                        &mut EntityBindings::default(),
                        true,
                    );
                    assert_eq!(default, view);
                }
            }
            for reader_case in case["readers"].as_array().unwrap() {
                let mut reader = crate::theater::NativeFilmBits::new(&data);
                reader.set_position(reader_case["start"].as_i64().unwrap());
                let view = decode_native_frame_records(
                    &mut reader,
                    &registry(),
                    &encoding,
                    &mut EntityBindings::default(),
                    true,
                    reader_case["preamble"].as_i64().unwrap(),
                )
                .unwrap();
                assert_eq!(
                    reader.position(),
                    reader_case["end"],
                    "reader {reader_case}"
                );
                assert_eq!(view.end_bit, reader.position());
                assert_eq!(
                    view.stop == EntityViewStop::Complete,
                    reader_case["complete"]
                );
                let records: Vec<_> = view
                    .records
                    .iter()
                    .filter(|r| r.header.kind != RecordKind::End)
                    .collect();
                let expected = reader_case["records"].as_array().unwrap();
                assert_eq!(records.len(), expected.len());
                for (r, e) in records.iter().zip(expected) {
                    assert_eq!(r.header.id, Some(e["ID"].as_u64().unwrap() as u32));
                    assert_eq!(
                        r.header.start_bit - if encoding.extra_fields { 32 } else { 0 },
                        e["HeaderBit"]
                    );
                    if r.header.kind != RecordKind::Delete {
                        assert_eq!(r.end_bit, e["Trace"]["EndBit"]);
                    }
                }
            }
            for wildcard in case["wildcards"].as_array().unwrap() {
                let mut bindings = EntityBindings::default();
                match wildcard["mode"].as_u64().unwrap() {
                    1 => bindings.bind(0x40000001, 3),
                    2 => bindings.bind(0x80000001, 3),
                    3 => bindings.bind_wildcard(0xc0000001, 3),
                    _ => {}
                }
                let serialized = serde_json::to_value(&bindings).unwrap();
                assert_eq!(
                    serde_json::from_value::<EntityBindings>(serialized).unwrap(),
                    bindings
                );
                let decoded = decode_native_entity_view(
                    &data,
                    wildcard["start"].as_u64().unwrap() as usize,
                    &registry(),
                    &encoding,
                    &mut bindings,
                    wildcard["strict"].as_bool().unwrap(),
                );
                assert_eq!(decoded.end_bit, wildcard["end"], "wildcard {wildcard}");
                assert_eq!(
                    decoded.stop == EntityViewStop::Complete,
                    wildcard["complete"]
                );
                assert_eq!(bindings.slots.contains_key(&1), wildcard["exists"]);
                if let Some(binding) = bindings.slots.get(&1) {
                    assert_eq!(binding.id, wildcard["id"]);
                    assert_eq!(binding.generation_any, wildcard["generation_any"]);
                }
                let records: Vec<_> = decoded
                    .records
                    .iter()
                    .filter(|r| r.header.kind != RecordKind::End)
                    .collect();
                let expected = wildcard["records"].as_array().unwrap();
                assert_eq!(records.len(), expected.len());
                for (r, e) in records.iter().zip(expected) {
                    assert_eq!(r.header.id, Some(e["ID"].as_u64().unwrap() as u32));
                    if r.header.kind != RecordKind::Delete {
                        assert_eq!(r.end_bit, e["Trace"]["EndBit"]);
                    }
                }
            }
            for tail in case["tails"].as_array().unwrap() {
                let cut = tail["cut"].as_u64().unwrap() as usize;
                let native = decode_native_entity_view(
                    &data[..cut],
                    2,
                    &registry(),
                    &encoding,
                    &mut EntityBindings::default(),
                    tail["strict"].as_bool().unwrap(),
                );
                assert_eq!(native.end_bit, tail["end"], "tail {tail}");
                assert_eq!(
                    native.stop == EntityViewStop::Complete,
                    tail["complete"],
                    "tail {tail}"
                );
                let records: Vec<_> = native
                    .records
                    .iter()
                    .filter(|r| r.header.kind != RecordKind::End)
                    .collect();
                let expected = tail["records"].as_array().unwrap();
                assert_eq!(records.len(), expected.len(), "tail {tail}");
                for (r, e) in records.iter().zip(expected) {
                    assert_eq!(r.header.id, Some(e["ID"].as_u64().unwrap() as u32));
                    assert_eq!(
                        r.header.start_bit - if encoding.extra_fields { 32 } else { 0 },
                        e["HeaderBit"]
                    );
                    if r.header.kind != RecordKind::Delete {
                        assert_eq!(r.end_bit, e["Trace"]["EndBit"], "tail {tail}");
                    }
                }
                for r in &native.records {
                    assert_eq!(
                        r.padded_bits,
                        crate::theater::bits::padded_from_native(r.end_bit, cut * 8)
                    );
                }
            }
            let mut relaxed_bindings = EntityBindings::default();
            let relaxed = decode_entity_view_with_generation_policy(
                &data,
                2,
                &registry(),
                &encoding,
                &mut relaxed_bindings,
                false,
            );
            assert_eq!(relaxed.end_bit, case["relaxed_end"]);
            assert_eq!(
                relaxed.stop == EntityViewStop::Complete,
                case["relaxed_complete"]
            );
            assert_eq!(relaxed_bindings.slots[&1].id, 0x80000001);
            let relaxed_records: Vec<_> = relaxed
                .records
                .iter()
                .filter(|r| r.header.kind != RecordKind::End)
                .collect();
            let expected_relaxed = case["relaxed_records"].as_array().unwrap();
            assert_eq!(relaxed_records.len(), expected_relaxed.len());
            for (record, expected) in relaxed_records.iter().zip(expected_relaxed) {
                assert_eq!(
                    record.header.id,
                    Some(expected["ID"].as_u64().unwrap() as u32)
                );
                assert_eq!(
                    record.header.start_bit - if encoding.extra_fields { 32 } else { 0 },
                    expected["HeaderBit"]
                );
                if record.header.kind != RecordKind::Delete {
                    assert_eq!(record.end_bit, expected["Trace"]["EndBit"]);
                    assert_eq!(
                        record.archetype,
                        Some(expected["TypeIndex"].as_u64().unwrap() as u32)
                    );
                }
            }
            let mut bindings = EntityBindings::default();
            let frame =
                decode_replication_frame(&data, &registry(), &encoding, &mut bindings).unwrap();
            let view = frame.entities.unwrap();
            assert_eq!(view.end_bit, case["end_bit"], "{case}");
            assert_eq!(view.stop == EntityViewStop::Complete, case["complete"]);
            assert_eq!(bindings.slots[&1].id, 0x80000001);
            assert_eq!(bindings.slots[&1].archetype, 3);
            let expected = case["records"].as_array().unwrap();
            let actual: Vec<_> = view
                .records
                .iter()
                .filter(|r| r.header.kind != RecordKind::End)
                .collect();
            assert_eq!(actual.len(), expected.len());
            for (record, expected) in actual.iter().zip(expected) {
                assert_eq!(
                    record.header.id,
                    Some(expected["ID"].as_u64().unwrap() as u32)
                );
                let prefix = if encoding.extra_fields { 32 } else { 0 };
                assert_eq!(record.header.start_bit - prefix, expected["HeaderBit"]);
                if record.header.kind != RecordKind::Delete {
                    assert_eq!(record.end_bit, expected["Trace"]["EndBit"]);
                }
            }
            let delete = &actual[2];
            assert_eq!(
                delete
                    .fields
                    .iter()
                    .find(|f| f.name == "delete.word")
                    .unwrap()
                    .raw,
                0x91827364
            );
            if case["complete"] == true {
                assert_eq!(frame.controls.unwrap().stop, FrameViewStop::Complete);
            } else {
                assert!(matches!(
                    view.stop,
                    EntityViewStop::GenerationMismatch { .. }
                ));
                assert!(frame.controls.is_none());
            }
            // Truncation during a NEW must not install a binding from its header.
            let first = &actual[0];
            let mut empty = EntityBindings::default();
            let short = decode_replication_frame(
                &data[..crate::theater::bits::native_address((first.end_bit - 1) / 8)],
                &registry(),
                &encoding,
                &mut empty,
            )
            .unwrap();
            assert_eq!(short.entities.unwrap().stop, EntityViewStop::Truncated);
            assert!(empty.slots.is_empty());
        }
    }
}
