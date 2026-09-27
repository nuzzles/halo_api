//! Return gauge binding by correlation with published dropped-flag spans.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFlagGaugeSlot {
    pub slot: u32,
    pub samples: Vec<ReplayGaugeSample>,
}
pub fn replay_flag_gauge_slots(
    reads: &[ManagedPropertyRead],
    clock: ReplayMatchClock,
) -> Vec<ReplayFlagGaugeSlot> {
    let mut by_slot = BTreeMap::<u32, Vec<ReplayGaugeSample>>::new();
    for r in reads {
        if r.field != 0 || r.tag != 3 || !r.has_value {
            continue;
        }
        let f = flag_frame_of(r.timestamp_us, clock);
        if f < 0 || f >= clock.frames {
            continue;
        }
        by_slot
            .entry(r.slot)
            .or_default()
            .push(ReplayGaugeSample { t: f, v: r.value });
    }
    by_slot
        .into_iter()
        .map(|(slot, mut samples)| {
            samples.sort_by_key(|s| s.t);
            ReplayFlagGaugeSlot { slot, samples }
        })
        .collect()
}
pub fn bind_replay_flag_gauges(
    slots: &[ReplayFlagGaugeSlot],
    carries: &[ReplayFlagCarry],
) -> BTreeMap<usize, usize> {
    let mut candidates = Vec::new();
    for (si, s) in slots.iter().enumerate() {
        if s.samples.len() < 8 {
            continue;
        }
        for (f, c) in carries.iter().enumerate() {
            let inside = s
                .samples
                .iter()
                .filter(|s| {
                    let i = c.spans.partition_point(|p| p.t1 < s.t);
                    c.spans
                        .get(i)
                        .is_some_and(|p| p.state == "dropped" && s.t >= p.t0)
                })
                .count();
            let share = inside as f64 / s.samples.len() as f64;
            if share >= 0.8 {
                candidates.push((f, si, share, inside));
            }
        }
    }
    candidates.sort_by(|a, b| b.2.total_cmp(&a.2).then(b.3.cmp(&a.3)));
    let mut out = BTreeMap::new();
    let mut used = BTreeSet::new();
    for (f, s, _, _) in candidates {
        if !out.contains_key(&f) && used.insert(s) {
            out.insert(f, s);
        }
    }
    out
}
pub fn attach_replay_flag_return_gauges(
    carries: &mut [ReplayFlagCarry],
    reads: &[ManagedPropertyRead],
    scanned: bool,
    clock: ReplayMatchClock,
    mut coverage: Option<&mut ReplayFlagCoverage>,
) {
    if let Some(c) = coverage.as_deref_mut() {
        c.gauge_scanned = scanned;
    }
    if !scanned || carries.is_empty() || clock.step_us == 0 {
        return;
    }
    let slots = replay_flag_gauge_slots(reads, clock);
    if let Some(c) = coverage.as_deref_mut() {
        c.gauge_slots = slots.len();
        c.gauge_reads = slots.iter().map(|s| s.samples.len()).sum();
    }
    let bound = bind_replay_flag_gauges(&slots, carries);
    let gap = replay_gauge_gap_frames((clock.step_us / 1000) as i64);
    for (f, si) in bound {
        if let Some(c) = coverage.as_deref_mut() {
            c.gauge_paired += 1;
        }
        for span in carries[f].spans.iter_mut().filter(|s| s.state == "dropped") {
            let mut pts = Vec::new();
            append_replay_gauge_thinned(&mut pts, &slots[si].samples, span.t0, span.t1, gap);
            if pts.is_empty() {
                continue;
            }
            if let Some(c) = coverage.as_deref_mut() {
                c.gauge_spans += 1;
                c.gauge_points += pts.len();
            }
            span.return_progress = pts;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_flag_gauge_binding_and_shared_series() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-flag-gauges-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let get = |s: &str| row[s].clone();
            let clock: ReplayMatchClock = serde_json::from_value(get("clock")).unwrap();
            let reads: Vec<ManagedPropertyRead> = serde_json::from_value(get("reads")).unwrap();
            let mut carries: Vec<ReplayFlagCarry> = serde_json::from_value(get("before")).unwrap();
            let slots = replay_flag_gauge_slots(&reads, clock);
            assert_eq!(
                slots,
                serde_json::from_value::<Vec<ReplayFlagGaugeSlot>>(get("slots")).unwrap(),
                "slots {i}"
            );
            assert_eq!(
                bind_replay_flag_gauges(&slots, &carries),
                serde_json::from_value(get("binding")).unwrap(),
                "binding {i}"
            );
            let mut c = clock;
            if row["zero_step"].as_bool().unwrap() {
                c.step_us = 0;
            }
            let mut cov = ReplayFlagCoverage::default();
            attach_replay_flag_return_gauges(
                &mut carries,
                &reads,
                row["scanned"].as_bool().unwrap(),
                c,
                Some(&mut cov),
            );
            assert_eq!(
                carries,
                serde_json::from_value::<Vec<ReplayFlagCarry>>(get("after")).unwrap(),
                "publication {i}"
            );
            assert_eq!(
                cov,
                serde_json::from_value(get("coverage")).unwrap(),
                "coverage {i}"
            );
            let samples: Vec<ReplayGaugeSample> = serde_json::from_value(get("samples")).unwrap();
            let wins: Vec<(i64, i64)> = serde_json::from_value(get("windows")).unwrap();
            let gap = replay_gauge_gap_frames((clock.step_us / 1000) as i64);
            assert_eq!(
                replay_gauge_series(&samples, &wins, gap),
                serde_json::from_value::<Vec<ReplayGaugePoint>>(get("series")).unwrap(),
                "series {i}"
            );
            let mut thin = Vec::new();
            let next = append_replay_gauge_thinned(&mut thin, &samples, 20, 65, gap);
            assert_eq!(
                thin,
                serde_json::from_value::<Vec<ReplayGaugePoint>>(get("thin")).unwrap(),
                "thin {i}"
            );
            assert_eq!(next, row["next"].as_u64().unwrap() as usize, "next {i}");
        }
    }
}
