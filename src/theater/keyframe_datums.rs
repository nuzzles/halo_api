//! Longest-increasing-subsequence keyframe datum recovery, separate from type-1 tables.
use super::{FilmWorld, bits::Bits};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeDatumCandidate {
    pub bit: usize,
    pub slot: u32,
    pub archetype: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeDatums {
    pub candidates: Vec<KeyframeDatumCandidate>,
    /// Indices of the selected strictly increasing slot chain.
    pub selected: Vec<usize>,
    pub table: BTreeMap<u32, u32>,
    pub ambiguous: usize,
}
pub fn recover_keyframe_datums(data: &[u8]) -> KeyframeDatums {
    let bits = Bits(data);
    let mut out = KeyframeDatums::default();
    for bit in 0..bits.len().saturating_sub(63) {
        if let Some((id, archetype)) = super::recovery::anchor(bits, bit, None) {
            out.candidates.push(KeyframeDatumCandidate {
                bit,
                slot: id & 0x3fff_ffff,
                archetype,
            });
        }
    }
    let mut tails: Vec<usize> = Vec::new();
    let mut predecessors = vec![None; out.candidates.len()];
    for (i, c) in out.candidates.iter().enumerate() {
        let at = tails.partition_point(|&j| out.candidates[j].slot < c.slot);
        if at > 0 {
            predecessors[i] = Some(tails[at - 1]);
        }
        if at == tails.len() {
            tails.push(i)
        } else {
            tails[at] = i
        }
    }
    let mut next = tails.last().copied();
    while let Some(i) = next {
        out.selected.push(i);
        next = predecessors[i];
    }
    out.selected.reverse();
    out.ambiguous = out.candidates.len() - out.selected.len();
    for &i in &out.selected {
        let c = &out.candidates[i];
        out.table.insert(c.slot, c.archetype);
    }
    out
}
impl KeyframeDatums {
    /// Supplement missing bindings; keyframe/creation facts already in the world win.
    pub fn bind_missing(&self, world: &mut FilmWorld) -> usize {
        let mut count = 0;
        for (&slot, &ti) in &self.table {
            if world.archetype(slot).is_none() {
                world.bind_datum(slot, ti);
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    #[test]
    fn candidates_lis_and_table_match_native_synthetic_and_captured_keyframes() {
        let mut text = String::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/world-levelup-v41.json.zlib")[..])
            .read_to_string(&mut text)
            .unwrap();
        let oracle: Value = serde_json::from_str(&text).unwrap();
        let rows = oracle["datums"].as_array().unwrap();
        assert_eq!(rows.len(), 513);
        for row in rows {
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let actual = recover_keyframe_datums(&bytes);
            let candidates: Vec<_> = actual
                .candidates
                .iter()
                .map(|c| json!([c.bit, c.slot, c.archetype]))
                .collect();
            assert_eq!(json!(candidates), row["candidates"]);
            assert_eq!(
                actual.selected,
                serde_json::from_value::<Option<Vec<usize>>>(row["selected"].clone())
                    .unwrap()
                    .unwrap_or_default()
            );
            assert_eq!(json!(actual.table), row["table"]);
            assert_eq!(json!(actual.ambiguous), row["ambiguous"]);
            let mut world = FilmWorld::default();
            if let Some((&slot, &ti)) = actual.table.first_key_value() {
                world.bind_full(slot, (ti + 1) % 50);
            }
            let before = world.clone();
            assert_eq!(
                actual.bind_missing(&mut world),
                actual.table.len() - before.slots.len()
            );
            assert_eq!(actual.bind_missing(&mut world), 0);
            for (&slot, &ti) in &actual.table {
                assert_eq!(world.archetype(slot), before.archetype(slot).or(Some(ti)));
            }
        }
        for n in 0..8 {
            assert_eq!(
                recover_keyframe_datums(&vec![255; n]),
                KeyframeDatums::default()
            );
        }
    }
}
