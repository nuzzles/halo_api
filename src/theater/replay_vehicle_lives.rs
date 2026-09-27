//! Vehicle census windows and written-death attribution from the native replay builder.
use super::{
    EquipmentLifeKey, FactsWorldKeyframes, ObjectDeath, WorldObjectKeyframes, native_sort,
};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVehicleLife {
    pub slot: u32,
    pub generation: u32,
    pub first_us: u64,
    pub last_us: u64,
    pub gone_by_us: u64,
    pub lo_us: u64,
    pub hi_us: u64,
    pub census: usize,
    pub death_us: u64,
    pub death_tail_desync: bool,
}
impl ReplayVehicleLife {
    pub fn key(&self) -> EquipmentLifeKey {
        EquipmentLifeKey {
            slot: self.slot,
            generation: self.generation,
        }
    }
    /// Written death wins over final-census presence. A known death at frame zero
    /// remains explicit; timestamp zero is the native absence sentinel.
    pub fn end(&self, origin: u64, step: NonZeroU64, frames: i64) -> (&'static str, Option<i64>) {
        if self.death_us > 0 {
            (
                "destroyed",
                Some(super::replay_vehicle_shots::frame(
                    self.death_us,
                    origin,
                    step,
                    frames,
                )),
            )
        } else if self.gone_by_us == 0 {
            ("film_end", None)
        } else {
            ("unknown", None)
        }
    }
    /// Display birth, last presence and first absence. Contradictory movement
    /// after a written death remains visible instead of being silently discarded.
    pub fn bounds(
        &self,
        spawn_us: u64,
        last_seen_us: u64,
        origin: u64,
        step: NonZeroU64,
        frames: i64,
    ) -> [i64; 3] {
        let project = |t| super::replay_vehicle_shots::frame(t, origin, step, frames);
        let born = if spawn_us > 0 && spawn_us < self.first_us {
            spawn_us
        } else {
            self.first_us
        };
        let t0 = project(born);
        let t1 = project(self.last_us.max(last_seen_us)).max(t0);
        let mut t1_max = if self.gone_by_us > 0 {
            project(self.gone_by_us)
        } else {
            frames.wrapping_sub(1)
        };
        if self.death_us > 0 {
            t1_max = t1_max.min(project(self.death_us));
        }
        [t0, t1, t1_max.max(t1)]
    }
}
/// The fields used by vehicle attribution from an already-read object death.
/// The caller supplies vehicle deaths; this stage does not filter archetypes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVehicleDeathEvidence {
    pub slot: u32,
    pub generation: u32,
    pub timestamp_us: u64,
    pub tail_desync: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVehicleDeathTally {
    pub read: usize,
    pub matched: usize,
    pub unmatched: usize,
    pub tail_desync: usize,
    pub closed: usize,
    pub no_slot: usize,
    pub no_gen: usize,
    pub window: usize,
    pub window_before: usize,
    pub window_after: usize,
    pub window_max_gap_ms: i64,
}
/// Bound vehicle lives using census evidence, then assign the earliest written
/// death. Census arrays retain their supplied order, as in the native builder.
/// An open final life has an unlimited upper window, except for slot reuse.
pub fn build_replay_vehicle_lives(
    census: &WorldObjectKeyframes,
    deaths: &[ReplayVehicleDeathEvidence],
) -> (Vec<ReplayVehicleLife>, ReplayVehicleDeathTally) {
    build_vehicle_life_values(
        &census.times_us,
        census
            .seen_us
            .iter()
            .map(|s| (s.slot, s.generation, s.times_us.as_slice())),
        deaths.iter().cloned(),
    )
}
/// Consume the cache's already-selected vehicle deaths; do not re-filter their
/// archetype or dead-state payload at this assembly boundary.
pub fn build_facts_replay_vehicle_lives(
    census: &FactsWorldKeyframes,
    deaths: &[ObjectDeath],
) -> (Vec<ReplayVehicleLife>, ReplayVehicleDeathTally) {
    build_vehicle_life_values(
        &census.times_us,
        census
            .seen_us
            .iter()
            .map(|(&(slot, generation), times)| (slot, generation, times.as_slice())),
        deaths.iter().map(facts_vehicle_death),
    )
}
fn facts_vehicle_death(d: &ObjectDeath) -> ReplayVehicleDeathEvidence {
    ReplayVehicleDeathEvidence {
        slot: d.slot,
        generation: d.r#gen,
        timestamp_us: d.timestamp_us,
        tail_desync: d.tail_desync,
    }
}
fn build_vehicle_life_values<'a>(
    times: &[u64],
    sightings: impl Iterator<Item = (u32, u32, &'a [u64])>,
    deaths: impl ExactSizeIterator<Item = ReplayVehicleDeathEvidence>,
) -> (Vec<ReplayVehicleLife>, ReplayVehicleDeathTally) {
    let mut lives: Vec<_> = sightings
        .filter_map(|(slot, generation, seen)| {
            let first_us = *seen.first()?;
            let last_us = *seen.last()?;
            let (mut lo, mut hi) = (0, times.len());
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if times[mid] > last_us {
                    hi = mid;
                } else {
                    lo = mid + 1;
                }
            }
            Some(ReplayVehicleLife {
                slot,
                generation,
                first_us,
                last_us,
                gone_by_us: times.get(lo).copied().unwrap_or(0),
                census: seen.len(),
                ..Default::default()
            })
        })
        .collect();
    native_sort::sort_by(&mut lives, |a, b| {
        a.slot.cmp(&b.slot).then(a.first_us.cmp(&b.first_us))
    });
    for i in 0..lives.len() {
        lives[i].lo_us = lives[i].first_us.saturating_sub(20_000_000);
        lives[i].hi_us = if lives[i].gone_by_us == 0 {
            u64::MAX
        } else {
            lives[i].gone_by_us
        };
        if i > 0 && lives[i - 1].slot == lives[i].slot && lives[i - 1].hi_us > lives[i].lo_us {
            lives[i].lo_us = lives[i - 1].hi_us;
        }
        if i + 1 < lives.len()
            && lives[i + 1].slot == lives[i].slot
            && lives[i].hi_us > lives[i + 1].first_us
        {
            lives[i].hi_us = lives[i + 1].first_us;
        }
    }
    let tally = assign_vehicle_death_values(&mut lives, deaths);
    (lives, tally)
}
/// Assign deaths to existing windows without changing those windows. Repeated
/// observations are counted separately from closed lives, including timestamp zero.
pub fn assign_replay_vehicle_deaths(
    lives: &mut [ReplayVehicleLife],
    deaths: &[ReplayVehicleDeathEvidence],
) -> ReplayVehicleDeathTally {
    assign_vehicle_death_values(lives, deaths.iter().cloned())
}
pub fn assign_facts_replay_vehicle_deaths(
    lives: &mut [ReplayVehicleLife],
    deaths: &[ObjectDeath],
) -> ReplayVehicleDeathTally {
    assign_vehicle_death_values(lives, deaths.iter().map(facts_vehicle_death))
}
fn assign_vehicle_death_values(
    lives: &mut [ReplayVehicleLife],
    deaths: impl ExactSizeIterator<Item = ReplayVehicleDeathEvidence>,
) -> ReplayVehicleDeathTally {
    let mut tally = ReplayVehicleDeathTally {
        read: deaths.len(),
        ..Default::default()
    };
    if lives.is_empty() || deaths.len() == 0 {
        tally.unmatched = deaths.len();
        return tally;
    }
    for d in deaths {
        if let Some(l) = lives.iter_mut().find(|l| {
            l.slot == d.slot
                && l.generation == d.generation
                && d.timestamp_us >= l.lo_us
                && d.timestamp_us <= l.hi_us
        }) {
            tally.matched += 1;
            tally.tail_desync += usize::from(d.tail_desync);
            if l.death_us == 0 {
                tally.closed += 1;
            }
            if l.death_us == 0 || d.timestamp_us < l.death_us {
                l.death_us = d.timestamp_us;
                l.death_tail_desync = d.tail_desync;
            }
            continue;
        }
        tally.unmatched += 1;
        if !lives.iter().any(|l| l.slot == d.slot) {
            tally.no_slot += 1;
            continue;
        }
        if !lives
            .iter()
            .any(|l| l.slot == d.slot && l.generation == d.generation)
        {
            tally.no_gen += 1;
            continue;
        }
        tally.window += 1;
        let mut best = -1_i64;
        let mut after = false;
        for l in lives
            .iter()
            .filter(|l| l.slot == d.slot && l.generation == d.generation)
        {
            let (gap, side) = if d.timestamp_us < l.lo_us {
                ((l.lo_us - d.timestamp_us) as i64, false)
            } else {
                (d.timestamp_us.wrapping_sub(l.hi_us) as i64, true)
            };
            if best < 0 || gap < best {
                best = gap;
                after = side;
            }
        }
        if best >= 0 {
            if after {
                tally.window_after += 1;
            } else {
                tally.window_before += 1;
            }
            tally.window_max_gap_ms = tally.window_max_gap_ms.max(best / 1000);
        }
    }
    tally.log();
    tally
}

impl ReplayVehicleDeathTally {
    /// Native attribution diagnostics, emitted only after the assignment loop.
    /// Empty life/death inputs return before this call in the native reducer.
    pub fn log(&self) {
        if self.read == 0 {
            return;
        }
        macro_rules! emit {
            ($level:expr) => {
                tracing::event!(
                    $level,
                    lues = self.read,
                    appariees = self.matched,
                    viesFermees = self.closed,
                    nonAppariees = self.unmatched,
                    queueRompue = self.tail_desync,
                    perdues_slotAbsent = self.no_slot,
                    perdues_generationAbsente = self.no_gen,
                    perdues_horsFenetre = self.window,
                    horsFenetre_avant = self.window_before,
                    horsFenetre_apres = self.window_after,
                    horsFenetre_ecartMaxMS = self.window_max_gap_ms,
                    "rejeu : attribution des morts de vehicule"
                );
            };
        }
        if self.unmatched > 0 {
            emit!(tracing::Level::WARN);
        } else {
            emit!(tracing::Level::INFO);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Probe {
        life: usize,
        origin: u64,
        step: u64,
        frames: i64,
        spawn: u64,
        last: u64,
        bounds: [i64; 3],
        end: String,
        t_end: Option<i64>,
    }
    #[derive(Deserialize)]
    struct Case {
        census: WorldObjectKeyframes,
        deaths: Vec<ReplayVehicleDeathEvidence>,
        lives: Vec<ReplayVehicleLife>,
        tally: ReplayVehicleDeathTally,
        probes: Vec<Probe>,
    }
    #[test]
    fn native_vehicle_lives_deaths_and_bounds() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-vehicle-lives-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let (lives, tally) = build_replay_vehicle_lives(&c.census, &c.deaths);
            assert_eq!(lives, c.lives, "lives {i}");
            assert_eq!(tally, c.tally, "tally {i}");
            for p in c.probes {
                let step = NonZeroU64::new(p.step).unwrap();
                assert_eq!(
                    lives[p.life].bounds(p.spawn, p.last, p.origin, step, p.frames),
                    p.bounds,
                    "bounds {i}/{}",
                    p.life
                );
                assert_eq!(
                    lives[p.life].end(p.origin, step, p.frames),
                    (p.end.as_str(), p.t_end),
                    "end {i}/{}",
                    p.life
                );
            }
        }
    }
}
