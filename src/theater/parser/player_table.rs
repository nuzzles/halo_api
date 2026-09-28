//! Bootstrap player table, ported from LevelUp's player_table*.go.
//! See docs/CREDIT.md. Slot ranks include vacant entries.
use super::{
    FilmIdentity,
    bits::{Bits, Cursor},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTableSlot {
    pub film_index: usize,
    pub xuid: u64,
    pub gamertag: String,
    /// Observed 48-bit token; session semantics remain unproven.
    pub session_token: u64,
    pub bit: usize,
    pub total_bits: usize,
    pub shorts: PlayerTableShorts,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTableShorts {
    pub tete: u32,
    pub deux: u32,
    pub repr: u32,
    pub q64: u64,
    pub f10: u32,
    pub f14: u32,
    pub f6: i32,
    pub f8: u32,
    pub f7: u32,
    pub f1: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTableReport {
    pub build: String,
    pub perso_bytes: usize,
    pub profile_delta_bits: i64,
    pub film_delta_bits: i64,
    pub film_delta_gaps: usize,
    pub calibration_agrees: bool,
    pub gaps_agree: usize,
    pub gaps_vacant: usize,
    pub gaps_hidden: usize,
    pub gaps_contradict: usize,
    pub occupied: usize,
    pub vacant: usize,
    pub head_vacant: usize,
    pub interleaved_vacant: bool,
    pub first_record_bit: usize,
    pub candidates_scanned: usize,
    pub candidates_real: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerTableError {
    UnknownBuild,
    Truncated,
    NotFound,
}
/// A failed attempt retains diagnostics but never publishes partial slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTable {
    pub slots: Vec<PlayerTableSlot>,
    pub report: PlayerTableReport,
    pub error: Option<PlayerTableError>,
}
const VACANT_FIXED: usize = 1683;
fn printable(s: &str) -> bool {
    s.len() >= 3 && s.bytes().all(|c| (32..=126).contains(&c))
}
fn vacant(d: &[u8], p: usize) -> bool {
    // Deliberately matches the native prefix predicate: it does not inspect the
    // personalization body or require the entire vacant record in the buffer.
    if p.checked_add(VACANT_FIXED)
        .is_none_or(|end| end > d.len() * 8)
    {
        return false;
    }
    let Some(mut r) = Cursor::new(d, p) else {
        return false;
    };
    [3, 32, 2, 48, 64, 12, 12, 8]
        .into_iter()
        .chain([64; 13])
        .chain([16, 64, 64, 32])
        .all(|n| r.read(n) == Some(0))
}
/// One bounded native read. Opaque spans refer to the retained source bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeSlotField {
    pub name: String,
    pub bit: usize,
    pub bits: usize,
    pub value: NativeSlotValue,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeSlotValue {
    Scalar(u64),
    Opaque,
    /// The requested range was unavailable; the cursor did not advance.
    Unavailable,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePlayerSlotRead {
    pub start_bit: usize,
    pub end_bit: usize,
    pub source_bits: usize,
    pub truncated: bool,
    pub slot: Option<PlayerTableSlot>,
    pub fields: Vec<NativeSlotField>,
}
/// Read an explicitly supplied candidate boundary, without validating its
/// selection or inferring a vacant slot. The returned rank is not resolved.
pub(crate) fn read_native_player_slot(
    data: &[u8],
    start_bit: usize,
    personalization_bits: usize,
) -> NativePlayerSlotRead {
    let mut out = NativePlayerSlotRead {
        start_bit,
        end_bit: start_bit,
        source_bits: data.len() * 8,
        truncated: true,
        slot: None,
        fields: Vec::new(),
    };
    let Some(cursor) = Cursor::new(data, start_bit) else {
        return out;
    };
    let mut reader = SlotReader::new(cursor, true);
    out.slot = slot_read(&mut reader, start_bit, personalization_bits);
    out.end_bit = reader.cursor.position;
    out.truncated = reader.truncated;
    out.fields = reader.fields.unwrap_or_default();
    out
}
fn slot(d: &[u8], p: usize, personalization: usize) -> Option<PlayerTableSlot> {
    slot_read(
        &mut SlotReader::new(Cursor::new(d, p)?, false),
        p,
        personalization,
    )
}
fn slot_read(r: &mut SlotReader<'_>, p: usize, personalization: usize) -> Option<PlayerTableSlot> {
    if r.read("header", 3)? != 4 {
        return None;
    }
    let tete = r.read("tete", 32)? as u32;
    let deux = r.read("deux", 2)? as u32;
    let session_token = r.read("session_token", 48)?;
    let xuid = r.read("xuid", 64)?;
    let (gamertag, mut shorts) = slot_body(r, personalization)?;
    shorts.tete = tete;
    shorts.deux = deux;
    r.skip("trailing", 32)?;
    Some(PlayerTableSlot {
        film_index: 0,
        xuid,
        gamertag,
        session_token,
        bit: p,
        total_bits: r.cursor.position - p,
        shorts,
    })
}
/// The shared bootstrap/type-8 body uses the native sticky bounded reader.
pub(super) struct SlotReader<'a> {
    pub cursor: Cursor<'a>,
    pub truncated: bool,
    pub fields: Option<Vec<NativeSlotField>>,
}
impl<'a> SlotReader<'a> {
    pub(crate) fn new(cursor: Cursor<'a>, trace: bool) -> Self {
        Self {
            cursor,
            truncated: false,
            fields: trace.then(Vec::new),
        }
    }
    fn retain(&mut self, name: &str, bit: usize, bits: usize, value: NativeSlotValue) {
        if let Some(fields) = &mut self.fields {
            fields.push(NativeSlotField {
                name: name.into(),
                bit,
                bits,
                value,
            });
        }
    }
    pub(crate) fn read(&mut self, name: &str, n: usize) -> Option<u64> {
        if self.truncated {
            return None;
        }
        let bit = self.cursor.position;
        let value = self.cursor.read(n);
        self.truncated = value.is_none();
        self.retain(
            name,
            bit,
            n,
            value.map_or(NativeSlotValue::Unavailable, NativeSlotValue::Scalar),
        );
        value
    }
    pub(crate) fn skip(&mut self, name: &str, n: usize) -> Option<()> {
        if self.truncated {
            return None;
        }
        let bit = self.cursor.position;
        let value = self.cursor.skip(n);
        self.truncated = value.is_none();
        self.retain(
            name,
            bit,
            n,
            value.map_or(NativeSlotValue::Unavailable, |_| NativeSlotValue::Opaque),
        );
        value
    }
}
pub(super) fn slot_body(
    r: &mut SlotReader<'_>,
    personalization: usize,
) -> Option<(String, PlayerTableShorts)> {
    let mask = r.read("mask_length_minus_one", 11)? as usize + 1;
    r.skip("mask", mask)?;
    let n = r.read("byte_list_count", 12)? as usize;
    if n > 2048 {
        return None;
    }
    r.skip("byte_list", n * 8)?;
    let m = r.read("word_list_count", 8)? as usize;
    if m > 192 {
        return None;
    }
    r.skip("word_list", m * 32)?;
    r.skip("unknown_104_bytes", 832)?;
    let mut units = Vec::new();
    for _ in 0..16 {
        let u = r.read("name_unit", 16)? as u16;
        if u == 0 {
            break;
        }
        units.push(u);
    }
    let gamertag = String::from_utf16_lossy(&units);
    r.skip("unknown_16_bytes", 128)?;
    let shorts = PlayerTableShorts {
        tete: 0,
        deux: 0,
        repr: r.read("repr", 32)? as u32,
        q64: r.read("q64", 64)?,
        f10: r.read("f10", 10)? as u32,
        f14: r.read("f14", 14)? as u32,
        f6: r.read("f6", 6)? as i32 - 1,
        f8: r.read("f8", 8)? as u32,
        f7: r.read("f7", 7)? as u32,
        f1: r.read("f1", 1)? as u32,
    };
    r.skip("personalization", personalization)?;
    r.skip("unknown_44_bytes", 352)?;
    Some((gamertag, shorts))
}

fn walk(d: &[u8], mut p: usize, perso: usize) -> Option<(Vec<PlayerTableSlot>, usize)> {
    let mut slots = Vec::new();
    let mut vac = 0;
    for rank in 0..32 {
        if vacant(d, p) {
            vac += 1;
            p += VACANT_FIXED + perso;
            continue;
        }
        let mut s = slot(d, p, perso)?;
        if !printable(&s.gamertag) {
            return None;
        }
        s.film_index = rank;
        p += s.total_bits;
        slots.push(s);
    }
    Some((slots, vac))
}
/// Read a decompressed v41 bootstrap with a previously decoded identity.
/// Unknown builds are reported explicitly; no neighboring-build fallback is used.
pub(crate) fn decode_player_table(d: &[u8], identity: &FilmIdentity) -> PlayerTable {
    let mut out = PlayerTable {
        slots: Vec::new(),
        report: PlayerTableReport {
            build: identity.build.clone(),
            ..Default::default()
        },
        error: None,
    };
    let perso = match identity.build.as_str() {
        "HI_1_12_0" | "HI_1_13_0" => 1852,
        _ => {
            out.error = Some(PlayerTableError::UnknownBuild);
            return out;
        }
    };
    out.report.perso_bytes = perso;
    let end = d.iter().rposition(|b| *b != 0).map_or(0, |n| (n + 1) * 8);
    if identity.body_bit == 0 || end <= identity.body_bit {
        out.error = Some(PlayerTableError::Truncated);
        return out;
    }
    let bits = Bits(d);
    let mut candidates = Vec::new();
    for p in identity.body_bit..=end.saturating_sub(149) {
        if bits.read(p, 3) != Some(4) || bits.read(p + 3, 32) != Some(0) {
            continue;
        }
        let x = bits.read(p + 85, 64).unwrap_or(0);
        if x > 0x0009000000000000
            && x < 0x000a000000000000
            && bits.read(p + 37, 48).is_some_and(|x| x != 0)
        {
            candidates.push(p);
        }
    }
    out.report.candidates_scanned = candidates.len();
    let real: Vec<_> = candidates
        .into_iter()
        .filter_map(|p| slot(d, p, perso * 8))
        .filter(|s| printable(&s.gamertag))
        .collect();
    out.report.candidates_real = real.len();
    let Some(first) = real.first() else {
        out.error = Some(PlayerTableError::NotFound);
        return out;
    };
    let stride = VACANT_FIXED + perso * 8;
    for head in 0..32 {
        let Some(start) = first.bit.checked_sub(head * stride) else {
            break;
        };
        if let Some((slots, vac)) = walk(d, start, perso * 8)
            && slots.len() > out.slots.len()
            && real.iter().all(|r| slots.iter().any(|s| s.bit == r.bit))
        {
            out.slots = slots;
            out.report.vacant = vac;
            out.report.head_vacant = head;
            out.report.first_record_bit = start;
        }
        if !start.checked_sub(stride).is_some_and(|p| vacant(d, p)) {
            break;
        }
    }
    if out.slots.is_empty() {
        out.error = Some(PlayerTableError::NotFound);
        return out;
    }
    let rep = &mut out.report;
    rep.occupied = out.slots.len();
    rep.interleaved_vacant = out
        .slots
        .last()
        .is_some_and(|s| s.film_index != rep.head_vacant + rep.occupied - 1);
    let mut surplus = BTreeMap::<i64, usize>::new();
    for pair in real.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        let after = a.bit + a.total_bits;
        let gap = b.bit as i64 - after as i64;
        *surplus.entry(gap).or_default() += 1;
        if gap == 0 {
            rep.gaps_agree += 1;
        } else if gap > 0
            && (gap as usize).is_multiple_of(stride)
            && (0..gap as usize / stride).all(|k| vacant(d, after + k * stride))
        {
            rep.gaps_vacant += 1;
        } else if out.slots.iter().any(|s| s.bit > a.bit && s.bit < b.bit) {
            rep.gaps_hidden += 1;
        } else {
            rep.gaps_contradict += 1;
        }
    }
    let mut modal = 0;
    for (gap, count) in surplus {
        if count > rep.film_delta_gaps {
            modal = gap;
            rep.film_delta_gaps = count;
        }
    }
    rep.film_delta_bits = modal + rep.profile_delta_bits;
    rep.calibration_agrees = rep.film_delta_gaps > 0 && modal == 0;
    out
}
