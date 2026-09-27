//! Complete capture-based zone owner layer, before the hill-specific fallback.
use super::*;
use std::collections::BTreeMap;
pub struct ReplayZoneOwnerInput<'a> {
    pub series: &'a ReplayZoneSeries,
    pub pairs: &'a [ReplayZonePair],
    pub teams: &'a BTreeMap<String, i64>,
    pub frames: i64,
    pub interval_ms: i64,
    pub catalog: usize,
    pub hill: bool,
}
/// Returns published states and the number of capture-team inferences from ramp outcomes.
/// Coverage is accumulated in place, as in the native assembly pipeline.
pub fn build_replay_zone_owner_layer(
    input: ReplayZoneOwnerInput<'_>,
    coverage: &mut ReplayZonesCoverage,
) -> (Vec<ReplayZoneState>, usize) {
    let ReplayZoneOwnerInput {
        series,
        pairs,
        teams,
        frames,
        interval_ms,
        catalog,
        hill,
    } = input;
    let win = replay_zone_window_frames(interval_ms);
    let (gauges, unpaired) = pair_replay_zone_gauges(&series.ramps(), pairs, win);
    coverage.paired = gauges.len();
    coverage.unpaired = unpaired;
    let owners = elect_replay_zone_owners(replay_zone_owner_candidates(series, pairs, teams, win));
    coverage.method = "captures+geometry".into();
    let refs: Vec<_> = gauges
        .keys()
        .filter(|r| owners.contains_key(r))
        .copied()
        .collect();
    coverage.owner_unpaired = gauges.len() - refs.len();
    let team_set = replay_zone_team_set(teams);
    let letters = replay_zone_letter_ranks(&gauges, catalog as i64, hill);
    let mut out = Vec::new();
    let mut inferred = 0;
    for zone_ref in refs {
        let slot = gauges[&zone_ref];
        let gauge = &series.gauge[&slot];
        let owner = &series.owner[&owners[&zone_ref]];
        let spans = replay_zone_owner_spans(owner, gauge, frames, &team_set, coverage);
        if spans.is_empty() {
            continue;
        }
        let mut one = ReplayZoneSeries::default();
        one.gauge.insert(slot, gauge.clone());
        let ramps = one.ramps();
        let samples: Vec<_> = gauge
            .iter()
            .map(|s| ReplayGaugeSample { t: s.t, v: s.v })
            .collect();
        let windows: Vec<_> = ramps.iter().map(|r| (r.t0, r.t_peak)).collect();
        let capturer = elect_replay_zone_capturer(series, &ramps, owner, owners[&zone_ref], win);
        let (gauge_ramps, n) = replay_zone_gauge_ramps(&ramps, owner, &capturer, &team_set, win);
        inferred += n;
        out.push(ReplayZoneState {
            zone_ref,
            letter_rank: letters.get(&zone_ref).copied(),
            key: series.keys.get(&slot).copied().unwrap_or(0),
            spans,
            gauge: replay_gauge_series(&samples, &windows, replay_gauge_gap_frames(interval_ms)),
            gauge_ramps,
        });
    }
    check_replay_zone_owner_agreement(series, &owners, pairs, teams, win, coverage);
    (out, inferred)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        series: ReplayZoneSeries,
        pairs: Vec<ReplayZonePair>,
        teams: BTreeMap<String, i64>,
        frames: i64,
        interval: i64,
        catalog: usize,
        hill: bool,
        states: Vec<ReplayZoneState>,
        coverage: ReplayZonesCoverage,
        inferred: usize,
    }
    #[test]
    fn native_complete_zone_owner_layer() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-zone-owner-layer-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let mut coverage = ReplayZonesCoverage::default();
            let (states, n) = build_replay_zone_owner_layer(
                ReplayZoneOwnerInput {
                    series: &c.series,
                    pairs: &c.pairs,
                    teams: &c.teams,
                    frames: c.frames,
                    interval_ms: c.interval,
                    catalog: c.catalog,
                    hill: c.hill,
                },
                &mut coverage,
            );
            tally_replay_zone_states(&states, &mut coverage);
            assert_eq!(states, c.states, "states {i}");
            assert_eq!(coverage, c.coverage, "coverage {i}");
            assert_eq!(n, c.inferred, "inferred {i}");
        }
    }
}
