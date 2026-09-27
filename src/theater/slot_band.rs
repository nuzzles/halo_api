//! Native dense slot membership. Cardinality is distinct from addressable range.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilmSlotBand {
    dense: Vec<bool>,
    count: usize,
}
impl FilmSlotBand {
    /// Native NewSlotBand: false map entries are absent, including high slots.
    /// Allocation scales with max(8192, highest present slot + 1).
    pub fn from_flags(flags: &BTreeMap<u32, bool>) -> Result<Self, super::DecodeError> {
        Self::from_slots(
            flags
                .iter()
                .filter_map(|(&slot, &present)| present.then_some(slot)),
        )
    }
    pub fn from_set(slots: &BTreeSet<u32>) -> Result<Self, super::DecodeError> {
        Self::from_slots(slots.iter().copied())
    }
    /// Duplicate input slots count once. Allocation failures are explicit.
    pub fn from_slots(slots: impl IntoIterator<Item = u32>) -> Result<Self, super::DecodeError> {
        let slots: Vec<u32> = slots.into_iter().collect();
        let size = slots
            .iter()
            .copied()
            .max()
            .map_or(Some(8192), |slot| {
                (slot as usize).checked_add(1).map(|n| n.max(8192))
            })
            .ok_or_else(|| super::DecodeError::Inconsistent("slot band domain overflow".into()))?;
        let mut dense = Vec::new();
        dense
            .try_reserve_exact(size)
            .map_err(|e| super::DecodeError::Inconsistent(format!("slot band allocation: {e}")))?;
        dense.resize(size, false);
        let mut count = 0;
        for slot in slots {
            if !dense[slot as usize] {
                dense[slot as usize] = true;
                count += 1;
            }
        }
        Ok(Self { dense, count })
    }
    /// O(1) membership, including slots outside the allocated domain.
    pub fn contains(&self, slot: u32) -> bool {
        self.dense.get(slot as usize).copied().unwrap_or(false)
    }
    /// Present-slot cardinality, not dense table length.
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// Ascending slots. Enumeration scans the addressable range.
    pub fn slots(&self) -> Vec<u32> {
        self.dense
            .iter()
            .enumerate()
            .filter_map(|(s, &present)| present.then_some(s as u32))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        flags: BTreeMap<u32, bool>,
        count: usize,
        slots: Vec<u32>,
        queries: Vec<u32>,
        has: Vec<bool>,
    }
    #[test]
    fn native_dense_slot_membership() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/slot-band-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 256);
        for (i, c) in rows.into_iter().enumerate() {
            let band = FilmSlotBand::from_flags(&c.flags).unwrap();
            assert_eq!(band.count(), c.count, "count {i}");
            assert_eq!(band.is_empty(), c.count == 0);
            assert_eq!(band.slots(), c.slots, "slots {i}");
            assert_eq!(c.queries.len(), c.has.len());
            for (q, expected) in c.queries.into_iter().zip(c.has) {
                assert_eq!(band.contains(q), expected, "case {i} slot {q}");
            }
            let set = c
                .flags
                .into_iter()
                .filter_map(|(s, present)| present.then_some(s))
                .collect();
            assert_eq!(FilmSlotBand::from_set(&set).unwrap(), band);
            assert_eq!(
                FilmSlotBand::from_slots(c.slots.iter().chain(c.slots.iter()).copied()).unwrap(),
                band
            );
        }
        let empty = FilmSlotBand::default();
        assert!(empty.is_empty());
        assert!(!empty.contains(u32::MAX));
        assert!(empty.slots().is_empty());
    }
}
