//! Independent carrier-marker checks and final raw-carry diagnostics.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub fn mark_replay_flag_carries(
    raw: &mut [ReplayFlagCarryRaw],
    scan: &CarrierMarkScan,
    bridge: &BTreeMap<u32, u64>,
    death_offset_ms: i64,
) {
    mark_flag_values(raw, &scan.marks, &scan.keyframe_us, bridge, death_offset_ms)
}
/// Cached scanner counters are retained by the cache but do not participate in
/// native marker confirmation. Consume only the recorded marks and keyframe times.
pub fn mark_facts_replay_flag_carries(
    raw: &mut [ReplayFlagCarryRaw],
    scan: &FactsCarrierMarkScan,
    bridge: &BTreeMap<u32, u64>,
    death_offset_ms: i64,
) {
    mark_flag_values(
        raw,
        scan.marks.as_deref().unwrap_or(&[]),
        scan.keyframe_us.as_deref().unwrap_or(&[]),
        bridge,
        death_offset_ms,
    )
}
fn mark_flag_values(
    raw: &mut [ReplayFlagCarryRaw],
    marks: &[CarrierMark],
    keyframes: &[u64],
    bridge: &BTreeMap<u32, u64>,
    death_offset_ms: i64,
) {
    let mut marked = BTreeMap::<String, BTreeSet<i64>>::new();
    for m in marks {
        if let Some(x) = bridge.get(&m.slot) {
            marked
                .entry(x.to_string())
                .or_default()
                .insert(((m.timestamp_us / 1000) as i64).wrapping_sub(death_offset_ms));
        }
    }
    for r in raw {
        for &us in keyframes {
            let at = ((us / 1000) as i64).wrapping_sub(death_offset_ms);
            if at < r.t0 || at > r.t1 {
                continue;
            }
            r.observable = true;
            if marked.get(&r.xuid).is_some_and(|times| times.contains(&at)) {
                r.confirmed = true;
                break;
            }
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayFlagCarryTally {
    pub carries: usize,
    pub closed: usize,
    pub open: usize,
    pub marker_observed: usize,
    pub marker_confirmed: usize,
    pub open_observed: usize,
    pub open_confirmed: usize,
    pub closed_by_handoff: usize,
    pub closed_by_return: usize,
    pub closed_by_home: usize,
    pub closed_by_object: usize,
    pub overlaps: usize,
    pub closed_overlaps: usize,
}
pub fn tally_replay_flag_carries(raw: &[ReplayFlagCarryRaw]) -> ReplayFlagCarryTally {
    let mut c = ReplayFlagCarryTally {
        carries: raw.len(),
        ..Default::default()
    };
    for r in raw {
        match r.closed_by {
            ReplayFlagCloser::Handoff => c.closed_by_handoff += 1,
            ReplayFlagCloser::Return => c.closed_by_return += 1,
            ReplayFlagCloser::Home => c.closed_by_home += 1,
            ReplayFlagCloser::Object => c.closed_by_object += 1,
            _ => {}
        }
        if r.closed {
            c.closed += 1;
            c.marker_observed += usize::from(r.observable);
            c.marker_confirmed += usize::from(r.confirmed);
        } else {
            c.open += 1;
            c.open_observed += usize::from(r.observable);
            c.open_confirmed += usize::from(r.confirmed);
        }
        if r.flag_index < 0 {
            continue;
        }
        let active: Vec<_> = raw
            .iter()
            .filter(|p| p.flag_index == r.flag_index && p.t0 <= r.t0 && r.t0 < p.t1)
            .collect();
        if active.len() > 1 {
            c.overlaps += 1;
        }
        if r.closed && active.iter().filter(|p| p.closed).count() > 1 {
            c.closed_overlaps += 1;
        }
    }
    c
}
