//! Bounded reference player-slot grammar shared by roster and bootstrap reads.
use crate::theater::parser::bits::Cursor;
use crate::theater::resolved::identity::{PlayerTableShorts, PlayerTableSlot};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlotField {
    pub name: String,
    pub bit: usize,
    pub bits: usize,
    pub value: SlotValue,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlotValue {
    Scalar(u64),
    Opaque,
    Unavailable,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlayerSlotRead {
    pub start_bit: usize,
    pub end_bit: usize,
    pub source_bits: usize,
    pub truncated: bool,
    pub slot: Option<PlayerTableSlot>,
    pub fields: Vec<SlotField>,
}
/// Read an explicitly supplied candidate boundary, without validating its
/// selection or inferring a vacant slot. The returned rank is not resolved.
pub(crate) fn read_player_slot(
    data: &[u8],
    start_bit: usize,
    personalization_bits: usize,
) -> PlayerSlotRead {
    let mut out = PlayerSlotRead {
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
pub(crate) fn slot(d: &[u8], p: usize, personalization: usize) -> Option<PlayerTableSlot> {
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
/// The shared bootstrap/type-8 body uses the reference sticky bounded reader.
pub(crate) struct SlotReader<'a> {
    pub cursor: Cursor<'a>,
    pub truncated: bool,
    pub fields: Option<Vec<SlotField>>,
}
impl<'a> SlotReader<'a> {
    pub(crate) fn new(cursor: Cursor<'a>, trace: bool) -> Self {
        Self {
            cursor,
            truncated: false,
            fields: trace.then(Vec::new),
        }
    }
    fn retain(&mut self, name: &str, bit: usize, bits: usize, value: SlotValue) {
        if let Some(fields) = &mut self.fields {
            fields.push(SlotField {
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
            value.map_or(SlotValue::Unavailable, SlotValue::Scalar),
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
            value.map_or(SlotValue::Unavailable, |_| SlotValue::Opaque),
        );
        value
    }
}
pub(crate) fn slot_body(
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
