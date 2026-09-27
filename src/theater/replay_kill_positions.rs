//! Native kill placement and a time-shifted engagement-opening proxy.
//! Opening estimates are derived, not recorded first-damage events.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const REPLAY_OPENING_LEAD_MS: i64 = 1500;
pub const REPLAY_KILL_POSITION_TOLERANCE_US: u64 = 120_000;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayKillPosition {
    #[serde(flatten)]
    pub kill: ReplayKillReference,
    pub killer: Option<[f64; 3]>,
    pub victim: Option<[f64; 3]>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayKillPositionReport {
    pub kills: usize,
    pub both: usize,
    pub killer_only: usize,
    pub victim_only: usize,
    pub dropped: usize,
    pub no_bridge: usize,
    pub opening_out_of_life: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayKillPositions {
    pub positions: Vec<ReplayKillPosition>,
    pub report: ReplayKillPositionReport,
}
pub fn shift_replay_kill_references(
    kills: &[ReplayKillReference],
    delta_ms: i64,
) -> Vec<ReplayKillReference> {
    kills
        .iter()
        .map(|k| ReplayKillReference {
            time_ms: k.time_ms.wrapping_add(delta_ms),
            ..*k
        })
        .collect()
}
fn count(report: &mut ReplayKillPositionReport, p: &ReplayKillPosition) -> bool {
    match (p.killer.is_some(), p.victim.is_some()) {
        (true, true) => report.both += 1,
        (true, false) => report.killer_only += 1,
        (false, true) => report.victim_only += 1,
        (false, false) => {
            report.dropped += 1;
            return false;
        }
    }
    true
}
fn place(
    positions: &[ReplayPlayerPosition],
    identity: &ReplayIdentityState,
    kills: &[ReplayKillReference],
    offset_us: i64,
) -> (ReplayKillPositions, Vec<[u32; 2]>) {
    let mut out = ReplayKillPositions {
        report: ReplayKillPositionReport {
            kills: kills.len(),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut sides = Vec::new();
    if positions.is_empty() || !identity.bridge_established() || kills.is_empty() {
        out.report.dropped = kills.len();
        return (out, sides);
    }
    let mut tracks: BTreeMap<u32, Vec<ReplayPlayerPosition>> = BTreeMap::new();
    for p in positions {
        tracks.entry(p.slot).or_default().push(p.clone());
    }
    for ps in tracks.values_mut() {
        native_sort::sort_by(ps, |a, b| a.timestamp_us.cmp(&b.timestamp_us));
    }
    for k in kills {
        let time = k.time_ms.wrapping_mul(1000).wrapping_add(offset_us) as u64;
        let mut locations = [None, None];
        let mut slots = [0, 0];
        let mut bridged = [false, false];
        for (i, xuid) in [k.killer_xuid, k.victim_xuid].into_iter().enumerate() {
            if xuid == 0 {
                continue;
            }
            let mut found = 0;
            for (&slot, ps) in &tracks {
                if identity.xuid_at(slot, time) != xuid {
                    continue;
                }
                bridged[i] = true;
                let (p, distance) = replay_shots::nearest(ps, time);
                let Some(p) =
                    p.filter(|p| p.has_world && distance <= REPLAY_KILL_POSITION_TOLERANCE_US)
                else {
                    continue;
                };
                found += 1;
                locations[i] = Some([f64::from(p.x), f64::from(p.y), f64::from(p.z)]);
                slots[i] = slot;
            }
            if found != 1 {
                locations[i] = None;
                slots[i] = 0;
            }
        }
        if !bridged[0] || !bridged[1] {
            out.report.no_bridge += 1;
        }
        let p = ReplayKillPosition {
            kill: *k,
            killer: locations[0],
            victim: locations[1],
        };
        if count(&mut out.report, &p) {
            out.positions.push(p);
            sides.push(slots);
        }
    }
    (out, sides)
}
/// Locate each side only when exactly one temporally owned slot has a world
/// sample within 120ms. Preserve input kill order and account for missing sides.
pub fn build_replay_kill_positions(
    positions: &[ReplayPlayerPosition],
    identity: &ReplayIdentityState,
    kills: &[ReplayKillReference],
    offset_us: i64,
) -> ReplayKillPositions {
    place(positions, identity, kills, offset_us).0
}

/// Estimate positions 1.5 seconds before each kill. Both query times must fit
/// the same replication life (5s gap split, with 120ms end tolerance).
/// Returned timestamps remain the original kill timestamps, not opening times.
pub fn build_replay_kill_openings(
    positions: &[ReplayPlayerPosition],
    identity: &ReplayIdentityState,
    kills: &[ReplayKillReference],
    offset_us: i64,
) -> ReplayKillPositions {
    let shifted = shift_replay_kill_references(kills, -REPLAY_OPENING_LEAD_MS);
    let (mut out, sides) = place(positions, identity, &shifted, offset_us);
    let lives = build_identity_life_spans(positions);
    out.report.both = 0;
    out.report.killer_only = 0;
    out.report.victim_only = 0;
    out.positions = out
        .positions
        .into_iter()
        .zip(sides)
        .filter_map(|(mut p, slots)| {
            let opening = p.kill.time_ms.wrapping_mul(1000).wrapping_add(offset_us);
            let fatal = opening.wrapping_add(REPLAY_OPENING_LEAD_MS * 1000);
            for (i, side) in [&mut p.killer, &mut p.victim].into_iter().enumerate() {
                let same = lives.iter().any(|l| {
                    l.slot == slots[i]
                        && [opening, fatal].into_iter().all(|t| {
                            t >= l.from
                                && t <= l.to.wrapping_add(REPLAY_KILL_POSITION_TOLERANCE_US as i64)
                        })
                });
                if side.is_some() && !same {
                    *side = None;
                    out.report.opening_out_of_life += 1;
                }
            }
            p.kill.time_ms = p.kill.time_ms.wrapping_add(REPLAY_OPENING_LEAD_MS);
            count(&mut out.report, &p).then_some(p)
        })
        .collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        positions: Vec<ReplayPlayerPosition>,
        lives: Vec<IdentityLife>,
        indices: BTreeMap<u64, i64>,
        kills: Vec<ReplayKillReference>,
        offset: i64,
        delta: i64,
        shifted: Vec<ReplayKillReference>,
        placed: ReplayKillPositions,
        openings: ReplayKillPositions,
    }
    #[test]
    fn native_kill_positions_and_openings() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/kill-positions-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 512);
        let mut rejected_sides = 0;
        for (i, c) in cases.into_iter().enumerate() {
            let state = ReplayIdentityState::from_lives(c.lives, &c.indices);
            assert_eq!(
                shift_replay_kill_references(&c.kills, c.delta),
                c.shifted,
                "shift {i}"
            );
            let placed = build_replay_kill_positions(&c.positions, &state, &c.kills, c.offset);
            assert_eq!(placed, c.placed, "placed {i}");
            let openings = build_replay_kill_openings(&c.positions, &state, &c.kills, c.offset);
            assert_eq!(openings, c.openings, "openings {i}");
            rejected_sides += openings.report.opening_out_of_life;
            for result in [placed, openings] {
                assert_eq!(
                    result.report.kills,
                    result.report.both
                        + result.report.killer_only
                        + result.report.victim_only
                        + result.report.dropped
                );
                assert_eq!(
                    serde_json::from_slice::<ReplayKillPositions>(
                        &serde_json::to_vec(&result).unwrap()
                    )
                    .unwrap(),
                    result
                );
            }
        }
        assert!(rejected_sides > 0);
    }
}
