//! Native firing/respawn deductions that extend a supplied identity bridge.
use super::{IdentityDeath, IdentityLife, ReplayPlayerPosition};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityFireReference {
    pub film_index: i64,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityClosureReport {
    pub by_shot: usize,
    pub by_respawn: usize,
    pub contested: usize,
    pub refused: usize,
    pub closed_life: BTreeMap<u32, i64>,
}
impl IdentityClosureReport {
    fn note(&mut self, slot: u32, life: i64) {
        let value = if self.closed_life.get(&slot).is_some_and(|&p| p != life) {
            -1
        } else {
            life
        };
        self.closed_life.insert(slot, value);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityClosureOutput {
    pub owners: BTreeMap<u32, i64>,
    pub report: IdentityClosureReport,
}
pub struct IdentityClosureInput<'a> {
    pub positions: &'a [ReplayPlayerPosition],
    pub lives: &'a [IdentityLife],
    pub deaths: &'a [IdentityDeath],
    pub offset_ms: i64,
    pub indices: &'a BTreeMap<u64, i64>,
    pub fire: &'a [IdentityFireReference],
}
type Tracks = BTreeMap<u32, Vec<u64>>;
/// Apply firing deductions, then respawn deductions. The input bridge is preserved.
pub fn close_identity_bridge(
    owner: &BTreeMap<u32, i64>,
    input: IdentityClosureInput<'_>,
) -> IdentityClosureOutput {
    let mut tracks = Tracks::new();
    for p in input.positions {
        tracks.entry(p.slot).or_default().push(p.timestamp_us);
    }
    for t in tracks.values_mut() {
        t.sort_unstable();
    }
    let mut out = IdentityClosureOutput {
        owners: owner.clone(),
        report: IdentityClosureReport::default(),
    };
    close_shots(&tracks, &mut out, input.lives, input.fire);
    close_respawns(
        &tracks,
        &mut out,
        input.lives,
        input.deaths,
        input.offset_ms,
        input.indices,
    );
    out
}
fn free_lives(owners: &BTreeMap<u32, i64>, lives: &[IdentityLife]) -> Vec<usize> {
    lives
        .iter()
        .enumerate()
        .filter(|(_, l)| l.xuid == 0 && !owners.contains_key(&l.slot))
        .map(|(i, _)| i)
        .collect()
}
fn close_shots(
    tracks: &Tracks,
    out: &mut IdentityClosureOutput,
    lives: &[IdentityLife],
    fire: &[IdentityFireReference],
) {
    let free = free_lives(&out.owners, lives);
    if free.is_empty() {
        return;
    }
    let mut claims = BTreeMap::<u32, BTreeSet<i64>>::new();
    let mut blocked = BTreeSet::new();
    let mut designated = BTreeMap::<u32, i64>::new();
    for shot in fire {
        let attached = out
            .owners
            .iter()
            .filter(|(slot, pi)| {
                **pi == shot.film_index
                    && tracks.get(slot).is_some_and(|ts| {
                        ts.iter().any(|&t| t.abs_diff(shot.timestamp_us) <= 120_000)
                    })
            })
            .count();
        if attached == 1 {
            continue;
        }
        let t = shot.timestamp_us as i64;
        let candidates: Vec<_> = free
            .iter()
            .copied()
            .filter(|&i| lives[i].from <= t && t <= lives[i].to)
            .collect();
        if candidates.len() != 1 {
            for i in candidates {
                blocked.insert(lives[i].slot);
            }
            continue;
        }
        let i = candidates[0];
        let slot = lives[i].slot;
        if claims.contains_key(&slot) {
            if designated[&slot] != i as i64 {
                designated.insert(slot, -1);
            }
        } else {
            designated.insert(slot, i as i64);
        }
        claims.entry(slot).or_default().insert(shot.film_index);
    }
    let mut player_claims = BTreeMap::<i64, usize>::new();
    for c in claims.values().filter(|c| c.len() == 1) {
        *player_claims.entry(*c.first().unwrap()).or_default() += 1;
    }
    for (&slot, claim) in &claims {
        if claim.len() != 1 {
            out.report.contested += 1;
            continue;
        }
        let pi = *claim.first().unwrap();
        if player_claims[&pi] > 1 {
            out.report.contested += 1;
            continue;
        }
        let life = designated[&slot];
        let from = usize::try_from(life)
            .ok()
            .and_then(|i| lives.get(i))
            .filter(|l| l.slot == slot)
            .map(|l| l.from as u64)
            .unwrap_or_else(|| {
                tracks
                    .get(&slot)
                    .and_then(|v| v.first())
                    .copied()
                    .unwrap_or(0)
            });
        let mut anchored = false;
        let mut possible = tracks.get(&slot).is_some_and(|t| !t.is_empty());
        for (&s, &p) in &out.owners {
            if p != pi || s == slot {
                continue;
            }
            if let Some(&last) = tracks.get(&s).and_then(|t| t.last()) {
                if last >= from {
                    possible = false;
                    break;
                }
                anchored = true;
            }
        }
        if !possible || !anchored {
            out.report.refused += 1;
            continue;
        }
        out.owners.insert(slot, pi);
        out.report.note(slot, life);
        out.report.by_shot += 1;
    }
    out.report.contested += blocked.iter().filter(|s| !claims.contains_key(s)).count();
}
pub fn identity_respawn_window(
    lives: &[IdentityLife],
    deaths: &[IdentityDeath],
    offset: i64,
) -> (i64, i64) {
    let mut deltas = Vec::new();
    for l in lives.iter().filter(|l| l.xuid != 0) {
        let mut best = -1;
        for d in deaths.iter().filter(|d| d.xuid == l.xuid) {
            let t = d.time_ms.wrapping_add(offset);
            if t < l.from / 1000 && (best < 0 || t > best) {
                best = t;
            }
        }
        if best >= 0 {
            deltas.push((l.from / 1000).wrapping_sub(best));
        }
    }
    if deltas.is_empty() {
        return (0, 0);
    }
    deltas.sort_unstable();
    let median = deltas[deltas.len() / 2].wrapping_mul(1000);
    (median.wrapping_sub(750_000), median.wrapping_add(750_000))
}
fn close_respawns(
    tracks: &Tracks,
    out: &mut IdentityClosureOutput,
    lives: &[IdentityLife],
    deaths: &[IdentityDeath],
    offset: i64,
    indices: &BTreeMap<u64, i64>,
) {
    let free = free_lives(&out.owners, lives);
    if free.is_empty() || deaths.is_empty() {
        return;
    }
    let (lo, hi) = identity_respawn_window(lives, deaths, offset);
    if lo == 0 && hi == 0 {
        return;
    }
    let mut claims = BTreeMap::<u64, Vec<usize>>::new();
    for i in free {
        let candidates: BTreeSet<_> = deaths
            .iter()
            .filter(|d| {
                let delta = lives[i]
                    .from
                    .wrapping_sub(d.time_ms.wrapping_add(offset).wrapping_mul(1000));
                delta >= lo && delta <= hi
            })
            .map(|d| d.xuid)
            .collect();
        if candidates.len() > 1 {
            out.report.contested += 1;
        } else if let Some(&x) = candidates.first() {
            claims.entry(x).or_default().push(i);
        }
    }
    for (x, indices_life) in claims {
        if indices_life.len() != 1 {
            out.report.contested += indices_life.len();
            continue;
        }
        let Some(&pi) = indices.get(&x) else {
            out.report.refused += 1;
            continue;
        };
        let i = indices_life[0];
        let life = &lives[i];
        let slot = life.slot;
        let overlap = tracks.get(&slot).is_some_and(|t| !t.is_empty())
            && out.owners.iter().any(|(&s, &p)| {
                if p != pi || s == slot {
                    return false;
                }
                tracks.get(&s).is_some_and(|t| {
                    t.first().zip(t.last()).is_some_and(|(&from, &to)| {
                        from <= life.to as u64 && life.from as u64 <= to
                    })
                })
            });
        if overlap {
            out.report.refused += 1;
            continue;
        }
        out.owners.insert(slot, pi);
        out.report.note(slot, i as i64);
        out.report.by_respawn += 1;
    }
}

/// Name only the life designated by a closure, preserving existing human names
/// and lifetime causes. Indices must be injective, as in the native pipeline.
pub fn name_identity_closed_lives(
    lives: &mut [IdentityLife],
    owners: &BTreeMap<u32, i64>,
    closed: &BTreeMap<u32, i64>,
    indices: &BTreeMap<u64, i64>,
) {
    let inverse: BTreeMap<_, _> = indices.iter().map(|(&x, &i)| (i, x)).collect();
    for (slot, &i) in closed {
        let Some(life) = usize::try_from(i).ok().and_then(|i| lives.get_mut(i)) else {
            continue;
        };
        if life.xuid != 0 {
            continue;
        }
        if let Some(&x) = owners.get(slot).and_then(|pi| inverse.get(pi)) {
            life.xuid = x;
            life.named_by = "closure".into();
        }
    }
}
/// Extend slot identities without replacing an existing entry. The participant
/// index table must be injective before this operation.
pub fn extend_identity_slot_xuids(
    slot_xuids: &BTreeMap<u32, u64>,
    owners: &BTreeMap<u32, i64>,
    indices: &BTreeMap<u64, i64>,
) -> BTreeMap<u32, u64> {
    let inverse: BTreeMap<_, _> = indices.iter().map(|(&x, &i)| (i, x)).collect();
    let mut out = slot_xuids.clone();
    for (&slot, pi) in owners {
        if let Some(&x) = inverse.get(pi) {
            out.entry(slot).or_insert(x);
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
        lives: Vec<IdentityLife>,
        positions: Vec<ReplayPlayerPosition>,
        owners: BTreeMap<u32, i64>,
        indices: BTreeMap<u64, i64>,
        deaths: Vec<IdentityDeath>,
        fire: Vec<IdentityFireReference>,
        offset: i64,
        window: (i64, i64),
        output: IdentityClosureOutput,
        named: Vec<IdentityLife>,
        slot_xuids: BTreeMap<u32, u64>,
        extended: BTreeMap<u32, u64>,
    }
    #[test]
    fn closure_naming_preserves_native_guards_and_causes() {
        let mut lives = vec![
            IdentityLife {
                slot: 1,
                xuid: 101,
                cause: "death".into(),
                named_by: "biped_creation".into(),
                ..Default::default()
            },
            IdentityLife {
                slot: 2,
                bid: "bid(7.0)".into(),
                cause: "film_end".into(),
                ..Default::default()
            },
            IdentityLife {
                slot: 3,
                cause: "cut".into(),
                ..Default::default()
            },
        ];
        let original = lives.clone();
        let indices = BTreeMap::from([(100, 0)]);
        let owners = BTreeMap::from([(1, 0), (2, 0), (3, 9), (4, 0), (5, 0)]);
        // Native closure naming guards XUID, not the bot label. It preserves
        // that label, unlike the later guarded deduction mutation API.
        let closed = BTreeMap::from([(1, 0), (2, 1), (3, 2), (4, -1), (5, i64::MAX), (6, 2)]);
        name_identity_closed_lives(&mut lives, &owners, &closed, &indices);
        assert_eq!(lives[0], original[0]);
        assert_eq!(lives[1].xuid, 100);
        assert_eq!(lives[1].bid, "bid(7.0)");
        assert_eq!(lives[1].cause, "film_end");
        assert_eq!(lives[1].named_by, "closure");
        assert_eq!(lives[2], original[2]);
        let extended =
            extend_identity_slot_xuids(&BTreeMap::from([(1, 101), (8, 0)]), &owners, &indices);
        assert_eq!(
            extended,
            BTreeMap::from([(1, 101), (2, 100), (4, 100), (5, 100), (8, 0)])
        );
    }
    #[test]
    fn native_identity_closure_deductions() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-closures-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            let mut named = c.lives.clone();
            name_identity_closed_lives(
                &mut named,
                &c.output.owners,
                &c.output.report.closed_life,
                &c.indices,
            );
            assert_eq!(named, c.named, "named {i}");
            assert_eq!(
                extend_identity_slot_xuids(&c.slot_xuids, &c.output.owners, &c.indices),
                c.extended,
                "extended {i}"
            );
            assert_eq!(
                identity_respawn_window(&c.lives, &c.deaths, c.offset),
                c.window,
                "window {i}"
            );
            assert_eq!(
                close_identity_bridge(
                    &c.owners,
                    IdentityClosureInput {
                        positions: &c.positions,
                        lives: &c.lives,
                        deaths: &c.deaths,
                        offset_ms: c.offset,
                        indices: &c.indices,
                        fire: &c.fire
                    }
                ),
                c.output,
                "closures {i}"
            );
        }
    }
}
