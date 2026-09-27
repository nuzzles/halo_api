//! Native team projection, roster publication, and external comparison counts.
use super::{FilmPlayerTeams, IdentityBot, IdentityRegistryOutput, PlayerIndexTable, ReplayTrack};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayRosterEntry {
    pub xuid: String,
    pub film_index: i64,
    #[serde(skip_serializing_if = "crate::theater::ReplayByteString::is_empty")]
    pub name: crate::theater::ReplayByteString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<i64>,
    #[serde(skip_serializing_if = "is_false")]
    pub bot: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub bid: String,
    pub seat: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub seat_source: String,
}
fn is_false(v: &bool) -> bool {
    !*v
}
fn zero(v: &usize) -> bool {
    *v == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayTeamCoverage {
    pub read: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub refusal: String,
    pub records: i64,
    pub rejected: i64,
    pub divergences: i64,
    pub film: usize,
    pub no_team: usize,
    pub unread: usize,
    pub accord: usize,
    pub contradiction: usize,
    pub silence: usize,
    pub tracks: usize,
    pub tracks_named: usize,
    #[serde(skip_serializing_if = "zero")]
    pub tracks_slot_ambiguous: usize,
}
pub struct ReplayTeamPublication<'a> {
    teams: &'a FilmPlayerTeams,
    by_xuid: BTreeMap<u64, i64>,
    by_slot: &'a BTreeMap<u32, i64>,
    ambiguous: &'a BTreeSet<u32>,
    control: &'a BTreeMap<String, i64>,
}
impl<'a> ReplayTeamPublication<'a> {
    pub fn new(
        registry: &'a IdentityRegistryOutput,
        teams: &'a FilmPlayerTeams,
        control: &'a BTreeMap<String, i64>,
    ) -> Self {
        Self::from_tables(
            &registry.tables.table,
            registry.owners.state.indices_by_slot(),
            registry.owners.state.ambiguous_slots(),
            teams,
            control,
        )
    }
    pub fn from_tables(
        indices: &PlayerIndexTable,
        by_slot: &'a BTreeMap<u32, i64>,
        ambiguous: &'a BTreeSet<u32>,
        teams: &'a FilmPlayerTeams,
        control: &'a BTreeMap<String, i64>,
    ) -> Self {
        let by_xuid = indices
            .by_xuid
            .iter()
            .filter_map(|(&x, i)| teams.by_index.get(i).map(|&t| (x, t)))
            .collect();
        Self {
            teams,
            by_xuid,
            by_slot,
            ambiguous,
            control,
        }
    }
    pub fn team_for_xuid(&self, xuid: u64) -> Option<i64> {
        self.by_xuid.get(&xuid).copied()
    }
    /// The second result distinguishes ambiguous slots from missing evidence.
    pub fn team_for_slot(&self, slot: u32) -> (Option<i64>, bool) {
        if self.ambiguous.contains(&slot) {
            return (None, true);
        }
        (
            self.by_slot
                .get(&slot)
                .and_then(|i| self.teams.by_index.get(i))
                .copied(),
            false,
        )
    }
    /// Returns total tracks, named tracks (including no-team), and ambiguous slots.
    /// Existing team values survive when no reading can be assigned.
    pub fn apply_to_tracks(&self, tracks: &mut [ReplayTrack]) -> [usize; 3] {
        let mut counts = [tracks.len(), 0, 0];
        for t in tracks {
            let xuid = (!t.xuid.is_empty() && t.xuid.bytes().all(|b| b.is_ascii_digit()))
                .then(|| t.xuid.parse::<u64>().ok())
                .flatten();
            if let Some(team) = xuid.and_then(|x| self.team_for_xuid(x)) {
                t.team = team;
                counts[1] += 1;
                continue;
            }
            match self.team_for_slot(t.slot) {
                (Some(team), _) => {
                    t.team = team;
                    counts[1] += 1;
                }
                (None, true) => counts[2] += 1,
                _ => {}
            }
        }
        counts
    }
    pub fn coverage(&self, counts: [usize; 3], roster: &[ReplayRosterEntry]) -> ReplayTeamCoverage {
        let r = &self.teams.report;
        let refusal = if r.archetype_absent {
            "archetype_absent"
        } else if r.component_mismatch {
            "composant_inattendu"
        } else if r.component.is_empty() {
            "non_balaye"
        } else {
            ""
        };
        let mut cov = ReplayTeamCoverage {
            read: r.has_readings(),
            refusal: refusal.into(),
            records: r.records,
            rejected: r
                .unreached
                .wrapping_add(r.out_of_domain_index)
                .wrapping_add(r.out_of_domain_value),
            divergences: r.entity_divergences.wrapping_add(r.index_divergences),
            tracks: counts[0],
            tracks_named: counts[1],
            tracks_slot_ambiguous: counts[2],
            ..Default::default()
        };
        for e in roster {
            let Some(&team) = self.teams.by_index.get(&e.film_index) else {
                cov.unread += 1;
                continue;
            };
            cov.film += 1;
            cov.no_team += usize::from(team == -1);
            if e.xuid.is_empty() {
                continue;
            }
            match self.control.get(&e.xuid) {
                None => cov.silence += 1,
                Some(&t) if t == team => cov.accord += 1,
                _ => cov.contradiction += 1,
            }
        }
        cov
    }
    pub fn flag_carrier_teams(&self) -> BTreeMap<String, i64> {
        self.by_xuid
            .iter()
            .filter(|(_, t)| **t != -1)
            .map(|(x, &t)| (x.to_string(), t))
            .collect()
    }
    /// Build the roster from the registry's effective indices and film/death names.
    pub fn roster_from_registry(
        &self,
        registry: &IdentityRegistryOutput,
        deaths: &[super::IdentityDeath],
        bots: &[IdentityBot],
    ) -> Vec<ReplayRosterEntry> {
        self.roster(
            &registry.tables.table,
            &super::identity_player_names(registry, deaths),
            bots,
        )
    }
    /// Human indices exclude bot entries. Bot names are deduplicated globally,
    /// preserving the first eligible declaration; seat assignment is a later pass.
    pub fn roster(
        &self,
        indices: &PlayerIndexTable,
        names: &BTreeMap<u64, crate::theater::ReplayByteString>,
        bots: &[IdentityBot],
    ) -> Vec<ReplayRosterEntry> {
        let human: BTreeSet<_> = indices.by_xuid.values().copied().collect();
        let mut out: Vec<_> = indices
            .by_xuid
            .iter()
            .map(|(&x, &i)| ReplayRosterEntry {
                xuid: x.to_string(),
                film_index: i,
                name: names.get(&x).cloned().unwrap_or_default(),
                team: self.teams.by_index.get(&i).copied(),
                ..Default::default()
            })
            .collect();
        let mut seen = BTreeSet::new();
        for b in bots {
            if human.contains(&b.film_index) || b.name.is_empty() || !seen.insert(&b.name) {
                continue;
            }
            out.push(ReplayRosterEntry {
                film_index: b.film_index,
                name: b.name.clone(),
                team: self.teams.by_index.get(&b.film_index).copied(),
                bot: true,
                bid: b.bid(),
                ..Default::default()
            });
        }
        super::native_sort::sort_by(&mut out, |a, b| {
            (a.film_index, &a.xuid).cmp(&(b.film_index, &b.xuid))
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        indices: PlayerIndexTable,
        names: BTreeMap<u64, crate::theater::ReplayByteString>,
        owners: BTreeMap<u32, i64>,
        ambiguous: BTreeSet<u32>,
        teams: FilmPlayerTeams,
        control: BTreeMap<String, i64>,
        bots: Vec<IdentityBot>,
        tracks: Vec<ReplayTrack>,
        output: Vec<ReplayTrack>,
        roster: Vec<ReplayRosterEntry>,
        coverage: ReplayTeamCoverage,
        counts: [usize; 3],
        flags: BTreeMap<String, i64>,
    }
    #[test]
    fn native_team_and_roster_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-teams-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, mut c) in cases.into_iter().enumerate() {
            let p = ReplayTeamPublication::from_tables(
                &c.indices,
                &c.owners,
                &c.ambiguous,
                &c.teams,
                &c.control,
            );
            let roster = p.roster(&c.indices, &c.names, &c.bots);
            assert_eq!(roster, c.roster, "roster {i}");
            let counts = p.apply_to_tracks(&mut c.tracks);
            assert_eq!(counts, c.counts, "counts {i}");
            assert_eq!(c.tracks, c.output, "tracks {i}");
            assert_eq!(p.coverage(counts, &roster), c.coverage, "coverage {i}");
            assert_eq!(p.flag_carrier_teams(), c.flags, "flag teams {i}");
        }
    }
}
