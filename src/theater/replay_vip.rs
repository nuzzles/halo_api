//! VIP selection periods, gated by an explicitly recognized game mode.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVipRawPeriod {
    pub slot: i64,
    pub xuid: String,
    pub t0_ms: i64,
    pub t1_ms: i64,
    pub closed: bool,
    pub cause: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVipPeriod {
    pub xuid: String,
    pub t0: i64,
    pub t1: i64,
    pub closed: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayVipCoverage {
    pub vip_film: bool,
    pub selections: usize,
    pub periods: usize,
    pub closed: usize,
    pub open: usize,
    pub closed_by_death: usize,
    pub closed_by_selection: usize,
    pub no_bridge: usize,
    pub out_of_window: usize,
}
impl ReplayVipCoverage {
    pub fn balanced(&self) -> bool {
        self.periods + self.no_bridge + self.out_of_window == self.selections
            && self.closed + self.open == self.periods
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVipCrown {
    pub periods: Vec<ReplayVipPeriod>,
    pub coverage: Option<ReplayVipCoverage>,
}
/// A later selection of the same statborg slot closes the previous period even
/// if identity changes. On equal closure times, written death wins the tie.
pub fn reconstruct_vip_periods(
    events: &[StatborgNamedEvent],
    identity: &StatborgRoundIdentity,
    deaths: &[IdentityDeath],
    end_ms: i64,
) -> Vec<ReplayVipRawPeriod> {
    reconstruct_vip_values(
        events,
        identity,
        &deaths
            .iter()
            .map(|d| (d.xuid, d.time_ms))
            .collect::<Vec<_>>(),
        end_ms,
    )
}
fn reconstruct_vip_values(
    events: &[StatborgNamedEvent],
    identity: &StatborgRoundIdentity,
    deaths: &[(u64, i64)],
    end_ms: i64,
) -> Vec<ReplayVipRawPeriod> {
    let mut out: Vec<_> = events
        .iter()
        .filter(|e| e.stat == "vip_selected")
        .map(|e| ReplayVipRawPeriod {
            slot: e.slot,
            xuid: identity.at(e.slot, e.time_ms).into(),
            t0_ms: e.time_ms,
            t1_ms: end_ms,
            closed: false,
            cause: "fin".into(),
        })
        .collect();
    out.sort_by_key(|r| (r.t0_ms, r.slot));
    let mut by_xuid = BTreeMap::<String, Vec<i64>>::new();
    for d in deaths {
        by_xuid.entry(d.0.to_string()).or_default().push(d.1);
    }
    for times in by_xuid.values_mut() {
        times.sort_unstable();
    }
    let mut last = BTreeMap::new();
    let mut next = BTreeMap::new();
    for (i, r) in out.iter().enumerate() {
        if let Some(prev) = last.insert(r.slot, i) {
            next.insert(prev, r.t0_ms);
        }
    }
    for (i, r) in out.iter_mut().enumerate() {
        if let Some(times) = by_xuid.get(&r.xuid) {
            let at = times.partition_point(|t| *t <= r.t0_ms);
            if let Some(&d) = times.get(at).filter(|d| **d < r.t1_ms) {
                r.t1_ms = d;
                r.cause = "mort".into();
            }
        }
        if let Some(&n) = next.get(&i).filter(|n| **n < r.t1_ms) {
            r.t1_ms = n;
            r.cause = "selection".into();
        }
        r.closed = r.cause != "fin";
    }
    out
}
pub fn build_replay_vip_crown(
    scanned: bool,
    events: &[StatborgNamedEvent],
    identity: &StatborgRoundIdentity,
    deaths: &[IdentityDeath],
    clock: ReplayMatchClock,
) -> ReplayVipCrown {
    build_vip_values(
        scanned,
        events,
        identity,
        &deaths
            .iter()
            .map(|d| (d.xuid, d.time_ms))
            .collect::<Vec<_>>(),
        clock,
    )
}
pub(super) fn build_vip_values(
    scanned: bool,
    events: &[StatborgNamedEvent],
    identity: &StatborgRoundIdentity,
    deaths: &[(u64, i64)],
    clock: ReplayMatchClock,
) -> ReplayVipCrown {
    if !scanned {
        return ReplayVipCrown::default();
    }
    let end = events
        .iter()
        .map(|e| e.time_ms)
        .chain(deaths.iter().map(|d| d.1))
        .fold(0, i64::max)
        .wrapping_add(1)
        .max(clock.match_ms_of_frame(clock.frames.wrapping_sub(1)));
    let raw = reconstruct_vip_values(events, identity, deaths, end);
    let mut coverage = ReplayVipCoverage {
        vip_film: true,
        selections: raw.len(),
        ..Default::default()
    };
    let mut periods = Vec::new();
    for r in raw {
        if r.xuid.is_empty() {
            coverage.no_bridge += 1;
            continue;
        }
        let t0 = clock.frame_of_match_ms(r.t0_ms);
        if t0 < 0 || t0 >= clock.frames {
            coverage.out_of_window += 1;
            continue;
        }
        let t1 = clock.frame_of_match_ms(r.t1_ms);
        let t1 = if t1 < 0 {
            0
        } else if t1 >= clock.frames {
            clock.frames.wrapping_sub(1)
        } else {
            t1
        };
        periods.push(ReplayVipPeriod {
            xuid: r.xuid,
            t0,
            t1: t1.max(t0),
            closed: r.closed,
        });
        if r.closed {
            coverage.closed += 1;
            if r.cause == "mort" {
                coverage.closed_by_death += 1;
            } else if r.cause == "selection" {
                coverage.closed_by_selection += 1;
            }
        } else {
            coverage.open += 1;
        }
    }
    coverage.periods = periods.len();
    ReplayVipCrown {
        periods,
        coverage: Some(coverage),
    }
}
/// The caller must recognize VIP independently: component 22 means a different
/// objective counter in CTF. Do not infer the mode from counter values.
#[allow(dead_code)]
pub(crate) fn build_film_replay_vip_crown(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    vip_recognized: bool,
) -> ReplayVipCrown {
    build_film_replay_vip_crown_observed(film, players, vip_recognized, None)
}

/// Retain native VIP named-event diagnostics, respecting the recognition gate.
#[allow(dead_code)]
pub(crate) fn build_film_replay_vip_crown_with_diagnostics(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    vip_recognized: bool,
) -> (ReplayVipCrown, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out =
        build_film_replay_vip_crown_observed(film, players, vip_recognized, Some(&mut diagnostics));
    (out, diagnostics)
}

fn build_film_replay_vip_crown_observed(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    vip_recognized: bool,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> ReplayVipCrown {
    vip_crown_from_records(
        &film.statborg.records,
        &players.evidence.deaths,
        ReplayMatchClock {
            origin_us: players.clock.origin_us,
            step_us: players.clock.step_us,
            frames: players.clock.frame_count,
            death_offset_ms: players.registry.owners.clock.offset_ms,
        },
        vip_recognized,
        diagnostics,
    )
}

pub(super) fn vip_crown_from_records(
    records: &[StatborgRecord],
    deaths: &[IdentityDeath],
    clock: ReplayMatchClock,
    recognized: bool,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> ReplayVipCrown {
    if !recognized {
        return ReplayVipCrown::default();
    }
    let events = super::statborg_named::named_events_with_sink(records, "vip", diagnostics);
    let instants: Vec<_> = deaths
        .iter()
        .map(|d| StatborgDeathInstant {
            xuid: d.xuid.to_string(),
            time_ms: d.time_ms,
        })
        .collect();
    let identity = resolve_statborg_round_identity(records, &instants);
    build_replay_vip_crown(true, &events, &identity, deaths, clock)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_flag_and_vip_diagnostic_gates() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/objective-diagnostics-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 128);
        let mut warnings = [0; 2];
        let mut periods = 0;
        for (i, row) in rows.into_iter().enumerate() {
            let records: Vec<StatborgRecord> =
                serde_json::from_value(row["records"].clone()).unwrap();
            let deaths: Vec<IdentityDeath> = serde_json::from_value(row["deaths"].clone()).unwrap();
            let bursts: Vec<i64> = serde_json::from_value(row["bursts"].clone()).unwrap();
            let clock: ReplayMatchClock = serde_json::from_value(row["clock"].clone()).unwrap();
            let mut flag_logs = Vec::new();
            let (signals, events) = super::super::replay_flags_film::flag_named_inputs(
                &records,
                &bursts,
                Some(&mut flag_logs),
            );
            assert_eq!(
                signals,
                serde_json::from_value::<ReplayFlagFilmSignals>(row["signals"].clone()).unwrap(),
                "signals {i}"
            );
            assert_eq!(
                events,
                serde_json::from_value::<Vec<StatborgNamedEvent>>(row["events"].clone()).unwrap(),
                "events {i}"
            );
            assert_eq!(
                flag_logs,
                serde_json::from_value::<Vec<StatborgDiagnostic>>(row["flag_logs"].clone())
                    .unwrap(),
                "flag logs {i}"
            );
            let recognized = row["recognized"].as_bool().unwrap();
            let mut vip_logs = Vec::new();
            let vip =
                vip_crown_from_records(&records, &deaths, clock, recognized, Some(&mut vip_logs));
            assert_eq!(
                vip,
                serde_json::from_value::<ReplayVipCrown>(row["vip"].clone()).unwrap(),
                "VIP {i}"
            );
            assert_eq!(
                vip,
                vip_crown_from_records(&records, &deaths, clock, recognized, None),
                "unobserved {i}"
            );
            assert_eq!(
                vip_logs,
                serde_json::from_value::<Vec<StatborgDiagnostic>>(row["vip_logs"].clone()).unwrap(),
                "VIP logs {i}"
            );
            if !recognized {
                assert!(vip_logs.is_empty());
            }
            warnings[0] += flag_logs.len();
            warnings[1] += vip_logs.len();
            periods += vip.periods.len();
        }
        assert!(warnings.iter().all(|n| *n > 0) && periods > 0);
        println!("flag/VIP warnings {warnings:?}; VIP periods {periods}");
    }
    #[test]
    fn native_vip_periods_and_match_clock() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-vip-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let records: Vec<StatborgRecord> =
                serde_json::from_value(r["records"].clone()).unwrap();
            let deaths: Vec<IdentityDeath> = serde_json::from_value(r["deaths"].clone()).unwrap();
            let events: Vec<StatborgNamedEvent> =
                serde_json::from_value(r["events"].clone()).unwrap();
            let clock: ReplayMatchClock = serde_json::from_value(r["clock"].clone()).unwrap();
            let instants: Vec<_> = deaths
                .iter()
                .map(|d| StatborgDeathInstant {
                    xuid: d.xuid.to_string(),
                    time_ms: d.time_ms,
                })
                .collect();
            let identity = resolve_statborg_round_identity(&records, &instants);
            assert_eq!(
                reconstruct_vip_periods(&events, &identity, &deaths, 100000),
                serde_json::from_value::<Vec<ReplayVipRawPeriod>>(r["raw"].clone()).unwrap(),
                "raw VIP {i}"
            );
            assert_eq!(
                build_replay_vip_crown(
                    r["scanned"].as_bool().unwrap(),
                    &events,
                    &identity,
                    &deaths,
                    clock
                ),
                serde_json::from_value::<ReplayVipCrown>(r["output"].clone()).unwrap(),
                "VIP {i}"
            );
            for p in r["probes"].as_array().unwrap() {
                let v = p[0].as_i64().unwrap();
                assert_eq!(
                    [
                        clock.frame_of_match_ms(v),
                        clock.match_ms_of_frame(v),
                        clock.slack_frames(v)
                    ],
                    [
                        p[1].as_i64().unwrap(),
                        p[2].as_i64().unwrap(),
                        p[3].as_i64().unwrap()
                    ],
                    "clock {i}/{v}"
                );
            }
        }
    }
}
