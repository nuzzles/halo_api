//! Type-1 entity datum tables, ported from LevelUp `type1_datums.go`.
use super::{DecodeError, bits::Cursor};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatumEntry {
    pub flags: u8,
    pub generation_tag: u8,
    pub generation: u32,
    pub view_mask: u64,
    pub component_mask: [u64; 4],
}

impl DatumEntry {
    pub fn live(&self) -> bool {
        self.flags & 5 == 5 && self.flags & 0x22 == 0
    }
    pub fn component(&self, index: usize) -> bool {
        index < 256 && self.component_mask[index / 64] & (1 << (index % 64)) != 0
    }
}

impl Default for DatumEntry {
    fn default() -> Self {
        Self {
            flags: 0,
            generation_tag: 0,
            generation: 1,
            view_mask: 0,
            component_mask: [0; 4],
        }
    }
}

/// Lossless sparse representation: absent entries equal `default_entry`, not unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatumTable {
    pub slot_count: usize,
    pub default_entry: DatumEntry,
    pub entries: BTreeMap<usize, DatumEntry>,
    pub tail_words: [u32; 5],
    pub consumed_bits: usize,
    pub padding_bits: usize,
    pub padding: u8,
}

impl DatumTable {
    pub fn entry(&self, slot: usize) -> Option<&DatumEntry> {
        (slot < self.slot_count).then(|| self.entries.get(&slot).unwrap_or(&self.default_entry))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatumSnapshot {
    pub chunk_index: i32,
    pub payload_byte: usize,
    pub timestamp_us: u64,
    pub table: DatumTable,
}

/// Native datum-block size refusals, preserving the quantities in each error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum DatumTableError {
    #[error("bloc de datums : {bytes} octets ne portent aucune entree")]
    NoEntries { bytes: usize },
    #[error("bloc de datums : {slots} entrees derivees au-dela du cap 8191")]
    AboveCapacity { slots: usize },
    #[error("bloc de datums : {total_bits} bits pour {slots} entrees, reste {remainder}")]
    Misaligned {
        total_bits: usize,
        slots: usize,
        remainder: i64,
    },
}

/// A framed datum payload that the table decoder refused. The raw bytes and
/// packet source remain available; no default table is substituted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatumDecodeFailure {
    pub source: super::FilmPacket,
    pub message: String,
    /// Absent in older exports or for non-native internal failures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_error: Option<DatumTableError>,
    pub payload: Vec<u8>,
}

/// Decode one complete decompressed type-1 packet payload. The layout has 79 bits
/// per datum, a separate 256-bit bitmap per slot, five tail words and byte alignment.
pub fn decode_datum_table(data: &[u8]) -> Result<DatumTable, DecodeError> {
    let invalid = || DecodeError::Inconsistent("invalid type-1 datum table length".into());
    let total = data.len().checked_mul(8).ok_or_else(invalid)?;
    if total < 160 {
        return Err(DatumTableError::NoEntries { bytes: data.len() }.into());
    }
    let count = (total - 160 + 7) / 335;
    if count == 0 {
        return Err(DatumTableError::NoEntries { bytes: data.len() }.into());
    }
    if count > 8191 {
        return Err(DatumTableError::AboveCapacity { slots: count }.into());
    }
    let consumed_bits = count * 335 + 160;
    let remainder = total as i64 - consumed_bits as i64;
    if !(0..=7).contains(&remainder) {
        return Err(DatumTableError::Misaligned {
            total_bits: total,
            slots: count,
            remainder,
        }
        .into());
    }
    let padding_bits = remainder as usize;
    let mut r = Cursor::new(data, 0).ok_or_else(invalid)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(DatumEntry {
            flags: r.read(6).ok_or_else(invalid)? as u8,
            generation_tag: r.read(8).ok_or_else(invalid)? as u8,
            generation: r.read(32).ok_or_else(invalid)? as u32,
            view_mask: r.read(33).ok_or_else(invalid)?.reverse_bits() >> 31,
            component_mask: [0; 4],
        });
    }
    for entry in &mut entries {
        for word in &mut entry.component_mask {
            *word = r.read(64).ok_or_else(invalid)?.reverse_bits();
        }
    }
    let mut tail_words = [0; 5];
    for word in &mut tail_words {
        *word = r.read(32).ok_or_else(invalid)? as u32;
    }
    let padding = r.read(padding_bits).ok_or_else(invalid)? as u8;
    let default_entry = DatumEntry::default();
    let entries = entries
        .into_iter()
        .enumerate()
        .filter(|(_, e)| *e != default_entry)
        .collect();
    Ok(DatumTable {
        slot_count: count,
        default_entry,
        entries,
        tail_words,
        consumed_bits,
        padding_bits,
        padding,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn native_datum_length_errors_and_capacity() {
        let rows: Vec<serde_json::Value> =
            serde_json::from_slice(include_bytes!("fixtures/datum-lengths-v41.json")).unwrap();
        assert_eq!(rows.len(), 139);
        let mut kinds = [false; 3];
        for row in rows {
            let data = vec![0; row["length"].as_u64().unwrap() as usize];
            match decode_datum_table(&data) {
                Ok(table) => {
                    assert_eq!(row["error"], false);
                    assert_eq!(table.slot_count, row["slots"].as_u64().unwrap() as usize);
                }
                Err(DecodeError::Datums(error)) => {
                    assert_eq!(row["error"], true);
                    assert_eq!(error.to_string(), row["message"].as_str().unwrap());
                    kinds[match error {
                        DatumTableError::NoEntries { .. } => 0,
                        DatumTableError::AboveCapacity { .. } => 1,
                        DatumTableError::Misaligned { .. } => 2,
                    }] = true;
                    let restored: DatumTableError =
                        serde_json::from_value(serde_json::to_value(&error).unwrap()).unwrap();
                    assert_eq!(restored, error);
                }
                Err(error) => panic!("untyped datum error: {error}"),
            }
        }
        assert_eq!(kinds, [true; 3]);
    }

    fn inflate(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        flate2::read::ZlibDecoder::new(data)
            .read_to_end(&mut out)
            .unwrap();
        out
    }

    #[test]
    fn captured_datum_table_matches_levelup_every_slot_and_bitmap() {
        let bytes = inflate(include_bytes!("fixtures/datums-v41.zlib"));
        let reference: serde_json::Value = serde_json::from_slice(&inflate(include_bytes!(
            "fixtures/datums-v41-oracle.json.zlib"
        )))
        .unwrap();
        let actual = decode_datum_table(&bytes).unwrap();
        assert_eq!(actual.slot_count, 8191);
        assert_eq!(actual.consumed_bits, reference["BitsLus"]);
        assert_eq!(actual.padding_bits, reference["Bourrage"]);
        assert_eq!(
            serde_json::to_value(actual.tail_words).unwrap(),
            reference["Queue"]
        );
        for (slot, expected) in reference["Entrees"].as_array().unwrap().iter().enumerate() {
            let entry = actual.entry(slot).unwrap();
            assert_eq!(entry.flags, expected["Drapeaux"], "slot {slot}");
            assert_eq!(entry.generation_tag, expected["Gen"], "slot {slot}");
            assert_eq!(entry.generation, expected["Generation"], "slot {slot}");
            assert_eq!(entry.view_mask, expected["MasqueVue"], "slot {slot}");
            assert_eq!(
                serde_json::to_value(entry.component_mask).unwrap(),
                expected["Composants"],
                "slot {slot}"
            );
        }
        assert!(actual.entry(8191).is_none());
        assert!(actual.entries.len() < actual.slot_count);
        let roundtrip: DatumTable =
            serde_json::from_slice(&serde_json::to_vec(&actual).unwrap()).unwrap();
        assert_eq!(roundtrip, actual);
    }

    #[test]
    fn datum_masks_reverse_bit_order_and_keep_alignment_bits() {
        let mut bytes = [0u8; 62]; // 335 bits per slot + 160 tail + 1 alignment.
        bytes[0] = 5 << 2;
        bytes[5] |= 2; // first view bit at position 46
        bytes[9] |= 1; // first component bit at position 79
        bytes[10] |= 0x80; // second component bit at position 80
        bytes[61] |= 1; // alignment bit
        let table = decode_datum_table(&bytes).unwrap();
        let entry = table.entry(0).unwrap();
        assert!(entry.live());
        assert_eq!(entry.view_mask, 1);
        assert_eq!(entry.component_mask[0], 3);
        assert!(entry.component(0) && entry.component(1));
        assert!(!entry.component(2) && !entry.component(256));
        assert_eq!((table.padding_bits, table.padding), (1, 1));
        assert!(decode_datum_table(&bytes[..61]).is_err());
        for len in 0..21 {
            assert!(decode_datum_table(&vec![0; len]).is_err());
        }
    }
}
