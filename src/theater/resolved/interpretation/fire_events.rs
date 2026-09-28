//! Native long fire-event records and modal aim grammar (LevelUp fire_events.go).
//! A fire event does not establish a hit or identify a victim.
use super::bits::{Bits, Cursor};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FilmFireEvent {
    pub chunk: i64,
    pub packet_index: usize,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    /// Recorded five-bit shooter index, or -1 when its gate is closed.
    pub film_index: i32,
    pub has_shooter: bool,
    pub fire_number: u8,
    pub unit: FireUnitReference,
    pub short: bool,
    pub bloc: bool,
    #[serde(rename = "WeaponID")]
    pub weapon_id: u64,
    pub has_aim: bool,
    pub aim: [f32; 3],
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FireUnitReference {
    pub present: bool,
    pub slot: u32,
    #[serde(rename = "Gen")]
    pub generation: u32,
    pub probe: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeFireAimMethod {
    /// Historical exports only. New reads use the grammar-derived modal offset.
    Fixed,
    Modal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeFireAimAttempt {
    pub method: NativeFireAimMethod,
    pub bit: usize,
    pub locator_padding_bits: usize,
    pub raw: Option<u32>,
    pub accepted: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeFireField {
    pub field: super::ComponentField,
    pub opaque: bool,
    pub padded_bits: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeFireHeaderStop {
    Read,
    Truncated,
    OtherHead,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeFireAimStop {
    NoHeader,
    Short,
    TimestampBlock,
    NonModal,
    TruncatedCounts,
    Located,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFireRead {
    pub source_bits: usize,
    pub end_bit: usize,
    pub header_end_bit: Option<usize>,
    pub header_stop: NativeFireHeaderStop,
    pub aim_stop: NativeFireAimStop,
    /// None means a refused/truncated head, never an absent recorded action.
    pub event: Option<FilmFireEvent>,
    /// Ordered header/count/aim fields. Unparsed body bytes stay in the source.
    pub fields: Vec<NativeFireField>,
    pub aim_attempts: Vec<NativeFireAimAttempt>,
}
struct FireReader<'a> {
    cursor: Cursor<'a>,
    source_bits: usize,
    fields: Vec<NativeFireField>,
}
impl FireReader<'_> {
    fn read(&mut self, name: &str, width: usize, opaque: bool) -> u64 {
        let bit = self.cursor.position;
        let raw = self.cursor.read(width).expect("native zero-tail fire read");
        self.fields.push(NativeFireField {
            field: super::ComponentField {
                name: name.into(),
                bit: bit as i64,
                width: width as u64,
                raw,
            },
            opaque,
            padded_bits: self
                .cursor
                .position
                .saturating_sub(bit.max(self.source_bits)),
        });
        raw
    }
    fn bit(&mut self, name: &str) -> bool {
        self.read(name, 1, false) != 0
    }
    fn reference(&mut self, name: &str, probe: bool) -> FireUnitReference {
        if !self.bit(&format!("{name}.present")) {
            return FireUnitReference::default();
        }
        let narrow = probe && self.bit(&format!("{name}.probe"));
        let index = self.read(&format!("{name}.index"), if narrow { 9 } else { 13 }, false) as u32;
        let generation = self.read(&format!("{name}.generation"), 2, false) as u32;
        FireUnitReference {
            present: true,
            slot: 512 + index,
            generation,
            probe: narrow,
        }
    }
}

/// Read the actual optional-field grammar, retaining refusals and zero-tail
/// provenance. The modal aim locator is accepted only within recorded bytes.
pub(crate) fn read_native_fire_event(payload: &[u8]) -> NativeFireRead {
    let mut r = FireReader {
        cursor: Cursor::new_padded(payload, 0),
        source_bits: payload.len() * 8,
        fields: vec![],
    };
    let mut out = NativeFireRead {
        source_bits: r.source_bits,
        end_bit: 0,
        header_end_bit: None,
        header_stop: NativeFireHeaderStop::Truncated,
        aim_stop: NativeFireAimStop::NoHeader,
        event: None,
        fields: vec![],
        aim_attempts: vec![],
    };
    if payload.len() < 2 {
        return out;
    }
    r.bit("header.configuration");
    let more = r.bit("header.continuation");
    let kind = r.read("header.code", 7, false);
    if !more || kind != 36 {
        out.header_stop = NativeFireHeaderStop::OtherHead;
    } else {
        let unit = r.reference("unit", true);
        r.reference("reference1", false);
        r.reference("reference2", false);
        let short = r.bit("head.short");
        let bloc = r.bit("head.bloc");
        let low = r.read("head.fire_number.low", 7, false) as u8;
        let high = r.bit("head.fire_number.high");
        let film_index = if !r.bit("head.shooter.absent") {
            r.read("head.shooter.index", 5, false) as i32
        } else {
            -1
        };
        if !r.bit("head.field_e.absent") {
            r.read("head.field_e", 2, true);
        }
        let upper = if r.bit("head.weapon_upper.present") {
            r.read("head.weapon_upper", 32, false)
        } else {
            0
        };
        let lower = r.read("head.weapon_lower", 32, false);
        r.read("head.flags_ij", 2, true);
        out.header_end_bit = Some(r.cursor.position);
        if r.cursor.position <= r.source_bits {
            out.header_stop = NativeFireHeaderStop::Read;
            let mut event = FilmFireEvent {
                chunk: 0,
                packet_index: 0,
                timestamp_us: 0,
                film_index,
                has_shooter: film_index >= 0,
                fire_number: low | ((high as u8) << 7),
                unit,
                short,
                bloc,
                weapon_id: upper << 32 | lower,
                has_aim: false,
                aim: [0.; 3],
            };
            out.aim_stop = locate_after_header(&mut r, short, bloc);
            if out.aim_stop == NativeFireAimStop::Located {
                let bit = r.cursor.position + 2;
                let raw = Bits(payload).read(bit, 30).map(|v| v as u32);
                let aim = raw.and_then(fire_aim_vector);
                out.aim_attempts.push(NativeFireAimAttempt {
                    method: NativeFireAimMethod::Modal,
                    bit,
                    locator_padding_bits: 0,
                    raw,
                    accepted: aim.is_some(),
                });
                if raw.is_some() {
                    r.read("modal.aim_gap", 2, true);
                    r.read("modal.aim", 30, false);
                }
                if let Some(aim) = aim {
                    event.has_aim = true;
                    event.aim = aim;
                }
            }
            out.event = Some(event);
        }
    }
    out.end_bit = r.cursor.position;
    out.fields = r.fields;
    out
}
fn locate_after_header(r: &mut FireReader<'_>, short: bool, bloc: bool) -> NativeFireAimStop {
    if bloc {
        r.read("body.bloc.flag", 1, true);
        if r.bit("body.bloc.timestamp") {
            return NativeFireAimStop::TimestampBlock;
        }
    }
    if short {
        return NativeFireAimStop::Short;
    }
    let (mut targets, mut components) = (0, 0);
    if !r.bit("body.counts.empty") {
        targets = if r.bit("body.targets.one") {
            1
        } else {
            r.read("body.targets.count", 4, false)
        };
        if !r.bit("body.components.empty") {
            components = if r.bit("body.components.one") {
                1
            } else {
                r.read("body.components.count", 4, false)
            };
        }
    }
    if targets != 0 || components != 0 {
        NativeFireAimStop::NonModal
    } else if r.cursor.position > r.source_bits {
        NativeFireAimStop::TruncatedCounts
    } else {
        NativeFireAimStop::Located
    }
}

// Preserve the pinned native float32 fused arithmetic and float64 square root.
fn fire_aim_vector(code: u32) -> Option<[f32; 3]> {
    let face = code / 178_956_970;
    let rem = code % 178_956_970;
    let step = 2.0_f32 / 13375.;
    let coord = |i: u32| {
        if i * 2 == 13374 {
            0.
        } else {
            (i as f32).mul_add(step, -1.) + step * 0.5
        }
    };
    let (a, b) = (coord(rem / 13376), coord(rem % 13376));
    let v = match face {
        0 => [1., a, b],
        1 => [a, 1., b],
        2 => [a, b, 1.],
        3 => [-1., a, b],
        4 => [a, -1., b],
        5 => [a, b, -1.],
        _ => return None,
    };
    let norm = (v[2].mul_add(v[2], v[1].mul_add(v[1], v[0] * v[0])) as f64).sqrt() as f32;
    Some(if norm < 1e-4 { v } else { v.map(|x| x / norm) })
}
