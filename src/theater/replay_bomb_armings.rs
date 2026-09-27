//! Bomb arming publication with full-quantum truncation and pause-corrected fuse validation.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayBombArmingsCoverage {
    pub scanned: bool,
    pub reads: usize,
    pub rises: usize,
    pub below_full: usize,
    pub armed: usize,
    pub pair_merged: usize,
    pub published: usize,
    pub out_of_window: usize,
    pub detonations: usize,
    pub detonations_covered: usize,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub suppressed: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayBombFuseVerdict {
    #[serde(rename = "FuseMS")]
    pub fuse_ms: i64,
    #[serde(rename = "CV")]
    pub cv: f64,
    pub measured: bool,
    pub inconsistent: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayBombArmings {
    pub armings: Vec<ReplayBombArming>,
    pub coverage: ReplayBombArmingsCoverage,
    pub verdict: ReplayBombFuseVerdict,
    pub start_zero_fallbacks: usize,
}
/// Crop at the first full sample before evaluating the segment shape. The
/// subsequent recharge tail does not negate the observed full-quantum arrival.
pub fn bomb_truncate_at_first_full(
    g: NavpointSegment,
    reads: &[NavpointRadialRead],
) -> NavpointSegment {
    if g.q_max < 254 {
        return g;
    }
    let mut out = NavpointSegment {
        slot: g.slot,
        start_ms: g.start_ms,
        q_start: g.q_start,
        ..Default::default()
    };
    for r in reads
        .iter()
        .filter(|r| r.time_ms >= g.start_ms && r.time_ms <= g.end_ms)
    {
        if out.samples == 0 {
            out.q_min = r.q;
            out.q_max = r.q;
        }
        out.samples += 1;
        out.end_ms = r.time_ms;
        out.q_end = r.q;
        out.q_min = out.q_min.min(r.q);
        out.q_max = out.q_max.max(r.q);
        if r.q >= 254 {
            return out;
        }
    }
    g
}
fn median_cv(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, f64::INFINITY);
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    let median = ordered[ordered.len() / 2];
    if median == 0.0 {
        return (median, f64::INFINITY);
    }
    let mut sum = 0.0;
    for x in values {
        let d = x - median;
        sum += d * d;
    }
    (median, (sum / values.len() as f64).sqrt() / median)
}
pub fn bomb_corrected_delay(
    armed: &[NavpointSegment],
    pauses: &BTreeMap<u32, Vec<NavpointSegment>>,
    target: i32,
) -> Option<i32> {
    // Preserve native binary-search assumptions even if truncation changes end order.
    let (mut i, mut hi) = (0, armed.len());
    while i < hi {
        let mid = i + (hi - i) / 2;
        if armed[mid].end_ms >= target {
            hi = mid;
        } else {
            i = mid + 1;
        }
    }
    let a = armed.get(i.checked_sub(1)?)?;
    let mut delay = target.wrapping_sub(a.end_ms);
    if delay <= 0 || delay > 120000 {
        return None;
    }
    if let Some(ps) = pauses.get(&a.slot) {
        for p in ps {
            if p.start_ms > a.end_ms && p.end_ms < target {
                delay = delay.wrapping_sub(p.end_ms.wrapping_sub(p.start_ms));
            }
        }
    }
    (delay > 0).then_some(delay)
}
fn measure_fuse(
    armed: &[NavpointSegment],
    pauses: &BTreeMap<u32, Vec<NavpointSegment>>,
    detonations: &[i64],
    cov: &mut ReplayBombArmingsCoverage,
) -> (ReplayBombFuseVerdict, bool) {
    let delays: Vec<_> = detonations
        .iter()
        .filter_map(|&t| bomb_corrected_delay(armed, pauses, t as i32))
        .map(f64::from)
        .collect();
    cov.detonations_covered = delays.len();
    if delays.len() < detonations.len() {
        if delays.is_empty() {
            return (ReplayBombFuseVerdict::default(), false);
        }
        let (median, cv) = median_cv(&delays);
        return (
            ReplayBombFuseVerdict {
                fuse_ms: median as i64,
                cv,
                ..Default::default()
            },
            false,
        );
    }
    if delays.is_empty() {
        return (
            ReplayBombFuseVerdict {
                fuse_ms: 4930,
                ..Default::default()
            },
            true,
        );
    }
    let (median, cv) = median_cv(&delays);
    let inconsistent = delays.len() >= 2 && cv > 0.20;
    (
        ReplayBombFuseVerdict {
            fuse_ms: median as i64,
            cv,
            measured: true,
            inconsistent,
        },
        !inconsistent,
    )
}
/// Every visible detonation must agree with the local fuse measurement. One
/// orphan or inconsistent delay suppresses the entire arming layer.
pub fn build_replay_bomb_armings(
    reads: &[NavpointRadialRead],
    detonations: &[i64],
    clock: ReplayScoreClock,
) -> ReplayBombArmings {
    let segments = navpoint_radial_segments(reads);
    let mut cov = ReplayBombArmingsCoverage {
        scanned: true,
        reads: reads.len(),
        rises: segments.len(),
        detonations: detonations.len(),
        ..Default::default()
    };
    let mut by_slot = BTreeMap::<u32, Vec<NavpointRadialRead>>::new();
    for r in reads {
        by_slot.entry(r.slot).or_default().push(*r);
    }
    for rs in by_slot.values_mut() {
        rs.sort_by_key(|r| r.time_ms);
    }
    let mut full = Vec::new();
    let mut pauses = BTreeMap::<u32, Vec<NavpointSegment>>::new();
    for segment in segments {
        let g = bomb_truncate_at_first_full(segment, &by_slot[&segment.slot]);
        if g.ends_at_summit() {
            if g.q_max < 254 {
                cov.below_full += 1;
            } else {
                full.push(g);
            }
        } else if g.is_disarm_hold() {
            pauses.entry(g.slot).or_default().push(g);
        }
    }
    let mut armed = Vec::<NavpointSegment>::new();
    for &r in &full {
        if let Some(last) = armed
            .last_mut()
            .filter(|last| r.end_ms.wrapping_sub(last.end_ms) <= 500)
        {
            cov.pair_merged += 1;
            last.start_ms = last.start_ms.min(r.start_ms);
            last.end_ms = r.end_ms;
        } else {
            armed.push(r);
        }
    }
    cov.armed = armed.len();
    let (verdict, ok) = measure_fuse(&full, &pauses, detonations, &mut cov);
    if !ok {
        cov.suppressed = true;
        return ReplayBombArmings {
            coverage: cov,
            verdict,
            ..Default::default()
        };
    }
    let mut out = Vec::new();
    let mut start_zero_fallbacks = 0;
    for r in armed {
        let Some(t) = clock.frame_of(r.end_ms as i64) else {
            cov.out_of_window += 1;
            continue;
        };
        let start_t = clock.frame_of(r.start_ms as i64).unwrap_or_else(|| {
            start_zero_fallbacks += 1;
            0
        });
        out.push(ReplayBombArming {
            t,
            time_ms: r.end_ms as i64,
            start_t,
            start_ms: r.start_ms as i64,
            fuse_ms: verdict.fuse_ms,
        });
    }
    cov.published = out.len();
    ReplayBombArmings {
        armings: out,
        coverage: cov,
        verdict,
        start_zero_fallbacks,
    }
}

/// Assemble the arming layer from retained raw LegacyFilm readings. Recognition is
/// explicit because other modes also write radial-progress navpoints. As native,
/// recognized but unreadable scans publish empty scan coverage rather than facts.
#[allow(dead_code)]
pub(crate) fn build_film_replay_bomb_armings(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    bomb_recognized: bool,
    detonation_times_ms: &[i64],
) -> Option<ReplayBombArmings> {
    bomb_recognized.then(|| {
        build_replay_bomb_armings(
            replay_bomb_radial_reads(film),
            detonation_times_ms,
            ReplayScoreClock {
                origin_ms: players.origin_ms.unwrap_or(0),
                interval_ms: players.frame_interval_ms,
                frames: players.clock.frame_count,
            },
        )
    })
}

// Native decodeFilmBombReads returns no replay readings after a scan error.
// LegacyFilm retains the partial scan separately for inspection and portable export.
pub(super) fn replay_bomb_radial_reads(film: &LegacyFilm) -> &[NavpointRadialRead] {
    film.navpoint_radial
        .as_ref()
        .filter(|_| film.navpoint_radial_error.is_none())
        .map_or(&[], |scan| scan.reads.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_radial_segments_and_complete_bomb_armings() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-bomb-armings-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let reads: Vec<NavpointRadialRead> =
                serde_json::from_value(r["reads"].clone()).unwrap();
            let detonations: Vec<i64> = serde_json::from_value(r["detonations"].clone()).unwrap();
            let clock: ReplayScoreClock = serde_json::from_value(r["clock"].clone()).unwrap();
            let segments = navpoint_radial_segments(&reads);
            assert_eq!(
                segments,
                serde_json::from_value::<Vec<NavpointSegment>>(r["segments"].clone()).unwrap(),
                "segments {i}"
            );
            assert_eq!(
                navpoint_contiguous_rises(&reads),
                serde_json::from_value::<Vec<NavpointRise>>(r["rises"].clone()).unwrap(),
                "rises {i}"
            );
            assert_eq!(
                segments
                    .iter()
                    .map(|s| [s.ends_at_summit(), s.is_disarm_hold()])
                    .collect::<Vec<_>>(),
                serde_json::from_value::<Vec<[bool; 2]>>(r["shapes"].clone()).unwrap(),
                "shapes {i}"
            );
            let mut by_slot = BTreeMap::<u32, Vec<NavpointRadialRead>>::new();
            for read in &reads {
                by_slot.entry(read.slot).or_default().push(*read);
            }
            for rs in by_slot.values_mut() {
                rs.sort_by_key(|r| r.time_ms);
            }
            let truncated: Vec<_> = segments
                .iter()
                .map(|s| bomb_truncate_at_first_full(*s, &by_slot[&s.slot]))
                .collect();
            let expected = if r["truncated"].is_null() {
                vec![]
            } else {
                serde_json::from_value::<Vec<NavpointSegment>>(r["truncated"].clone()).unwrap()
            };
            assert_eq!(truncated, expected, "first full {i}");
            let mut expected: ReplayBombArmings =
                serde_json::from_value(r["output"].clone()).unwrap();
            // Preserve native IEEE bits; JSON decimal parsing can round one ULP away.
            expected.verdict.cv = f64::from_bits(r["cv_bits"].as_u64().unwrap());
            assert_eq!(
                build_replay_bomb_armings(&reads, &detonations, clock),
                expected,
                "armings {i}"
            );
        }
    }
    #[test]
    fn native_bomb_start_fallback_boundaries() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/bomb-start-fallback-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 288);
        let mut positive = 0;
        let mut hits = 0;
        for (i, row) in rows.iter().enumerate() {
            let reads =
                serde_json::from_value::<Vec<NavpointRadialRead>>(row["reads"].clone()).unwrap();
            let detonations =
                serde_json::from_value::<Vec<i64>>(row["detonations"].clone()).unwrap();
            let clock: ReplayScoreClock = serde_json::from_value(row["clock"].clone()).unwrap();
            let mut expected: ReplayBombArmings =
                serde_json::from_value(row["output"].clone()).unwrap();
            expected.verdict.cv = f64::from_bits(row["cv_bits"].as_u64().unwrap());
            let actual = build_replay_bomb_armings(&reads, &detonations, clock);
            assert_eq!(actual, expected, "boundary case {i}");
            if actual.start_zero_fallbacks > 0 {
                positive += 1;
                hits += actual.start_zero_fallbacks;
                assert!(detonations.is_empty());
                assert!(clock.origin_ms > 0 && clock.origin_ms <= 900);
                assert!(clock.interval_ms > 0 && clock.frames > 0);
                // Mirrored slots represent one arming, not two fallback hits.
                assert_eq!(actual.armings.len(), 1);
                assert_eq!(actual.start_zero_fallbacks, 1);
            }
            if !detonations.is_empty()
                || clock.origin_ms <= 0
                || clock.origin_ms > 900
                || clock.interval_ms == 0
                || clock.frames == 0
            {
                assert_eq!(actual.start_zero_fallbacks, 0, "negative case {i}");
            }
        }
        assert_eq!((positive, hits), (24, 24));
    }
    #[test]
    fn failed_radial_scan_is_retained_but_not_published_as_bomb_armings() {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        let chunk = FilmChunkData {
            metadata: FilmChunk {
                index: 0,
                chunk_type: 1,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: bootstrap.len() as i64,
                file_relative_path: String::new(),
            },
            data: bootstrap,
        };
        let mut payload = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/captured-keyframe-v41.zlib")[..])
            .read_to_end(&mut payload)
            .unwrap();
        let mut packet = vec![2, 0, 0, 0];
        packet.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        packet.extend_from_slice(&1000u64.to_le_bytes());
        packet.extend(payload);
        let replication = FilmChunkData {
            metadata: FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 1,
                size: packet.len() as i64,
                file_relative_path: String::new(),
            },
            data: packet,
        };
        let mut film =
            LegacyFilm::try_from_chunks(&[chunk, replication], DecodeOptions::v41()).unwrap();
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/bomb-start-fallback-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        let row = rows
            .iter()
            .find(|r| r["output"]["start_zero_fallbacks"] == 1)
            .unwrap();
        film.navpoint_radial = Some(NavpointRadialScan {
            reads: serde_json::from_value(row["reads"].clone()).unwrap(),
            ..Default::default()
        });
        let clock: ReplayScoreClock = serde_json::from_value(row["clock"].clone()).unwrap();
        let successful = build_replay_bomb_armings(replay_bomb_radial_reads(&film), &[], clock);
        assert_eq!(successful.start_zero_fallbacks, 1);
        assert_eq!(successful.armings.len(), 1);
        // Explicitly injected partial-result/error pair tests the LegacyFilm adapter
        // contract, not a claim that this source failure occurs after these reads.
        film.navpoint_radial_error = Some("radial scan failed".into());
        let failed = build_replay_bomb_armings(replay_bomb_radial_reads(&film), &[], clock);
        assert!(failed.armings.is_empty());
        assert_eq!(failed.coverage.reads, 0);
        assert_eq!(failed.start_zero_fallbacks, 0);
        assert!(!film.navpoint_radial.as_ref().unwrap().reads.is_empty());
        let restored: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
        assert_eq!(restored.navpoint_radial, film.navpoint_radial);
        assert_eq!(restored.navpoint_radial_error, film.navpoint_radial_error);
        assert!(replay_bomb_radial_reads(&restored).is_empty());
        film.navpoint_radial_error = None;
        assert_eq!(
            build_replay_bomb_armings(replay_bomb_radial_reads(&film), &[], clock),
            successful
        );
        film.navpoint_radial = None;
        assert!(replay_bomb_radial_reads(&film).is_empty());
    }
}
