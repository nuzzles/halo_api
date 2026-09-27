//! Native replication stays and their refinement using written lifetime boundaries.
use super::{IdentityCreationRecord, IdentityLife, ReplayPlayerPosition};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityDeath {
    #[serde(rename = "XUID")]
    pub xuid: u64,
    pub gamertag: super::ReplayByteString,
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
}
/// This initial split is used for clock calibration, not final published lives.
/// Missing world coordinates still provide a timestamp, as in native indexBySlot.
pub fn build_identity_life_spans(positions: &[ReplayPlayerPosition]) -> Vec<IdentityLife> {
    let mut by_slot = BTreeMap::<u32, Vec<u64>>::new();
    for p in positions {
        by_slot.entry(p.slot).or_default().push(p.timestamp_us);
    }
    let mut out = Vec::new();
    for (slot, mut times) in by_slot {
        times.sort_unstable();
        let mut start = times[0] as i64;
        let mut last = start;
        for t in times.into_iter().skip(1).map(|t| t as i64) {
            if t.wrapping_sub(last) > 5_000_000 {
                out.push(IdentityLife {
                    slot,
                    from: start,
                    to: last,
                    cause: "cut".into(),
                    ..Default::default()
                });
                start = t;
            }
            last = t;
        }
        out.push(IdentityLife {
            slot,
            from: start,
            to: last,
            cause: "film_end".into(),
            ..Default::default()
        });
    }
    out
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityLifetimeBoundaries {
    pub creations: Vec<IdentityCreationRecord>,
    pub rounds_us: Vec<i64>,
    pub deaths_by_player: BTreeMap<u64, Vec<i64>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityLifetimeOutput {
    pub lives: Vec<IdentityLife>,
    /// Native repli_vie_coupee_au_trou_de_replication count.
    pub replication_gap_fallbacks: usize,
}
pub fn identity_deaths_by_player(
    deaths: &[IdentityDeath],
    offset_ms: i64,
) -> BTreeMap<u64, Vec<i64>> {
    let mut out = BTreeMap::<u64, Vec<i64>>::new();
    for d in deaths {
        out.entry(d.xuid)
            .or_default()
            .push(d.time_ms.wrapping_add(offset_ms).wrapping_mul(1000));
    }
    out
}
/// Input order is preserved. Unjustified gaps merge; a written death, creation,
/// round boundary, or explicitly counted no-death fallback keeps the split.
pub fn refine_identity_lives(
    lives: &[IdentityLife],
    boundaries: &IdentityLifetimeBoundaries,
) -> IdentityLifetimeOutput {
    let mut creations = BTreeMap::<u32, Vec<u64>>::new();
    for c in &boundaries.creations {
        creations.entry(c.slot).or_default().push(c.timestamp_us);
    }
    let mut out = IdentityLifetimeOutput {
        lives: Vec::with_capacity(lives.len()),
        replication_gap_fallbacks: 0,
    };
    for life in lives {
        if let Some(previous) = out.lives.last_mut().filter(|p| p.slot == life.slot) {
            let deaths = boundaries
                .deaths_by_player
                .get(&previous.xuid)
                .map_or(&[][..], Vec::as_slice);
            let death = if previous.xuid == 0 {
                previous.cause == "death"
            } else {
                deaths
                    .iter()
                    .any(|&t| t >= previous.to.wrapping_sub(150_000) && t <= life.from)
            };
            let creation = creations.get(&life.slot).is_some_and(|ts| {
                ts.iter()
                    .any(|&t| (t as i64) > previous.to && (t as i64) <= life.from)
            });
            let round = boundaries
                .rounds_us
                .iter()
                .any(|&t| t >= previous.to && t <= life.from);
            if death {
                previous.cause = "death".into();
            } else if creation || round {
                previous.cause = "cut".into();
            } else if deaths.is_empty() {
                previous.cause = "cut".into();
                out.replication_gap_fallbacks += 1;
            } else {
                previous.to = life.to;
                previous.cause = life.cause.clone();
                if previous.xuid == 0 && previous.bid.is_empty() {
                    previous.xuid = life.xuid;
                    previous.bid = life.bid.clone();
                    previous.named_by = life.named_by.clone();
                }
                continue;
            }
        }
        out.lives.push(life.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::{
        IdentityDeathPair, IdentityDeathVerification, mark_identity_deaths, match_identity_deaths,
        name_identity_lives_by_deaths, verify_identity_deaths,
    };
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        pairs: Vec<IdentityDeathPair>,
        verification: IdentityDeathVerification,
        marked: Vec<IdentityLife>,
        named: usize,
        fallback_named: Vec<IdentityLife>,
        positions: Vec<ReplayPlayerPosition>,
        scaffold: Vec<IdentityLife>,
        lives: Vec<IdentityLife>,
        deaths: Vec<IdentityDeath>,
        offset: i64,
        boundaries: IdentityLifetimeBoundaries,
        output: IdentityLifetimeOutput,
    }
    #[test]
    fn native_identity_lifetime_boundaries() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-lifetimes-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            let pairs = match_identity_deaths(&c.lives, &c.deaths, c.offset);
            assert_eq!(pairs, c.pairs, "pairs {i}");
            assert_eq!(
                verify_identity_deaths(&c.lives, &c.deaths, &pairs),
                c.verification,
                "verification {i}"
            );
            let mut marked = c.lives.clone();
            mark_identity_deaths(&mut marked, &pairs);
            assert_eq!(marked, c.marked, "marking {i}");
            assert_eq!(
                name_identity_lives_by_deaths(&mut marked, &c.deaths, &pairs),
                c.named,
                "fallback count {i}"
            );
            assert_eq!(marked, c.fallback_named, "fallback naming {i}");
            assert_eq!(
                build_identity_life_spans(&c.positions),
                c.scaffold,
                "scaffold {i}"
            );
            assert_eq!(
                identity_deaths_by_player(&c.deaths, c.offset),
                c.boundaries.deaths_by_player,
                "death times {i}"
            );
            assert_eq!(
                refine_identity_lives(&c.lives, &c.boundaries),
                c.output,
                "boundaries {i}"
            );
        }
    }
}
