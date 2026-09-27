//! Oddball tick trains and conservative carrier presence publication.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayPresenceSpan {
    pub t0: i64,
    pub t1: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCarrierPresence {
    pub named: BTreeMap<String, Vec<ReplayPresenceSpan>>,
    pub unnamed: Vec<ReplayPresenceSpan>,
}
impl ReplayCarrierPresence {
    pub fn from_tracks(tracks: &[ReplayTrack], deduced: &BTreeSet<usize>) -> Self {
        let mut out = Self::default();
        for (i, t) in tracks.iter().enumerate() {
            let span = ReplayPresenceSpan {
                t0: t.start_frame,
                t1: t.end_frame,
            };
            if !t.xuid.is_empty() {
                out.named.entry(t.xuid.clone()).or_default().push(span);
                if deduced.contains(&i) {
                    out.unnamed.push(span);
                }
            } else if t.bot.is_empty() {
                out.unnamed.push(span);
            }
        }
        out
    }
    /// Unknown identity prevents rejection or trimming. Otherwise retain the union
    /// of overlapping named lifetimes, including gaps between those lifetimes.
    pub fn gate(&self, xuid: &str, t0: i64, t1: i64) -> Option<ReplayPresenceSpan> {
        let original = ReplayPresenceSpan { t0, t1 };
        let Some(spans) = self.named.get(xuid).filter(|s| !s.is_empty()) else {
            return Some(original);
        };
        if presence_union(&self.unnamed, t0, t1).is_some() {
            return Some(original);
        }
        presence_union(spans, t0, t1).map(|s| ReplayPresenceSpan {
            t0: t0.max(s.t0),
            t1: t1.min(s.t1),
        })
    }
}
fn presence_union(spans: &[ReplayPresenceSpan], t0: i64, t1: i64) -> Option<ReplayPresenceSpan> {
    let mut out: Option<ReplayPresenceSpan> = None;
    for s in spans {
        if t1.min(s.t1) < t0.max(s.t0) {
            continue;
        }
        out = Some(match out {
            None => *s,
            Some(p) => ReplayPresenceSpan {
                t0: p.t0.min(s.t0),
                t1: p.t1.max(s.t1),
            },
        });
    }
    out
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaySkullRawCarry {
    pub xuid: String,
    pub round: i64,
    pub t0_ms: i64,
    pub t1_ms: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaySkullCarry {
    pub xuid: String,
    pub t0: i64,
    pub t1: i64,
    pub closed: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaySkullCoverage {
    pub skull_film: bool,
    pub grabs: i64,
    pub trains: usize,
    pub carries: usize,
    pub closed: usize,
    pub open: usize,
    pub no_bridge: usize,
    pub out_of_window: usize,
    pub carrier_absent: usize,
}
impl ReplaySkullCoverage {
    pub fn balanced(&self) -> bool {
        self.carries + self.no_bridge + self.out_of_window + self.carrier_absent == self.trains
            && self.closed + self.open == self.carries
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaySkullCarries {
    pub carries: Vec<ReplaySkullCarry>,
    pub coverage: Option<ReplaySkullCoverage>,
}
fn skull_series(records: &[StatborgRecord], grabs: bool) -> StatborgRoundSeries {
    statborg_series_by_round(
        records,
        StatborgCounterKey {
            component: if grabs { 21 } else { 0 },
            side: if grabs {
                StatborgSide::B
            } else {
                StatborgSide::A
            },
        },
        false,
        false,
        false,
    )
}
// Repeated units at one timestamp neither change train bounds nor contribute a
// positive tick-width gap. Retain one instant per positive counter emission.
fn skull_instants(points: &[StatborgScorePoint]) -> Vec<i64> {
    let mut prev = 0;
    points
        .iter()
        .filter_map(|p| {
            if p.value > prev {
                prev = p.value;
                Some(p.time_ms)
            } else {
                None
            }
        })
        .collect()
}
pub fn replay_skull_tick_width(records: &[StatborgRecord], clock: ReplayMatchClock) -> i64 {
    let mut gaps = Vec::new();
    for rounds in skull_series(records, false).values() {
        for pts in rounds.values() {
            for pair in skull_instants(pts).windows(2) {
                let d = pair[1].wrapping_sub(pair[0]);
                if d > 0 && d <= 3000 {
                    gaps.push(d);
                }
            }
        }
    }
    gaps.sort_unstable();
    if gaps.is_empty() {
        0
    } else {
        clock.slack_frames(gaps[gaps.len() / 2])
    }
}
pub fn reconstruct_skull_carries(
    records: &[StatborgRecord],
    identity: &StatborgRoundIdentity,
) -> Vec<ReplaySkullRawCarry> {
    let mut out = Vec::new();
    for (slot, rounds) in skull_series(records, false) {
        for (round, pts) in rounds {
            let times = skull_instants(&pts);
            let Some(&first) = times.first() else {
                continue;
            };
            let xuid = identity
                .publication
                .by_round
                .get(&round)
                .and_then(|r| r.get(&slot))
                .cloned()
                .unwrap_or_default();
            let (mut start, mut last) = (first, first);
            for &t in &times[1..] {
                if t.wrapping_sub(last) > 3000 {
                    out.push(ReplaySkullRawCarry {
                        xuid: xuid.clone(),
                        round,
                        t0_ms: start,
                        t1_ms: last,
                    });
                    start = t;
                }
                last = t;
            }
            out.push(ReplaySkullRawCarry {
                xuid,
                round,
                t0_ms: start,
                t1_ms: last,
            });
        }
    }
    super::native_sort::sort_by(&mut out, |a, b| {
        (a.t0_ms, a.round, &a.xuid).cmp(&(b.t0_ms, b.round, &b.xuid))
    });
    out
}
pub fn build_replay_skull_carries(
    scanned: bool,
    records: &[StatborgRecord],
    identity: &StatborgRoundIdentity,
    clock: ReplayMatchClock,
    presence: &ReplayCarrierPresence,
) -> ReplaySkullCarries {
    if !scanned {
        return ReplaySkullCarries::default();
    }
    let raw = reconstruct_skull_carries(records, identity);
    let grabs = skull_series(records, true)
        .values()
        .flat_map(|r| r.values())
        .filter_map(|p| p.last())
        .fold(0i64, |s, p| s.wrapping_add(p.value));
    let mut cov = ReplaySkullCoverage {
        skull_film: true,
        grabs,
        trains: raw.len(),
        ..Default::default()
    };
    let width = replay_skull_tick_width(records, clock);
    let half = if width <= 1 { 0 } else { (width - 1) / 2 };
    let threshold = clock
        .frames
        .wrapping_sub(1)
        .wrapping_sub(clock.slack_frames(3000));
    let clamp = |t: i64| {
        if t < 0 {
            0
        } else if t >= clock.frames {
            clock.frames.wrapping_sub(1)
        } else {
            t
        }
    };
    let mut carries = Vec::new();
    for r in raw {
        if r.xuid.is_empty() {
            cov.no_bridge += 1;
            continue;
        }
        let t0 = clock.frame_of_match_ms(r.t0_ms);
        if t0 < 0 || t0 >= clock.frames {
            cov.out_of_window += 1;
            continue;
        }
        let t1 = clamp(clock.frame_of_match_ms(r.t1_ms)).max(t0);
        let Some(span) = presence.gate(
            &r.xuid,
            t0.wrapping_sub(half).max(0),
            clamp(t1.wrapping_add(half)),
        ) else {
            cov.carrier_absent += 1;
            continue;
        };
        let closed = span.t1 < threshold;
        if closed {
            cov.closed += 1;
        } else {
            cov.open += 1;
        }
        carries.push(ReplaySkullCarry {
            xuid: r.xuid,
            t0: span.t0,
            t1: span.t1,
            closed,
        });
    }
    cov.carries = carries.len();
    ReplaySkullCarries {
        carries,
        coverage: Some(cov),
    }
}
/// Use the completed per-round bridge already resolved by player assembly.
/// Mode recognition is supplied by the caller, since component zero is shared.
#[allow(dead_code)]
pub(crate) fn build_film_replay_skull_carries(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    oddball_recognized: bool,
) -> ReplaySkullCarries {
    build_replay_skull_carries(
        oddball_recognized,
        &film.statborg.records,
        &players.statborg_identity,
        ReplayMatchClock {
            origin_us: players.clock.origin_us,
            step_us: players.clock.step_us,
            frames: players.clock.frame_count,
            death_offset_ms: players.registry.owners.clock.offset_ms,
        },
        &ReplayCarrierPresence::from_tracks(
            &players.players.publication.tracks,
            &players.players.naming.deduced,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_skull_carries_and_presence() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-skull-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let records: Vec<StatborgRecord> =
                serde_json::from_value(r["records"].clone()).unwrap();
            let deaths: Vec<IdentityDeath> = serde_json::from_value(r["deaths"].clone()).unwrap();
            let tracks: Vec<ReplayTrack> = serde_json::from_value(r["tracks"].clone()).unwrap();
            let deduced: BTreeMap<usize, bool> =
                serde_json::from_value(r["deduced"].clone()).unwrap();
            let deduced = deduced
                .into_iter()
                .filter_map(|(i, b)| b.then_some(i))
                .collect();
            let presence = ReplayCarrierPresence::from_tracks(&tracks, &deduced);
            assert_eq!(
                presence,
                serde_json::from_value::<ReplayCarrierPresence>(r["presence"].clone()).unwrap(),
                "presence {i}"
            );
            for g in r["gates"].as_array().unwrap() {
                assert_eq!(
                    presence.gate(
                        g["xuid"].as_str().unwrap(),
                        g["t0"].as_i64().unwrap(),
                        g["t1"].as_i64().unwrap()
                    ),
                    serde_json::from_value::<Option<ReplayPresenceSpan>>(g["result"].clone())
                        .unwrap(),
                    "gate {i}"
                );
            }
            let clock: ReplayMatchClock = serde_json::from_value(r["clock"].clone()).unwrap();
            let instants: Vec<_> = deaths
                .iter()
                .map(|d| StatborgDeathInstant {
                    xuid: d.xuid.to_string(),
                    time_ms: d.time_ms,
                })
                .collect();
            let identity = resolve_statborg_round_identity(&records, &instants);
            // Native map traversal leaves equal (start, round, xuid) keys
            // unordered. Compare every raw period with a final end-time tie break.
            let mut actual = reconstruct_skull_carries(&records, &identity);
            let mut expected: Vec<ReplaySkullRawCarry> =
                serde_json::from_value(r["raw"].clone()).unwrap();
            let order = |a: &ReplaySkullRawCarry, b: &ReplaySkullRawCarry| {
                (a.t0_ms, a.round, &a.xuid, a.t1_ms).cmp(&(b.t0_ms, b.round, &b.xuid, b.t1_ms))
            };
            actual.sort_by(order);
            expected.sort_by(order);
            assert_eq!(actual, expected, "raw skull {i}");
            assert_eq!(
                replay_skull_tick_width(&records, clock),
                r["width"].as_i64().unwrap(),
                "tick width {i}"
            );
            let output = build_replay_skull_carries(
                r["scanned"].as_bool().unwrap(),
                &records,
                &identity,
                clock,
                &presence,
            );
            assert_eq!(
                output,
                serde_json::from_value::<ReplaySkullCarries>(r["output"].clone()).unwrap(),
                "skull {i}"
            );
            assert!(
                output
                    .coverage
                    .as_ref()
                    .is_none_or(ReplaySkullCoverage::balanced)
            );
        }
    }
}
