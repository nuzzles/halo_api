//! Sequential event-list grammar from the pinned reference killsource decoder.
//! Unknown runtime settings stop decoding without speculative localization.
use crate::theater::parser::v41::chunks::replication::source::replication_stream::models::kill_event_chain::{
    EventAssumption, EventField, EventFieldStage, EventFieldValue, EventListRead, EventListStop,
    EventRecord,
};
use crate::theater::parser::bits::Cursor;
use crate::theater::parser::v41::context::events::{EventVersionContext, VariantContext};
use crate::theater::parser::v41::context::movement::PrecisionDescriptor;
mod actions;
mod fire;
mod projectile;
#[cfg(test)]
mod regions;
mod simple;
#[cfg(test)]
mod teleport_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod zero_axes;
pub(crate) use projectile::ProjectilePositionEncoding;
#[cfg(test)]
pub(crate) use projectile::detonation_position;
#[cfg(test)]
pub(crate) use projectile::detonation_region_position;

const CONFIG: [[i8; 3]; 123] = [
    [1, 1, 7], // 0
    [1, 8, 7], // 1
    [1, 8, 7], // 2
    [0, 8, 7], // 3
    [0, 8, 7], // 4
    [5, 8, 7], // 5
    [5, 8, 7], // 6
    [1, 8, 7], // 7
    [2, 3, 7], // 8
    [2, 8, 7], // 9
    [8, 8, 7], // 10
    [2, 8, 7], // 11
    [8, 8, 7], // 12
    [4, 8, 7], // 13
    [2, 2, 7], // 14
    [8, 8, 7], // 15
    [8, 8, 7], // 16
    [6, 6, 7], // 17
    [6, 8, 7], // 18
    [0, 8, 7], // 19
    [1, 1, 7], // 20
    [4, 8, 7], // 21
    [1, 1, 7], // 22
    [0, 8, 7], // 23
    [0, 2, 7], // 24
    [0, 2, 7], // 25
    [0, 2, 7], // 26
    [0, 0, 7], // 27
    [0, 8, 7], // 28
    [6, 8, 7], // 29
    [4, 8, 7], // 30
    [1, 8, 7], // 31
    [1, 8, 7], // 32
    [3, 4, 7], // 33
    [8, 8, 7], // 34
    [1, 8, 7], // 35
    [1, 8, 7], // 36
    [1, 8, 7], // 37
    [2, 8, 7], // 38
    [2, 8, 7], // 39
    [2, 0, 7], // 40
    [3, 8, 7], // 41
    [2, 8, 7], // 42
    [2, 0, 7], // 43
    [2, 0, 7], // 44
    [2, 8, 7], // 45
    [4, 8, 7], // 46
    [4, 8, 7], // 47
    [4, 8, 7], // 48
    [3, 2, 7], // 49
    [4, 4, 7], // 50
    [2, 8, 7], // 51
    [2, 0, 7], // 52
    [1, 1, 7], // 53
    [1, 1, 7], // 54
    [6, 8, 7], // 55
    [5, 1, 7], // 56
    [2, 0, 7], // 57
    [1, 8, 7], // 58
    [0, 8, 7], // 59
    [0, 8, 7], // 60
    [0, 8, 7], // 61
    [6, 8, 7], // 62
    [2, 8, 7], // 63
    [6, 8, 7], // 64
    [3, 8, 7], // 65
    [6, 8, 7], // 66
    [6, 8, 7], // 67
    [6, 8, 7], // 68
    [0, 8, 7], // 69
    [2, 8, 7], // 70
    [2, 8, 7], // 71
    [2, 8, 7], // 72
    [2, 8, 7], // 73
    [2, 8, 7], // 74
    [1, 8, 7], // 75
    [1, 8, 7], // 76
    [6, 8, 7], // 77
    [4, 8, 7], // 78
    [4, 0, 7], // 79
    [8, 8, 7], // 80
    [0, 8, 7], // 81
    [0, 8, 7], // 82
    [0, 8, 7], // 83
    [0, 8, 7], // 84
    [8, 8, 7], // 85
    [1, 8, 7], // 86
    [6, 8, 7], // 87
    [6, 8, 7], // 88
    [6, 8, 7], // 89
    [6, 8, 7], // 90
    [6, 8, 7], // 91
    [6, 8, 7], // 92
    [2, 0, 7], // 93
    [6, 8, 7], // 94
    [6, 8, 7], // 95
    [6, 8, 7], // 96
    [6, 8, 7], // 97
    [1, 1, 7], // 98
    [8, 8, 7], // 99
    [1, 8, 7], // 100
    [6, 8, 7], // 101
    [2, 8, 7], // 102
    [0, 0, 7], // 103
    [0, 0, 7], // 104
    [0, 0, 7], // 105
    [0, 0, 7], // 106
    [0, 8, 7], // 107
    [6, 8, 7], // 108
    [0, 0, 7], // 109
    [1, 8, 7], // 110
    [6, 8, 7], // 111
    [6, 8, 7], // 112
    [6, 8, 7], // 113
    [6, 8, 7], // 114
    [1, 0, 7], // 115
    [1, 0, 7], // 116
    [2, 0, 7], // 117
    [1, 8, 7], // 118
    [2, 0, 7], // 119
    [8, 8, 7], // 120
    [8, 8, 7], // 121
    [8, 8, 7], // 122
];

/// Recorded reference domains for the v41 event grammar. Runtime projections
/// use this same table without rereading or searching packet bytes.
pub(crate) fn reference_kinds(code: u8) -> Option<[i8; 3]> {
    CONFIG.get(usize::from(code)).copied()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct KillEventFields {
    pub killer: i32,
    pub victim: i32,
    pub assist: i32,
    /// Raw integer percentage; values above 100 are retained.
    pub killer_pct: u32,
    /// Meaningful as a damage share only when assist is present.
    pub assist_pct: u32,
    /// Unresolved one-bit field, retained without interpretation.
    pub flag: u8,
    /// -1 means a mandatory field exceeded the packet bounds.
    pub end: i64,
}

/// Walk an event list from an established continuation bit. This reuses the
/// pinned killsource event *layout* grammar, without localization, filtering or
/// roster inference. Unlike the reference scanner's selected runtime fallback,
/// an unspecified code-15 gate stops before its body. Fixed-size skipped bodies
/// are retained as opaque ranges, not advertised as interpreted fields.
#[cfg(test)]
pub(crate) fn read_reference_event_list(
    data: &[u8],
    start_bit: usize,
    gate15: Option<bool>,
    limit: usize,
) -> EventListRead {
    read_contextual_event_list(data, start_bit, gate15, limit, None, None, None)
}

/// Explicit v3 layout admission, independent of optional map position context.
#[derive(Clone, Copy)]
pub(crate) struct FireV3<'a> {
    pub position: Option<&'a PrecisionDescriptor>,
}

impl<'a> From<&'a PrecisionDescriptor> for FireV3<'a> {
    fn from(position: &'a PrecisionDescriptor) -> Self {
        Self {
            position: Some(position),
        }
    }
}

/// The optional descriptor supplies quantized map-vector context. `fire_v3`
/// separately admits the v3 fire layout with or without map widths. Neither
/// layout nor region is inferred from following bytes.
#[cfg(test)]
pub(crate) fn read_contextual_event_list(
    data: &[u8],
    start_bit: usize,
    gate15: Option<bool>,
    limit: usize,
    event_vector: Option<&PrecisionDescriptor>,
    fire_v3: Option<FireV3<'_>>,
    projectile_positions: Option<&ProjectilePositionEncoding>,
) -> EventListRead {
    // Historical reference fixtures use the original default assumptions.
    // Production callers must supply an explicit EventLayout to read_event_list.
    read_event_list(
        data,
        start_bit,
        limit,
        EventLayout {
            versions: EventVersionContext::Native,
            category_references: true,
            gate15,
            script_prefix_context: None,
            event_vector,
            fire_v3,
            projectile_positions,
            variant: VariantContext::default(),
            runtime_defaults: true,
            kill_tail: None,
            corruption_check: Some(false),
        },
    )
}

pub(crate) struct EventLayout<'a> {
    pub versions: EventVersionContext,
    pub category_references: bool,
    pub gate15: Option<bool>,
    pub script_prefix_context: Option<bool>,
    pub event_vector: Option<&'a PrecisionDescriptor>,
    pub fire_v3: Option<FireV3<'a>>,
    pub projectile_positions: Option<&'a ProjectilePositionEncoding>,
    pub variant: VariantContext,
    pub runtime_defaults: bool,
    pub kill_tail: Option<bool>,
    pub corruption_check: Option<bool>,
}

impl Default for EventLayout<'_> {
    fn default() -> Self {
        Self {
            versions: EventVersionContext::Native,
            category_references: true,
            gate15: None,
            script_prefix_context: None,
            event_vector: None,
            fire_v3: None,
            projectile_positions: None,
            variant: VariantContext::default(),
            runtime_defaults: false,
            kill_tail: None,
            // Explicit internal fixture/profile context. Film always supplies
            // its recorded policy, including None when it is unavailable.
            corruption_check: Some(false),
        }
    }
}

pub(crate) fn read_event_list(
    data: &[u8],
    start_bit: usize,
    limit: usize,
    layout: EventLayout<'_>,
) -> EventListRead {
    let mut out = EventListRead {
        start_bit,
        end_bit: start_bit,
        gate15: layout.gate15,
        script_prefix_context_used: None,
        assumptions: Vec::new(),
        kill_tail_context_used: None,
        records: Vec::new(),
        terminator: None,
        stop: EventListStop::InvalidStart,
    };
    let Some(mut r) = Reader::new(data, start_bit) else {
        return out;
    };
    r.trace = Some(Vec::new());
    loop {
        if out.records.len() >= limit {
            out.stop = EventListStop::RecordLimit;
            break;
        }
        if r.cursor.position >= data.len() * 8 {
            out.stop = EventListStop::SourceBoundary;
            break;
        }
        let start = r.cursor.position;
        debug_assert!(r.trace.as_ref().unwrap().is_empty());
        r.stage = EventFieldStage::Header;
        if r.read(1) == 0 {
            out.terminator = r.trace.as_mut().unwrap().pop();
            out.stop = if r.over {
                EventListStop::Truncated
            } else {
                EventListStop::Terminator
            };
            break;
        }
        let code = r.read(7) as usize;
        let mut record = EventRecord {
            start_bit: start,
            end_bit: r.cursor.position,
            code: (!r.over).then_some(code as u8),
            body_start_bit: None,
            fields: Vec::new(),
            layout_complete: false,
        };
        let stop = if r.over {
            Some(EventListStop::Truncated)
        } else if code >= CONFIG.len() {
            Some(EventListStop::UnsupportedCode)
        } else if let Some(stop) = layout.versions.stop() {
            Some(stop)
        } else if !layout.category_references {
            Some(EventListStop::UnsupportedReferenceConfiguration)
        } else {
            r.stage = EventFieldStage::References;
            if !presence(&mut r, code) {
                Some(if r.over {
                    EventListStop::Truncated
                } else {
                    EventListStop::UnsupportedReferences
                })
            } else {
                record.body_start_bit = Some(r.cursor.position);
                r.stage = EventFieldStage::Body;
                if code == 15 {
                    out.script_prefix_context_used = layout.script_prefix_context;
                }
                if code == 15 && layout.gate15.is_none() {
                    Some(EventListStop::MissingRuntimeGate15)
                } else if body(&mut r, data, code, &layout) {
                    // Native lireUnMessage returns before this guard for the
                    // thirteen descriptors whose structure size is zero.
                    if !empty_body(code) && layout.corruption_check.is_none() {
                        Some(EventListStop::MissingCorruptionCheckContext)
                    } else {
                        if layout.corruption_check == Some(true) && !empty_body(code) {
                            r.stage = EventFieldStage::CorruptionCheck;
                            if r.read(1) != 0 {
                                r.read(32);
                            }
                        }
                        record.layout_complete = !r.over;
                        r.over.then_some(EventListStop::Truncated)
                    }
                } else {
                    Some(if r.over {
                        EventListStop::Truncated
                    } else {
                        r.context_stop
                            .clone()
                            .unwrap_or(EventListStop::UnsupportedBody)
                    })
                }
            }
        };
        record.fields = std::mem::take(r.trace.as_mut().unwrap());
        record.end_bit = record
            .fields
            .iter()
            .filter(|field| !matches!(field.value, EventFieldValue::Unavailable))
            .filter_map(|field| field.bit.checked_add(field.width))
            .max()
            .unwrap_or(record.start_bit);
        out.records.push(record);
        if let Some(stop) = stop {
            out.stop = stop;
            break;
        }
    }
    out.end_bit = out
        .records
        .last()
        .map_or(start_bit, |record| record.end_bit)
        .max(
            out.terminator
                .as_ref()
                .map_or(start_bit, |field| field.bit + field.width),
        );
    out.assumptions = r.assumptions;
    out.kill_tail_context_used = r.kill_tail_context_used;
    out
}

// Stop structural field publication at the first failed read. The reference may
// retry shorter reads at that unchanged cursor; those are not established fields
// after the missing preceding word, so canonical decoding does not publish them.
struct Reader<'a> {
    cursor: Cursor<'a>,
    over: bool,
    trace: Option<Vec<EventField>>,
    stage: EventFieldStage,
    context_stop: Option<EventListStop>,
    assumptions: Vec<EventAssumption>,
    kill_tail_context_used: Option<bool>,
}
impl<'a> Reader<'a> {
    fn new(data: &'a [u8], position: usize) -> Option<Self> {
        Some(Self {
            cursor: Cursor::new(data, position)?,
            over: false,
            trace: None,
            stage: EventFieldStage::Body,
            context_stop: None,
            assumptions: Vec::new(),
            kill_tail_context_used: None,
        })
    }
    fn read(&mut self, n: usize) -> u64 {
        if self.over {
            return 0;
        }
        // Reference wide reads consume all requested bits, even beyond 64 bits.
        if n > 64 {
            self.skip(n);
            return 0;
        }
        let bit = self.cursor.position;
        let value = self.cursor.read(n);
        if let Some(trace) = &mut self.trace {
            trace.push(EventField {
                bit,
                width: n,
                stage: self.stage,
                value: value.map_or(EventFieldValue::Unavailable, EventFieldValue::Scalar),
            });
        }
        value.unwrap_or_else(|| {
            self.over = true;
            0
        })
    }
    fn skip(&mut self, n: usize) {
        if self.over {
            return;
        }
        let bit = self.cursor.position;
        let ok = self.cursor.skip(n).is_some();
        if let Some(trace) = &mut self.trace {
            trace.push(EventField {
                bit,
                width: n,
                stage: self.stage,
                value: if ok {
                    EventFieldValue::Opaque
                } else {
                    EventFieldValue::Unavailable
                },
            });
        }
        if !ok {
            self.over = true;
        }
    }
    fn entity5(&mut self) -> i32 {
        if self.read(1) != 0 {
            -1
        } else {
            self.read(5) as i32
        }
    }
}

fn kill_fields(r: &mut Reader<'_>) -> KillEventFields {
    let victim = r.entity5();
    let killer = r.entity5();
    let killer_pct = r.read(32) as u32;
    let flag = r.read(1) as u8;
    let assist = r.entity5();
    let assist_pct = r.read(32) as u32;
    KillEventFields {
        killer,
        victim,
        assist,
        killer_pct,
        assist_pct,
        flag,
        end: if r.over { -1 } else { r.cursor.position as i64 },
    }
}
fn presence(r: &mut Reader<'_>, code: usize) -> bool {
    const RANGE: [u32; 10] = [7679, 7679, 256, 256, 512, 256, 512, 8191, 8191, 0];
    for mut cfg in CONFIG[code] {
        if r.read(1) == 0 {
            continue;
        }
        if cfg < 0 {
            return false;
        }
        if cfg == 1 && r.read(1) == 1 {
            cfg = 4;
        }
        let range = RANGE[cfg as usize];
        if range > 0 {
            r.read((32 - (range - 1).leading_zeros()) as usize);
        }
        r.read(2);
    }
    !r.over
}
fn empty_body(code: usize) -> bool {
    matches!(
        code,
        3 | 4 | 23 | 24 | 25 | 26 | 33 | 49 | 54 | 57 | 59 | 92 | 103
    )
}

fn body(r: &mut Reader<'_>, data: &[u8], code: usize, layout: &EventLayout<'_>) -> bool {
    let EventLayout {
        event_vector,
        fire_v3,
        projectile_positions,
        ..
    } = *layout;
    if let Some(complete) = simple::read(r, code) {
        return complete;
    }
    let fixed = match code {
        code if empty_body(code) => Some(0),
        34 => Some(59),
        _ => None,
    };
    if let Some(n) = fixed {
        r.skip(n);
        return !r.over;
    }
    match code {
        12 => {
            // biped_melee_clang, FUN_142ef8e08: two optional players, tag,
            // precision-12 point and direction. This is not a fixed-size body.
            for _ in 0..2 {
                if r.read(1) == 0 {
                    r.read(5);
                }
            }
            if r.read(1) != 0 {
                r.read(32);
            }
            if read_event_point(r, projectile_positions.map(|p| &p.impact), 18).is_none() {
                return false;
            }
            r.read(19);
        }
        80 => {
            // networked_ai_effect, FUN_142ef8f74 -> FUN_142ef15e0.
            if r.read(1) != 0 {
                r.read(32);
            }
            let points = match r.read(2) {
                0 => 1,
                1 => 3,
                _ => 0,
            };
            for _ in 0..points {
                if !event_point(r, event_vector) {
                    return false;
                }
            }
        }
        106 => {
            // ObjectCollisionDamage, FUN_14112134c: precision-12 point.
            if read_event_point(r, projectile_positions.map(|p| &p.impact), 18).is_none() {
                return false;
            }
            r.read(10);
            r.read(9);
        }
        102 => {
            let variant = r.read(5);
            return actions::read(r, data, variant);
        }
        32 => {
            // unit_teleported: positions precede the two independent tag gates.
            if !event_point(r, event_vector) || !event_point(r, event_vector) {
                return false;
            }
            for _ in 0..2 {
                if r.read(1) != 0 {
                    r.read(32);
                }
            }
        }
        64 => {
            r.read(6);
            if !event_point(r, event_vector) {
                return false;
            }
            for _ in 0..3 {
                r.read(32);
            }
            r.read(1);
        }
        78 => {
            if !event_point(r, event_vector) {
                return false;
            }
            if r.read(1) == 0 {
                r.read(19);
                r.read(10);
            }
            r.read(2);
        }
        116 => {
            if r.read(1) != 0 {
                let (direction_width, roll_width) = match layout.variant.engine_type {
                    // FUN_140c5f938(mode 0) selects FUN_142e29bac for engine 1.
                    Some(1) => (30, 30),
                    Some(_) => (19, 8),
                    None if layout.runtime_defaults => {
                        if !r
                            .assumptions
                            .contains(&EventAssumption::TeleportEngineNotOne)
                        {
                            r.assumptions.push(EventAssumption::TeleportEngineNotOne);
                        }
                        (19, 8)
                    }
                    None => {
                        r.context_stop = Some(EventListStop::MissingVariantContext);
                        return false;
                    }
                };
                if r.read(1) == 0 {
                    r.read(direction_width);
                }
                r.read(roll_width);
            }
            r.read(1);
            if r.read(1) != 0 {
                r.read(32);
                if !event_point(r, event_vector) || !event_point(r, event_vector) {
                    return false;
                }
            }
        }
        5 | 6 => {
            return projectile::read(r, code, projectile_positions).is_some();
        }
        39 => {
            // biped_throw_initiate: reference analysis chargeLancerInitie, FUN_140c6a58c.
            if r.read(1) != 0 {
                if r.read(1) != 0 {
                    r.read(32);
                }
                r.read(4);
            } else {
                r.read(3);
            }
            if r.read(1) == 0 {
                r.read(5);
            }
        }
        75 => {
            // AIDialog: reference analysis chargeDialogueIA, FUN_140f2e634.
            r.read(5);
            if r.read(1) != 0 {
                r.read(32);
            }
            if r.read(1) == 0 {
                r.read(12);
            }
            if r.read(1) == 0 {
                r.read(5);
            }
            let count = r.read(1) as usize + 1;
            r.skip(count);
        }
        117 => {
            // EquipmentTranslocatorTeleportEffects: guarded effect and two map positions.
            if r.read(1) != 0 {
                r.read(32);
            }
            if !event_point(r, event_vector) || !event_point(r, event_vector) {
                return false;
            }
        }
        36 => {
            let Some(encoding) = fire_v3 else {
                return false;
            };
            // This layout uses category-relative references (configuration=1).
            if data.first().is_none_or(|b| b & 0x80 == 0) {
                return false;
            }
            return fire::read(r, encoding.position);
        }
        85 => {
            let mut nested = Reader::new(data, r.cursor.position).unwrap();
            nested.trace = r.trace.as_ref().map(|_| Vec::new());
            let k = kill_fields(&mut nested);
            if let (Some(trace), Some(fields)) = (&mut r.trace, nested.trace) {
                trace.extend(fields);
            }
            if k.end < 0 {
                r.over = true;
                return false;
            }
            r.cursor.position = k.end as usize;
            if layout.variant.excludes_kill_tail() {
                if layout.kill_tail == Some(true) {
                    r.context_stop = Some(EventListStop::ContradictoryKillPlaybackContext);
                    return false;
                }
            } else if let Some(present) = layout.kill_tail {
                r.kill_tail_context_used = Some(present);
                if present {
                    r.read(32);
                    r.read(32);
                    r.read(4);
                }
            } else {
                if !layout.runtime_defaults {
                    r.context_stop = Some(EventListStop::MissingKillPlaybackContext);
                    return false;
                }
                if !r
                    .assumptions
                    .contains(&EventAssumption::KillPlaybackDisabled)
                {
                    r.assumptions.push(EventAssumption::KillPlaybackDisabled);
                }
            }
        }
        1 => {
            r.read(5);
            if r.read(1) == 0 {
                r.read(4);
            }
            r.read(3);
            if r.read(1) != 0 {
                r.read(19);
            }
        }
        15 => {
            if layout.gate15 == Some(true) {
                r.read(15);
            }
            r.read(13);
            let n = r.read(10) as usize;
            r.skip(n);
        }
        0 => {
            // FUN_1407f15a4, confirmed by reference analysis chargeDegatsApres at
            // 413971bcc485be53a2fa8cf229e170539a5146f8. Its FUN_1407f2058
            // gate is absent-on-one, unlike the older weapon_hits reader.
            if r.read(1) != 0 {
                r.read(32);
            }
            if r.read(1) == 0 {
                r.read(5);
            }
            r.read(19);
            if r.read(1) != 0 {
                r.read(19);
                r.read(12);
            }
            r.read(5);
            r.read(5);
            r.read(6);
            if r.read(1) != 0 {
                r.read(5);
            }
            // The first fourteen flags are independent source fields. Keep
            // each complete flag when a packet ends inside this sequence.
            for _ in 0..14 {
                r.read(1);
            }
            if r.read(1) != 0 {
                r.read(32);
            }
            r.read(1);
            r.read(3);
            r.read(5);
            r.read(5);
            r.read(1);
            r.read(4);
            if r.read(1) == 0 {
                r.read(10);
            }
            let v58 = r.read(4);
            if r.read(1) != 0 {
                r.read(32);
            }
            if v58 == 1 {
                r.read(8);
            }
            r.read(4);
            if r.read(1) != 0 {
                r.read(13);
                r.read(2);
            }
        }
        82 => {
            r.read(32);
            r.read(8);
            let n = r.read(3);
            for _ in 0..n {
                r.read(32);
                match r.read(3) {
                    0 => {}
                    1 | 2 | 3 | 6 => {
                        r.read(32);
                    }
                    4 => {
                        r.read(1);
                    }
                    5 => {
                        // A bounded C string, not a padded 16-byte buffer.
                        // Preserve the terminator and every recorded byte.
                        for _ in 0..16 {
                            if r.read(8) == 0 || r.over {
                                break;
                            }
                        }
                    }
                    7 => {
                        if !event_point(r, event_vector) {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }
            if r.read(1) == 1 {
                r.read(32);
                let n = r.read(3);
                for _ in 0..n {
                    match r.read(3) {
                        0 => {}
                        1 => {
                            if r.read(1) == 0 {
                                r.read(5);
                            }
                        }
                        2 => {
                            if r.read(1) == 0 {
                                r.read(32);
                            } else {
                                r.read(24);
                            }
                        }
                        _ => {
                            r.read(32);
                        }
                    }
                }
            }
            r.read(32);
        }
        _ => return false,
    }
    !r.over
}

/// Full-precision event point. A default-box point has fixed wire widths and
/// needs no map bounds; only the indexed branch depends on external context.
/// Consuming a point does not imply that its codes are coordinates in our map.
fn event_point(r: &mut Reader<'_>, encoding: Option<&PrecisionDescriptor>) -> bool {
    read_event_point(r, encoding, 22).is_some()
}

/// Source encoding of a successfully consumed point. Default-box source codes
/// stay in the event fields; they cannot be interpreted using map-region bounds.
enum EventPoint {
    DefaultBox,
    Map { region: u32, codes: [u32; 3] },
}

fn read_event_point(
    r: &mut Reader<'_>,
    encoding: Option<&PrecisionDescriptor>,
    default_width: usize,
) -> Option<EventPoint> {
    if r.read(1) != 0 {
        for _ in 0..3 {
            r.read(default_width);
        }
        return (!r.over).then_some(EventPoint::DefaultBox);
    }
    let encoding = encoding?;
    if encoding.index_bits > 30
        || (encoding.regions.is_empty() && !encoding.axis_bits.iter().all(|w| (0..=32).contains(w)))
    {
        return None;
    }
    let region = r.read(encoding.index_bits as usize) as u32;
    if r.over {
        return None;
    }
    let widths = encoding.axes_for_region(region)?;
    if !widths.iter().all(|w| (0..=32).contains(w)) {
        return None;
    }
    let codes = widths.map(|width| r.read(width as usize) as u32);
    (!r.over).then_some(EventPoint::Map { region, codes })
}

#[cfg(test)]
mod payload_tests {
    use super::*;

    fn push(bits: &mut Vec<bool>, value: u64, width: usize) {
        bits.extend((0..width).rev().map(|shift| value & (1 << shift) != 0));
    }
    fn bytes(bits: &[bool]) -> Vec<u8> {
        let mut bytes = vec![0u8; bits.len().div_ceil(8)];
        for (i, bit) in bits.iter().enumerate() {
            if *bit {
                bytes[i / 8] |= 1 << (7 - i % 8);
            }
        }
        bytes
    }
    fn header(code: u64) -> Vec<bool> {
        let mut bits = vec![false];
        push(&mut bits, 1, 1);
        push(&mut bits, code, 7);
        bits
    }

    #[test]
    fn throw_initiate_consumes_both_selector_branches_and_optional_fields() {
        for (selector, word_present, player_present) in [
            (false, false, true),
            (true, true, true),
            (true, false, false),
        ] {
            let mut bits = header(39);
            push(&mut bits, 0, 3);
            push(&mut bits, u64::from(selector), 1);
            if selector {
                push(&mut bits, u64::from(word_present), 1);
                if word_present {
                    push(&mut bits, 0x1234_5678, 32);
                }
                push(&mut bits, 9, 4);
            } else {
                push(&mut bits, 5, 3);
            }
            push(&mut bits, u64::from(!player_present), 1);
            if player_present {
                push(&mut bits, 7, 5);
            }
            push(&mut bits, 0, 1);
            let end_bit = bits.len();
            let bytes = bytes(&bits);
            let parsed = read_reference_event_list(&bytes, 1, Some(true), bytes.len());
            assert_eq!(parsed.stop, EventListStop::Terminator);
            assert_eq!(parsed.records.len(), 1);
            assert_eq!(parsed.records[0].code, Some(39));
            assert_eq!(parsed.end_bit, end_bit);
        }
    }

    #[test]
    fn ai_dialogue_uses_conditional_body_width() {
        let mut bits = header(75);
        push(&mut bits, 0, 3);
        push(&mut bits, 17, 5);
        push(&mut bits, 1, 1);
        push(&mut bits, 0x1234_5678, 32);
        push(&mut bits, 0, 1);
        push(&mut bits, 0xabc, 12);
        push(&mut bits, 1, 1);
        push(&mut bits, 1, 1);
        push(&mut bits, 2, 2);
        push(&mut bits, 0, 1);
        let parsed = read_reference_event_list(&bytes(&bits), 1, Some(true), bytes(&bits).len());
        assert_eq!(parsed.stop, EventListStop::Terminator);
        assert_eq!(parsed.records[0].code, Some(75));
        assert_eq!(parsed.end_bit, bits.len());
    }

    #[test]
    fn translocator_positions_require_map_widths() {
        let descriptor = PrecisionDescriptor {
            regions: Default::default(),
            index_bits: 2,
            axis_bits: [12, 12, 11],
            region: 1,
        };
        let mut bits = header(117);
        push(&mut bits, 1, 1);
        push(&mut bits, 7, 8);
        push(&mut bits, 1, 2);
        push(&mut bits, 0, 2);
        push(&mut bits, 1, 1);
        push(&mut bits, 0xa134_4fc2, 32);
        for raw in [[1000, 2000, 500], [2000, 3000, 600]] {
            push(&mut bits, 0, 1);
            push(&mut bits, 1, 2);
            for (axis, width) in raw.into_iter().zip(descriptor.axis_bits) {
                push(&mut bits, axis, width as usize);
            }
        }
        push(&mut bits, 0, 1);
        let payload = bytes(&bits);
        let parsed = read_contextual_event_list(
            &payload,
            1,
            Some(true),
            payload.len(),
            Some(&descriptor),
            None,
            None,
        );
        assert_eq!(parsed.stop, EventListStop::Terminator);
        assert_eq!(parsed.records[0].code, Some(117));
        assert_eq!(parsed.end_bit, bits.len());
        assert_eq!(
            read_reference_event_list(&payload, 1, Some(true), payload.len()).stop,
            EventListStop::UnsupportedBody
        );
    }
}
