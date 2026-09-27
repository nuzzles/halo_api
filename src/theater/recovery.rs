//! LevelUp's guarded keyframe anchor recovery (`keyframe_world.go`).
//! This is explicitly heuristic; selected headers are not sequentially proven records.
use super::bits::Bits;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorRecovery {
    InitialHeader,
    Scan { from_bit: usize },
    RepeatedWidth { body_bits: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveredKeyframeAnchor {
    pub id: u32,
    pub archetype: u32,
    pub bit: usize,
    pub recovery: AnchorRecovery,
}

pub(super) fn anchor(data: Bits<'_>, bit: usize, previous: Option<u32>) -> Option<(u32, u32)> {
    let id = data.read(bit, 32)? as u32;
    let slot = id & 0x3fff_ffff;
    if id == u32::MAX || id >> 30 == 0 || slot >= 8192 || previous.is_some_and(|p| slot <= p) {
        return None;
    }
    let ti = data.read(bit.checked_add(32)?, 32)? as u32;
    (ti < 50).then_some((id, ti))
}

fn scan(data: Bits<'_>, from: usize, previous: Option<u32>) -> Option<usize> {
    let end = from.saturating_add(120_000).min(data.len());
    if end < 64 || from > end - 64 {
        return None;
    }
    let next_slot = previous.map_or(0, |p| p + 1);
    let mut best: Option<((bool, u32, u32, usize), usize)> = None;
    let mut sentinel_streak = 0;
    for bit in from..=end - 64 {
        if data.read(bit, 32)? == u32::MAX as u64 {
            sentinel_streak += 1;
            if sentinel_streak >= 2048 {
                break;
            }
            continue;
        }
        sentinel_streak = 0;
        let Some((id, _)) = anchor(data, bit, previous) else {
            continue;
        };
        let slot = id & 0x3fff_ffff;
        let generation = id >> 30;
        if slot == next_slot && generation == 1 {
            return Some(bit);
        }
        // Ascending tuple: consecutive first, then low generation/slot/offset.
        let score = (slot != next_slot, generation, slot, bit);
        if best.as_ref().is_none_or(|(current, _)| score < *current) {
            best = Some((score, bit));
        }
    }
    best.map(|(_, bit)| bit)
}

/// Exact reference selection policy: 120,000-bit search window, consecutive
/// generation-one preference, and per-archetype repeated-width shortcuts.
/// No selected anchor implies that the preceding payload was understood.
pub fn recover_keyframe_anchors(bytes: &[u8]) -> Vec<RecoveredKeyframeAnchor> {
    let data = Bits(bytes);
    let mut widths = BTreeMap::<u32, usize>::new();
    let mut seen = BTreeMap::<u32, usize>::new();
    let mut out = Vec::new();
    let mut previous = None;
    let (mut position, mut recovery) = if anchor(data, 1, None).is_some() {
        (Some(1), AnchorRecovery::InitialHeader)
    } else {
        (scan(data, 1, None), AnchorRecovery::Scan { from_bit: 1 })
    };
    while let Some(bit) = position {
        let Some((id, ti)) = anchor(data, bit, previous) else {
            break;
        };
        let slot = id & 0x3fff_ffff;
        out.push(RecoveredKeyframeAnchor {
            id,
            archetype: ti,
            bit,
            recovery,
        });
        let start = bit + 64;
        let fast = widths.get(&ti).and_then(|width| {
            let next = start.checked_add(*width)?;
            let (id, _) = anchor(data, next, Some(slot))?;
            (id >> 30 == 1).then_some((next, *width))
        });
        let next = if let Some((next, width)) = fast {
            recovery = AnchorRecovery::RepeatedWidth { body_bits: width };
            Some(next)
        } else {
            recovery = AnchorRecovery::Scan { from_bit: start };
            scan(data, start, Some(slot))
        };
        let Some(next) = next else {
            break;
        };
        let width = next - start;
        if let Some(first) = seen.get(&ti) {
            if *first == width {
                widths.insert(ti, width);
            } else {
                widths.remove(&ti);
            }
        } else {
            seen.insert(ti, width);
        }
        previous = Some(slot);
        position = Some(next);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn inflate(bytes: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        flate2::read::ZlibDecoder::new(bytes)
            .read_to_end(&mut out)
            .unwrap();
        out
    }
    fn compare(payload: &[u8], expected: &serde_json::Value) -> usize {
        let anchors = recover_keyframe_anchors(payload);
        let expected = expected.as_array().unwrap();
        assert_eq!(anchors.len(), expected.len());
        for (a, b) in anchors.iter().zip(expected) {
            assert_eq!(a.id & 0x3fff_ffff, b["Slot"]);
            assert_eq!(a.id >> 30, b["Gen"]);
            assert_eq!(a.archetype, b["TI"]);
            assert_eq!(a.bit, b["Bit"]);
        }
        anchors.len()
    }
    #[test]
    fn captured_anchor_recovery_matches_reference() {
        let payload = inflate(include_bytes!("fixtures/captured-keyframe-v41.zlib"));
        let oracle: serde_json::Value = serde_json::from_slice(&inflate(include_bytes!(
            "fixtures/captured-keyframe-v41-oracle.json.zlib"
        )))
        .unwrap();
        assert!(compare(&payload, &oracle["anchors"]) > 10);
        assert!(recover_keyframe_anchors(&[]).is_empty());
        assert!(recover_keyframe_anchors(&[255; 3000]).is_empty());
    }
    #[test]
    #[ignore = "requires downloaded 32-film corpus; compares all 451 captured keyframes"]
    fn corpus_anchor_recovery_matches_reference() {
        let oracle: Vec<serde_json::Value> = serde_json::from_slice(&inflate(include_bytes!(
            "fixtures/recovery-v41-oracle.json.zlib"
        )))
        .unwrap();
        assert_eq!(oracle.len(), 451);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films");
        let mut count = 0;
        for row in oracle {
            let bytes = std::fs::read(root.join(row["file"].as_str().unwrap())).unwrap();
            let at = row["payload_byte"].as_u64().unwrap() as usize;
            let size = row["size"].as_u64().unwrap() as usize;
            count += compare(&bytes[at..at + size], &row["anchors"]);
        }
        println!("matched {count} recovered anchors across 451 keyframes");
    }
}
