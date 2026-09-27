//! Published identity links with inclusive frame bounds and provenance coverage.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityClock {
    pub origin_us: u64,
    pub step_us: u64,
    pub frame_count: i64,
}
impl IdentityClock {
    pub fn last_frame(self) -> i64 {
        self.frame_count.saturating_sub(1).max(0)
    }
    pub fn frame_of(self, time_us: i64) -> i64 {
        if self.step_us == 0 {
            return 0;
        }
        time_us
            .wrapping_sub(self.origin_us as i64)
            .wrapping_div(self.step_us as i64)
            .clamp(0, self.last_frame())
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IdentityLink {
    pub source: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub method: String,
    #[serde(default, skip_serializing_if = "zero_readings")]
    pub readings: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<f64>,
    pub from: i64,
    pub to: i64,
}
fn zero_readings(n: &i64) -> bool {
    *n == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityLinkCounts {
    pub direct: usize,
    #[serde(rename = "catalogue")]
    pub catalog: usize,
    #[serde(rename = "externe")]
    pub external: usize,
    #[serde(rename = "deduit")]
    pub inferred: usize,
    #[serde(rename = "non_resolu")]
    pub unresolved: usize,
}
impl IdentityLinkCounts {
    pub fn add(&mut self, source: &str) {
        match source {
            "direct" => self.direct += 1,
            "catalogue" => self.catalog += 1,
            "externe" => self.external += 1,
            "deduit" => self.inferred += 1,
            "non_resolu" => self.unresolved += 1,
            _ => {}
        }
    }
    pub fn total(&self) -> usize {
        self.direct + self.catalog + self.external + self.inferred + self.unresolved
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityUnresolvedCauses {
    #[serde(rename = "index_hors_table")]
    pub index_out_of_table: usize,
    #[serde(rename = "sans_record")]
    pub no_creation_record: usize,
    #[serde(rename = "lectures_divergentes")]
    pub divergent_readings: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityBipedLinkCounts {
    #[serde(flatten)]
    pub counts: IdentityLinkCounts,
    #[serde(rename = "direct_propage")]
    pub direct_propagated: usize,
    #[serde(rename = "non_resolu_par_cause")]
    pub unresolved_by_cause: IdentityUnresolvedCauses,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityCoverage {
    pub film_index: IdentityLinkCounts,
    pub biped_slot: IdentityBipedLinkCounts,
    pub statborg_slot: IdentityLinkCounts,
    pub film_table: IdentityFilmTableCoverage,
}
impl IdentityCoverage {
    pub fn total(&self) -> IdentityLinkCounts {
        let mut out = IdentityLinkCounts::default();
        for c in [
            &self.film_index,
            &self.biped_slot.counts,
            &self.statborg_slot,
        ] {
            out.direct += c.direct;
            out.catalog += c.catalog;
            out.external += c.external;
            out.inferred += c.inferred;
            out.unresolved += c.unresolved;
        }
        out
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityPlayer {
    pub film_index: i64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub xuid: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bid: String,
    #[serde(default, skip_serializing_if = "super::ReplayByteString::is_empty")]
    pub name: super::ReplayByteString,
    pub link: IdentityLink,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityBipedSlot {
    pub slot: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub xuid: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bid: String,
    pub link: IdentityLink,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityStatborgSlot {
    pub slot: i64,
    pub round: i64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub xuid: String,
    pub link: IdentityLink,
}
/// Already-resolved round identities and origins. Each round key corresponds
/// to a native RoundIdentity.Rounds() entry; emitting slots come from records.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityStatborgPublication {
    pub by_round: BTreeMap<i64, BTreeMap<i64, String>>,
    pub origins: BTreeMap<i64, BTreeMap<i64, String>>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentitySection {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub players: Vec<IdentityPlayer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub biped_slots: Vec<IdentityBipedSlot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statborg_slots: Vec<IdentityStatborgSlot>,
    pub coverage: IdentityCoverage,
}
impl IdentitySection {
    /// Native publication gate: coverage alone does not make a section nonempty.
    pub fn is_empty(&self) -> bool {
        self.players.is_empty() && self.biped_slots.is_empty() && self.statborg_slots.is_empty()
    }
}

pub struct IdentitySectionInput<'a> {
    pub clock: IdentityClock,
    pub deaths: &'a [IdentityDeath],
    pub bots: &'a [IdentityBot],
    pub roster_xuids: &'a [u64],
    pub statborg: &'a IdentityStatborgPublication,
    pub statborg_records: &'a [StatborgRecord],
}
/// LegacyFilm-table names override the first nonempty name in the death feed.
pub fn identity_player_names(
    reg: &IdentityRegistryOutput,
    deaths: &[IdentityDeath],
) -> BTreeMap<u64, super::ReplayByteString> {
    let mut names = BTreeMap::new();
    for d in deaths {
        if !d.gamertag.is_empty() {
            names.entry(d.xuid).or_insert(d.gamertag.clone());
        }
    }
    for (&x, n) in &reg.tables.names {
        if !n.is_empty() {
            names.insert(x, n.clone());
        }
    }
    names
}
pub fn build_identity_section(
    reg: &IdentityRegistryOutput,
    input: IdentitySectionInput<'_>,
) -> IdentitySection {
    let c = input.clock;
    if c.step_us == 0 {
        return IdentitySection::default();
    }
    let mut out = IdentitySection::default();
    out.coverage.film_table = reg.tables.coverage.clone();
    let names = identity_player_names(reg, input.deaths);
    let mut humans = BTreeSet::new();
    for (&x, &pi) in &reg.tables.table.by_xuid {
        humans.insert(x);
        out.players.push(IdentityPlayer {
            film_index: pi,
            xuid: x.to_string(),
            bid: String::new(),
            name: names.get(&x).cloned().unwrap_or_default(),
            link: IdentityLink {
                source: "direct".into(),
                method: reg.tables.method(x).into(),
                readings: reg.tables.table.readings,
                to: c.last_frame(),
                ..Default::default()
            },
        });
    }
    for b in input.bots {
        out.players.push(IdentityPlayer {
            film_index: b.film_index,
            xuid: String::new(),
            bid: b.bid(),
            name: b.name.clone(),
            link: IdentityLink {
                source: "direct".into(),
                method: "bid".into(),
                to: c.last_frame(),
                ..Default::default()
            },
        });
    }
    for &x in input.roster_xuids {
        if x != 0 && humans.insert(x) {
            out.players.push(IdentityPlayer {
                film_index: -1,
                xuid: x.to_string(),
                bid: String::new(),
                name: names.get(&x).cloned().unwrap_or_default(),
                link: IdentityLink {
                    source: "externe".into(),
                    to: c.last_frame(),
                    ..Default::default()
                },
            });
        }
    }
    for (i, l) in reg.owners.state.lives().iter().enumerate() {
        let (source, method) = if l.xuid == 0 && l.bid.is_empty() {
            (
                "non_resolu",
                reg.owners
                    .creation
                    .causes
                    .get(&i)
                    .map(String::as_str)
                    .unwrap_or_default(),
            )
        } else {
            identity_naming_link(&l.named_by)
        };
        out.biped_slots.push(IdentityBipedSlot {
            slot: l.slot,
            xuid: if l.xuid == 0 {
                String::new()
            } else {
                l.xuid.to_string()
            },
            bid: l.bid.clone(),
            link: IdentityLink {
                source: source.into(),
                method: method.into(),
                from: c.frame_of(l.from),
                to: c.frame_of(l.to),
                ..Default::default()
            },
        });
    }
    for (&round, ids) in &input.statborg.by_round {
        for slot in statborg_emitting_player_slots(input.statborg_records, round) {
            let xuid = ids.get(&slot).cloned().unwrap_or_default();
            let origin = input
                .statborg
                .origins
                .get(&round)
                .and_then(|m| m.get(&slot))
                .map(String::as_str)
                .unwrap_or_default();
            let (source, method) = if xuid.is_empty() {
                ("non_resolu", "")
            } else {
                (
                    "deduit",
                    match origin {
                        "instants_de_mort" => "instants_de_mort",
                        "triplet_feuille" => "triplet_feuille",
                        "elimination_roster" => "elimination_roster",
                        _ => "",
                    },
                )
            };
            out.statborg_slots.push(IdentityStatborgSlot {
                slot,
                round,
                xuid,
                link: IdentityLink {
                    source: source.into(),
                    method: method.into(),
                    to: c.last_frame(),
                    ..Default::default()
                },
            });
        }
    }
    for p in &out.players {
        out.coverage.film_index.add(&p.link.source);
    }
    for b in &out.biped_slots {
        let counts = &mut out.coverage.biped_slot;
        counts.counts.add(&b.link.source);
        if b.link.source == "non_resolu" {
            match b.link.method.as_str() {
                "index_hors_table" => counts.unresolved_by_cause.index_out_of_table += 1,
                "sans_record" => counts.unresolved_by_cause.no_creation_record += 1,
                "lectures_divergentes" => counts.unresolved_by_cause.divergent_readings += 1,
                _ => {}
            }
        }
        if b.link.method == "creation_bipede_propagee" {
            counts.direct_propagated += 1;
        }
    }
    for s in &out.statborg_slots {
        out.coverage.statborg_slot.add(&s.link.source);
    }
    out
}
fn identity_naming_link(provenance: &str) -> (&'static str, &'static str) {
    match provenance {
        "biped_creation" => ("direct", "creation_bipede"),
        "biped_creation_propagee" => ("direct", "creation_bipede_propagee"),
        "death" => ("deduit", "pont_par_morts"),
        "closure" => ("deduit", "fermeture"),
        "elimination" => ("deduit", "elimination_roster"),
        "exclusion_temporelle" => ("deduit", "exclusion_temporelle"),
        "tableau_api" => ("externe", "tableau_api"),
        _ => ("deduit", ""),
    }
}
