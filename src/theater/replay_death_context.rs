//! Native death-context analysis. These are derived measurements using external
//! match deaths/teams, not directly recorded actions or the deferred replay model.
use super::{IdentityRegistryOutput, ReplayPlayerPosition};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DEATH_CONTEXT_VISIBILITY_MS: i64 = 1000;
pub const DEATH_CONTEXT_DISTANCE_SCALE: f64 = 100.0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayJournalDeath {
    pub victim_xuid: u64,
    pub time_ms: i64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayDeathContext {
    pub victim_xuid: u64,
    pub time_ms: i64,
    /// Horizontal world distance; unavailable is distinct from zero.
    pub nearest_teammate_m: Option<f64>,
    pub visible: usize,
    pub waiting: usize,
    pub out_of_sight: usize,
    pub total: usize,
}
#[derive(Clone, Copy)]
struct Point {
    time_ms: i64,
    x: f64,
    y: f64,
}
fn visible(points: &[Point], time_ms: i64) -> Option<Point> {
    let i = points.partition_point(|p| p.time_ms <= time_ms);
    let p = *points.get(i.checked_sub(1)?)?;
    (time_ms.wrapping_sub(p.time_ms) <= DEATH_CONTEXT_VISIBILITY_MS).then_some(p)
}
/// Preserve journal order, omitting victims without a recent world position or
/// an external team. Index disagreements or an absent identity bridge suppress
/// the entire result. Recycled slots remain valid: only covering named lives
/// attribute positions, without the registry's flattened-bridge fallback.
pub fn build_replay_death_contexts(
    positions: &[ReplayPlayerPosition],
    registry: &IdentityRegistryOutput,
    journal: &[ReplayJournalDeath],
    teams: &BTreeMap<u64, i64>,
) -> Vec<ReplayDeathContext> {
    let owners = &registry.owners;
    if owners.index_disagreements != 0 || !owners.state.bridge_established() {
        return Vec::new();
    }
    let offset = owners.clock.offset_ms;
    let lives = owners.state.lives();
    let mut points: BTreeMap<u64, Vec<Point>> = BTreeMap::new();
    let mut spans: BTreeMap<u64, Vec<(i64, i64)>> = BTreeMap::new();
    let mut deaths: BTreeMap<u64, Vec<i64>> = BTreeMap::new();
    for l in lives.iter().filter(|l| l.xuid != 0) {
        spans.entry(l.xuid).or_default().push((
            (l.from / 1000).wrapping_sub(offset),
            (l.to / 1000).wrapping_sub(offset),
        ));
    }
    for p in positions.iter().filter(|p| p.has_world) {
        let t = p.timestamp_us as i64;
        let Some(l) = lives
            .iter()
            .find(|l| l.xuid != 0 && l.slot == p.slot && t >= l.from && t <= l.to)
        else {
            continue;
        };
        points.entry(l.xuid).or_default().push(Point {
            time_ms: (t / 1000).wrapping_sub(offset),
            x: f64::from(p.x),
            y: f64::from(p.y),
        });
    }
    for ps in points.values_mut() {
        super::native_sort::sort_by(ps, |a, b| a.time_ms.cmp(&b.time_ms));
    }
    for d in journal {
        deaths.entry(d.victim_xuid).or_default().push(d.time_ms);
    }
    for ds in deaths.values_mut() {
        ds.sort_unstable();
    }
    let mut out = Vec::new();
    for d in journal {
        let victim_points = points
            .get(&d.victim_xuid)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let Some(victim) = visible(victim_points, d.time_ms) else {
            continue;
        };
        let Some(team) = teams.get(&d.victim_xuid) else {
            continue;
        };
        let mut c = ReplayDeathContext {
            victim_xuid: d.victim_xuid,
            time_ms: d.time_ms,
            nearest_teammate_m: None,
            visible: 0,
            waiting: 0,
            out_of_sight: 0,
            total: 0,
        };
        let mut nearest = f64::INFINITY;
        for (&xuid, other_team) in teams {
            if xuid == d.victim_xuid || team != other_team {
                continue;
            }
            c.total += 1;
            let ps = points.get(&xuid).map(Vec::as_slice).unwrap_or_default();
            let alive = spans
                .get(&xuid)
                .is_some_and(|ss| ss.iter().any(|&(a, b)| d.time_ms >= a && d.time_ms <= b));
            if let Some(p) = visible(ps, d.time_ms).filter(|_| alive) {
                c.visible += 1;
                let distance = (p.x - victim.x).hypot(p.y - victim.y);
                if distance < nearest {
                    nearest = distance;
                }
                continue;
            }
            let ds = deaths.get(&xuid).map(Vec::as_slice).unwrap_or_default();
            let i = ds.partition_point(|&t| t < d.time_ms);
            let waiting = i.checked_sub(1).is_some_and(|i| {
                let j = ps.partition_point(|p| p.time_ms <= ds[i]);
                !ps.get(j).is_some_and(|p| p.time_ms <= d.time_ms)
            });
            if waiting {
                c.waiting += 1;
            } else {
                c.out_of_sight += 1;
            }
        }
        if nearest != f64::INFINITY {
            c.nearest_teammate_m = Some(
                (nearest * DEATH_CONTEXT_DISTANCE_SCALE).round() / DEATH_CONTEXT_DISTANCE_SCALE,
            );
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::{IdentityLife, IdentityOwnerOutput, ReplayIdentityState};
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        positions: Vec<ReplayPlayerPosition>,
        lives: Vec<IdentityLife>,
        bridge: bool,
        offset: i64,
        disagreements: i64,
        teams: BTreeMap<u64, i64>,
        journal: Vec<ReplayJournalDeath>,
        expected: Vec<ReplayDeathContext>,
    }
    #[test]
    fn native_death_contexts() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/death-context-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 512);
        let mut states = [0; 3];
        for (i, c) in cases.into_iter().enumerate() {
            let indices = if c.bridge {
                (1..=5).map(|x| (x, x as i64)).collect()
            } else {
                BTreeMap::new()
            };
            let mut owners = IdentityOwnerOutput {
                state: ReplayIdentityState::from_lives(c.lives, &indices),
                index_disagreements: c.disagreements,
                ..Default::default()
            };
            owners.clock.offset_ms = c.offset;
            let registry = IdentityRegistryOutput {
                owners,
                tables: crate::theater::compose_identity_tables(
                    &Default::default(),
                    &Default::default(),
                ),
                scoreboard: Default::default(),
            };
            let out = build_replay_death_contexts(&c.positions, &registry, &c.journal, &c.teams);
            assert_eq!(out, c.expected, "native death context {i}");
            assert_eq!(
                serde_json::from_slice::<Vec<ReplayDeathContext>>(
                    &serde_json::to_vec(&out).unwrap()
                )
                .unwrap(),
                out
            );
            for context in out {
                states[0] += context.visible;
                states[1] += context.waiting;
                states[2] += context.out_of_sight;
                assert_eq!(
                    context.total,
                    context.visible + context.waiting + context.out_of_sight
                );
            }
        }
        assert!(states.into_iter().all(|n| n > 0));
    }
}
