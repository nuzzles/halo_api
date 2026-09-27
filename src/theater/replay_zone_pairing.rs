//! Independent gauge and owner-channel elections from geometrically attributed captures.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayZonePair {
    pub t: i64,
    pub zone_ref: i64,
    pub xuid: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayZoneOwnerCandidate {
    pub slot: u32,
    pub zone_ref: i64,
    pub score: usize,
}
pub fn replay_zone_pairs(attributions: &[ReplayZoneAttribution]) -> Vec<ReplayZonePair> {
    attributions
        .iter()
        .filter(|a| a.attributed)
        .map(|a| ReplayZonePair {
            t: a.action.t,
            zone_ref: a.spatial_rank as i64,
            xuid: a.action.xuid.clone(),
        })
        .collect()
}
pub fn replay_zone_catalog(zones: &[ObjectiveZone]) -> Vec<ObjectiveZone> {
    zones
        .iter()
        .enumerate()
        .map(|(i, z)| {
            let mut z = z.clone();
            z.spatial_rank = i;
            z
        })
        .collect()
}
pub fn replay_zone_slot_at_peak(ramps: &[ReplayZoneRamp], t: i64, win: i64) -> Option<u32> {
    let mut best = None;
    let mut distance = win.saturating_add(1);
    let mut ambiguous = false;
    for r in ramps {
        let d = (r.t_peak - t).abs();
        if d > win {
            continue;
        }
        if best.is_none() || d < distance {
            best = Some(r.slot);
            distance = d;
            ambiguous = false;
        } else if d == distance && best != Some(r.slot) {
            ambiguous = true;
        }
    }
    if ambiguous { None } else { best }
}
pub fn pair_replay_zone_gauges(
    ramps: &[ReplayZoneRamp],
    pairs: &[ReplayZonePair],
    win: i64,
) -> (BTreeMap<i64, u32>, usize) {
    let mut votes = BTreeMap::<u32, BTreeMap<i64, usize>>::new();
    for p in pairs {
        if let Some(slot) = replay_zone_slot_at_peak(ramps, p.t, win) {
            *votes
                .entry(slot)
                .or_default()
                .entry(p.zone_ref)
                .or_default() += 1;
        }
    }
    let mut best = BTreeMap::new();
    let mut counts = BTreeMap::new();
    for (slot, refs) in votes {
        let mut modal = (-1, 0);
        for (r, n) in refs {
            if n > modal.1 {
                modal = (r, n);
            }
        }
        if modal.1 > *counts.get(&modal.0).unwrap_or(&0) {
            best.insert(modal.0, slot);
            counts.insert(modal.0, modal.1);
        }
    }
    let held: BTreeSet<_> = best.values().copied().collect();
    let unpaired = ramps
        .iter()
        .map(|r| r.slot)
        .filter(|s| !held.contains(s))
        .collect::<BTreeSet<_>>()
        .len();
    (best, unpaired)
}
pub fn replay_zone_value_after(samples: &[ReplayZoneSample], t: i64, win: i64) -> Option<u64> {
    samples
        .get(samples.partition_point(|s| s.t < t))
        .filter(|s| s.t <= t.saturating_add(win))
        .map(|s| s.v)
}
pub fn replay_zone_owner_candidates(
    series: &ReplayZoneSeries,
    pairs: &[ReplayZonePair],
    teams: &BTreeMap<String, i64>,
    win: i64,
) -> Vec<ReplayZoneOwnerCandidate> {
    let mut out = Vec::new();
    for (&slot, ss) in &series.owner {
        if !ss.windows(2).any(|s| s[0].v != s[1].v) {
            continue;
        }
        let mut scores = BTreeMap::<i64, usize>::new();
        for p in pairs {
            if let Some(v) = replay_zone_value_after(ss, p.t, win)
                && (teams.get(&p.xuid).is_some_and(|&t| v == t as u64)
                    || (teams.is_empty() && v != 0xffff_ffff))
            {
                *scores.entry(p.zone_ref).or_default() += 1;
            }
        }
        out.extend(
            scores
                .into_iter()
                .filter(|(_, n)| *n >= 2)
                .map(|(zone_ref, score)| ReplayZoneOwnerCandidate {
                    slot,
                    zone_ref,
                    score,
                }),
        );
    }
    out
}
pub fn elect_replay_zone_owners(
    mut candidates: Vec<ReplayZoneOwnerCandidate>,
) -> BTreeMap<i64, u32> {
    candidates.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.zone_ref.cmp(&b.zone_ref))
            .then(a.slot.cmp(&b.slot))
    });
    let mut out = BTreeMap::new();
    let mut held = BTreeSet::new();
    for c in candidates {
        if !out.contains_key(&c.zone_ref) && held.insert(c.slot) {
            out.insert(c.zone_ref, c.slot);
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        series: ReplayZoneSeries,
        pairs: Vec<ReplayZonePair>,
        teams: BTreeMap<String, i64>,
        win: i64,
        gauges: BTreeMap<i64, u32>,
        unpaired: usize,
        candidates: Vec<ReplayZoneOwnerCandidate>,
        owners: BTreeMap<i64, u32>,
    }
    #[test]
    fn native_zone_channel_pairing() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-zone-pairing-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                pair_replay_zone_gauges(&c.series.ramps(), &c.pairs, c.win),
                (c.gauges, c.unpaired),
                "gauges {i}"
            );
            let candidates = replay_zone_owner_candidates(&c.series, &c.pairs, &c.teams, c.win);
            assert_eq!(candidates, c.candidates, "candidates {i}");
            assert_eq!(elect_replay_zone_owners(candidates), c.owners, "owners {i}");
        }
    }
}
