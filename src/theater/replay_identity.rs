//! Native identity bridge and deductions over decoded, bounded player lives.
//! Building these lives from creation/death/scoreboard evidence remains a separate stage.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityLife {
    pub slot: u32,
    pub from: i64,
    pub to: i64,
    pub xuid: u64,
    pub bid: String,
    pub cause: String,
    pub named_by: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayIdentityState {
    lives: Vec<IdentityLife>,
    owner: BTreeMap<u32, i64>,
    slot_xuid: BTreeMap<u32, u64>,
    ambiguous: BTreeSet<u32>,
    deduced_lives: BTreeSet<usize>,
    pub eliminated: usize,
    pub eliminated_slot: u32,
    pub eliminated_xuid: u64,
    pub excluded: usize,
    pub excluded_contradictions: usize,
}
impl ReplayIdentityState {
    /// Preserve input life order: the first known index owns the slot bridge.
    /// A conflicting index marks ambiguity without replacing that first entry.
    pub fn from_lives(lives: Vec<IdentityLife>, indices: &BTreeMap<u64, i64>) -> Self {
        let mut out = Self {
            lives,
            ..Default::default()
        };
        for l in &out.lives {
            if l.xuid == 0 {
                continue;
            }
            let Some(&index) = indices.get(&l.xuid) else {
                continue;
            };
            if out.owner.get(&l.slot).is_some_and(|&prev| prev != index) {
                out.ambiguous.insert(l.slot);
                continue;
            }
            out.owner.insert(l.slot, index);
            out.slot_xuid.insert(l.slot, l.xuid);
        }
        out
    }
    pub(crate) fn install_closure_bridge(
        &mut self,
        lives: Vec<IdentityLife>,
        owners: BTreeMap<u32, i64>,
        indices: &BTreeMap<u64, i64>,
    ) {
        self.slot_xuid = super::extend_identity_slot_xuids(&self.slot_xuid, &owners, indices);
        self.owner = owners;
        self.lives = lives;
    }
    pub fn lives(&self) -> &[IdentityLife] {
        &self.lives
    }
    pub fn indices_by_slot(&self) -> &BTreeMap<u32, i64> {
        &self.owner
    }
    pub fn ambiguous_slots(&self) -> &BTreeSet<u32> {
        &self.ambiguous
    }
    pub fn bridge_established(&self) -> bool {
        !self.slot_xuid.is_empty()
    }
    pub fn life_is_deduced(&self, i: usize) -> bool {
        self.deduced_lives.contains(&i)
    }
    pub fn naming_bridge(&self) -> BTreeMap<u32, u64> {
        self.slot_xuid
            .iter()
            .filter(|(slot, _)| !self.ambiguous.contains(slot))
            .map(|(&slot, &xuid)| (slot, xuid))
            .collect()
    }
    pub fn xuid_at(&self, slot: u32, timestamp_us: u64) -> u64 {
        let t = timestamp_us as i64;
        for l in &self.lives {
            if l.slot == slot && l.xuid != 0 && t >= l.from && t <= l.to {
                return l.xuid;
            }
        }
        if self.ambiguous.contains(&slot) {
            0
        } else {
            self.slot_xuid.get(&slot).copied().unwrap_or(0)
        }
    }
    /// Apply a deduction without overwriting an existing human/bot or changing cause.
    pub fn assign_life(&mut self, i: i64, xuid: u64, index: Option<i64>, provenance: &str) -> bool {
        if xuid == 0 || i < 0 {
            return false;
        }
        let Ok(i) = usize::try_from(i) else {
            return false;
        };
        let Some(l) = self.lives.get_mut(i) else {
            return false;
        };
        if l.xuid != 0 || !l.bid.is_empty() {
            return false;
        }
        l.xuid = xuid;
        l.named_by = provenance.into();
        self.deduced_lives.insert(i);
        if self.slot_xuid.get(&l.slot).is_some_and(|&old| old != xuid) {
            self.ambiguous.insert(l.slot);
            return true;
        }
        self.slot_xuid.insert(l.slot, xuid);
        if let Some(pi) = index {
            self.owner.entry(l.slot).or_insert(pi);
        }
        true
    }
    pub fn assign_bot(&mut self, i: i64, bid: &str) -> bool {
        if i < 0 || bid.is_empty() {
            return false;
        }
        let Ok(i) = usize::try_from(i) else {
            return false;
        };
        let Some(l) = self.lives.get_mut(i) else {
            return false;
        };
        if l.xuid != 0 || !l.bid.is_empty() {
            return false;
        }
        l.bid = bid.into();
        l.named_by = "tableau_api".into();
        true
    }
    pub fn resolve_roster_elimination(
        &mut self,
        roster: &[u64],
        indices: &BTreeMap<u64, i64>,
        deaths_named: usize,
    ) {
        if deaths_named == 0 || self.lives.is_empty() {
            return;
        }
        let named: BTreeSet<_> = self
            .lives
            .iter()
            .filter(|l| l.xuid != 0)
            .map(|l| l.xuid)
            .collect();
        let free: Vec<_> = identity_roster_candidates(roster, indices)
            .into_iter()
            .filter(|x| !named.contains(x))
            .collect();
        let named_slots: BTreeSet<_> = self
            .lives
            .iter()
            .filter(|l| l.xuid != 0)
            .map(|l| l.slot)
            .collect();
        let unnamed: BTreeSet<_> = self
            .lives
            .iter()
            .map(|l| l.slot)
            .filter(|s| !named_slots.contains(s))
            .collect();
        if free.len() != 1 || unnamed.len() != 1 {
            return;
        }
        let slot = *unnamed.first().unwrap();
        let xuid = free[0];
        let mut n = 0;
        for i in 0..self.lives.len() {
            if self.lives[i].slot == slot
                && self.assign_life(i as i64, xuid, indices.get(&xuid).copied(), "elimination")
            {
                n += 1;
            }
        }
        self.eliminated = n;
        if n != 0 {
            self.eliminated_slot = slot;
            self.eliminated_xuid = xuid;
        }
    }
    pub fn resolve_temporal_exclusion(
        &mut self,
        roster: &[u64],
        indices: &BTreeMap<u64, i64>,
        has_bots: bool,
    ) {
        if roster.is_empty() || has_bots || self.lives.is_empty() {
            return;
        }
        let candidates = identity_roster_candidates(roster, indices);
        if candidates.is_empty() || identity_max_occupancy(&self.lives) > candidates.len() {
            return;
        }
        for _ in 0..16 {
            let forced: Vec<_> = self
                .lives
                .iter()
                .enumerate()
                .filter(|(_, l)| l.xuid == 0)
                .filter_map(|(i, _)| {
                    let free = self.free_candidates(i, &candidates);
                    (free.len() == 1).then(|| (i, free[0]))
                })
                .collect();
            let mut placed = 0;
            for &(i, xuid) in &forced {
                if forced
                    .iter()
                    .any(|&(j, x)| j != i && x == xuid && overlap(&self.lives[i], &self.lives[j]))
                {
                    continue;
                }
                if self.assign_life(
                    i as i64,
                    xuid,
                    indices.get(&xuid).copied(),
                    "exclusion_temporelle",
                ) {
                    placed += 1;
                    self.excluded += 1;
                }
            }
            if placed == 0 {
                break;
            }
        }
        self.excluded_contradictions = self
            .lives
            .iter()
            .enumerate()
            .filter(|(i, l)| l.xuid == 0 && self.free_candidates(*i, &candidates).is_empty())
            .count();
    }
    fn free_candidates(&self, i: usize, roster: &[u64]) -> Vec<u64> {
        let occupied: BTreeSet<_> = self
            .lives
            .iter()
            .enumerate()
            .filter(|(j, l)| *j != i && l.xuid != 0 && overlap(&self.lives[i], l))
            .map(|(_, l)| l.xuid)
            .collect();
        roster
            .iter()
            .copied()
            .filter(|x| !occupied.contains(x))
            .collect()
    }
}
fn overlap(a: &IdentityLife, b: &IdentityLife) -> bool {
    a.from <= b.to && b.from <= a.to
}
pub fn identity_roster_candidates(roster: &[u64], indices: &BTreeMap<u64, i64>) -> Vec<u64> {
    roster
        .iter()
        .copied()
        .chain(indices.keys().copied())
        .filter(|&x| x != 0)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
pub fn identity_max_occupancy(lives: &[IdentityLife]) -> usize {
    let mut starts: Vec<_> = lives.iter().map(|l| l.from).collect();
    let mut ends: Vec<_> = lives.iter().map(|l| l.to.wrapping_add(1)).collect();
    starts.sort_unstable();
    ends.sort_unstable();
    let (mut current, mut peak, mut j) = (0_i64, 0_i64, 0);
    for start in starts {
        while j < ends.len() && ends[j] <= start {
            current -= 1;
            j += 1;
        }
        current += 1;
        peak = peak.max(current);
    }
    peak as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        lives: Vec<IdentityLife>,
        indices: BTreeMap<u64, i64>,
        roster: Vec<u64>,
        deaths_named: usize,
        bots: bool,
        initial: ReplayIdentityState,
        elimination: ReplayIdentityState,
        exclusion: ReplayIdentityState,
        operations: Vec<Operation>,
        queries: Vec<Query>,
        naming: BTreeMap<u32, u64>,
        peak: usize,
        candidates: Vec<u64>,
    }
    #[derive(Deserialize)]
    struct Operation {
        i: i64,
        xuid: u64,
        pi: i64,
        known: bool,
        bot: bool,
        bid: String,
        ok: bool,
        state: ReplayIdentityState,
    }
    #[derive(Deserialize)]
    struct Query {
        slot: u32,
        ts: u64,
        xuid: u64,
    }
    #[test]
    fn native_identity_bridge_and_deductions() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-identity-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            assert_eq!(identity_max_occupancy(&c.lives), c.peak, "occupancy {i}");
            assert_eq!(
                identity_roster_candidates(&c.roster, &c.indices),
                c.candidates,
                "candidates {i}"
            );
            let mut state = ReplayIdentityState::from_lives(c.lives, &c.indices);
            assert_eq!(state, c.initial, "initial {i}");
            state.resolve_roster_elimination(&c.roster, &c.indices, c.deaths_named);
            assert_eq!(state, c.elimination, "elimination {i}");
            state.resolve_temporal_exclusion(&c.roster, &c.indices, c.bots);
            assert_eq!(state, c.exclusion, "exclusion {i}");
            for (n, op) in c.operations.into_iter().enumerate() {
                let result = if op.bot {
                    state.assign_bot(op.i, &op.bid)
                } else {
                    state.assign_life(op.i, op.xuid, op.known.then_some(op.pi), "test_deduction")
                };
                assert_eq!(result, op.ok, "mutation result {i}/{n}");
                assert_eq!(state, op.state, "mutation state {i}/{n}");
            }
            for q in c.queries {
                assert_eq!(
                    state.xuid_at(q.slot, q.ts),
                    q.xuid,
                    "lookup {i} slot {} time {}",
                    q.slot,
                    q.ts
                );
            }
            assert_eq!(state.naming_bridge(), c.naming, "naming {i}");
        }
    }
}
