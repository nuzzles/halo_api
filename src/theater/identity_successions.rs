//! Bot replacement chains from supplied match arrival times and indexed fire.
use super::{IdentityFireReference, ReplayTrackIdentity};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentitySuccession {
    pub bot_name: super::ReplayByteString,
    pub film_index: i64,
    #[serde(rename = "SwitchMatchMS")]
    pub switch_match_ms: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentitySuccessionReport {
    pub claimed: usize,
    pub halted: usize,
    pub lifted_by_fire: usize,
}
fn time(frame: i64, origin: u64, step: u64) -> i64 {
    (origin as i64).wrapping_add(frame.wrapping_mul(step as i64))
}
/// Native half-open track containment uses unsigned clock arithmetic.
pub fn identity_track_contains(
    track: &ReplayTrackIdentity,
    origin: u64,
    step: u64,
    timestamp_us: u64,
) -> bool {
    let from = origin.wrapping_add((track.start_frame as u64).wrapping_mul(step));
    let to = origin.wrapping_add((track.end_frame.wrapping_add(1) as u64).wrapping_mul(step));
    timestamp_us >= from && timestamp_us < to
}
/// Select the sole anonymous birth in the window, or the sole candidate with
/// unambiguous fire evidence. Fire shared by overlapping tracks casts no vote.
pub fn identity_succession_candidate(
    tracks: &[ReplayTrackIdentity],
    origin: u64,
    step: u64,
    window: [i64; 2],
    film_index: i64,
    fire: &[IdentityFireReference],
) -> (Option<usize>, bool) {
    let candidates: Vec<_> = tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            let start = time(t.start_frame, origin, step);
            t.xuid.is_empty() && t.bot.is_empty() && start >= window[0] && start <= window[1]
        })
        .map(|(i, _)| i)
        .collect();
    if candidates.len() == 1 {
        return (Some(candidates[0]), false);
    }
    if candidates.len() < 2 {
        return (None, false);
    }
    let mut voted = None;
    for f in fire.iter().filter(|f| f.film_index == film_index) {
        let mut holders = candidates
            .iter()
            .copied()
            .filter(|&i| identity_track_contains(&tracks[i], origin, step, f.timestamp_us));
        let Some(holder) = holders.next() else {
            continue;
        };
        if holders.next().is_some() {
            continue;
        }
        if voted.is_some_and(|v| v != holder) {
            return (None, false);
        }
        voted = Some(holder);
    }
    (voted, voted.is_some())
}
/// Process replacements in supplied order, as native production does. A matched
/// death clock is required. Claimed tracks leave the candidate pool immediately.
pub fn attribute_identity_successions(
    tracks: &mut [ReplayTrackIdentity],
    successions: &[IdentitySuccession],
    origin: u64,
    step: u64,
    death_offset_ms: i64,
    offset_matches: usize,
    fire: &[IdentityFireReference],
) -> IdentitySuccessionReport {
    let mut report = IdentitySuccessionReport::default();
    if successions.is_empty() || offset_matches == 0 {
        return report;
    }
    for s in successions {
        let switch = s
            .switch_match_ms
            .wrapping_add(death_offset_ms)
            .wrapping_mul(1000);
        let mut window = [
            switch.wrapping_sub(2_000_000),
            switch.wrapping_add(20_000_000),
        ];
        loop {
            let (candidate, lifted) =
                identity_succession_candidate(tracks, origin, step, window, s.film_index, fire);
            let Some(i) = candidate else {
                report.halted += 1;
                break;
            };
            report.lifted_by_fire += usize::from(lifted);
            tracks[i].bot = s.bot_name.clone();
            report.claimed += 1;
            let end = time(tracks[i].end_frame, origin, step);
            window = [end.wrapping_add(2_000_000), end.wrapping_add(25_000_000)];
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::{
        IdentityRemainingReport, IdentityTrackNamingInput, ReplayIdentityState,
        name_replay_identity_tracks,
    };
    use std::{collections::BTreeSet, io::Read};
    #[derive(Deserialize)]
    struct Query {
        window: [i64; 2],
        index: i64,
        candidate: Option<usize>,
        lifted: bool,
    }
    #[derive(Deserialize)]
    struct Case {
        tracks: Vec<ReplayTrackIdentity>,
        output: Vec<ReplayTrackIdentity>,
        pipeline: Vec<ReplayTrackIdentity>,
        pipeline_report: IdentityRemainingReport,
        fallbacks: usize,
        state: ReplayIdentityState,
        deduced: BTreeSet<usize>,
        dead: BTreeSet<usize>,
        origin: u64,
        step: u64,
        offset: i64,
        matches: usize,
        fire: Vec<IdentityFireReference>,
        successions: Vec<IdentitySuccession>,
        queries: Vec<Query>,
    }
    #[test]
    fn native_succession_and_track_naming_pipeline() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-successions-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, mut c) in cases.into_iter().enumerate() {
            for q in c.queries {
                assert_eq!(
                    identity_succession_candidate(
                        &c.tracks, c.origin, c.step, q.window, q.index, &c.fire
                    ),
                    (q.candidate, q.lifted),
                    "candidate {i}"
                );
            }
            assert_eq!(
                c.state.deduced_tracks(&c.tracks, c.origin, c.step),
                c.deduced,
                "deduced {i}"
            );
            assert_eq!(
                c.state.death_closed_tracks(&c.tracks, c.origin, c.step),
                c.dead,
                "death closed {i}"
            );
            let mut pipeline = c.tracks.clone();
            let report = name_replay_identity_tracks(
                &mut pipeline,
                IdentityTrackNamingInput {
                    state: &c.state,
                    bots: &[],
                    successions: &c.successions,
                    fire: &c.fire,
                    origin_us: c.origin,
                    step_us: c.step,
                    death_offset_ms: c.offset,
                    offset_matches: c.matches,
                },
            );
            assert_eq!(pipeline, c.pipeline, "pipeline {i}");
            assert_eq!(report.remaining, c.pipeline_report, "pipeline report {i}");
            assert_eq!(
                report.overlap_fallbacks, c.fallbacks,
                "pipeline fallbacks {i}"
            );
            assert_eq!(report.death_closed, c.dead, "pipeline death bounds {i}");
            attribute_identity_successions(
                &mut c.tracks,
                &c.successions,
                c.origin,
                c.step,
                c.offset,
                c.matches,
                &c.fire,
            );
            assert_eq!(c.tracks, c.output, "succession {i}");
        }
    }
}
