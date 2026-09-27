//! Native kill-source identity-first feed and bot matching.
use super::{
    KillFeedEvent, KillFeedPairs, KillRoster, KillSourceCandidate, KillSourceMultiplicityKey,
    kill_source_multiplicity, strong_kill_damage_tag,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KillReadPath {
    #[serde(rename = "marche")]
    Walk,
    #[serde(rename = "scan")]
    Scan,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcedKillCandidate {
    pub candidate: KillSourceCandidate,
    pub path: KillReadPath,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillCandidateMatch {
    pub event: KillFeedEvent,
    pub candidate: Option<SourcedKillCandidate>,
    pub window_fallback: bool,
    pub fabricated: bool,
}
/// Read-only matching context shared by hybrid publication and diagnostic probes.
pub struct KillMatchingContext<'a> {
    pub roster: &'a KillRoster,
    pub feed: &'a KillFeedPairs,
    pub scan: &'a [KillSourceCandidate],
}
fn choose(
    n: usize,
    identity: impl Fn(usize) -> bool,
    window: impl Fn(usize) -> bool,
    couple: impl Fn(usize) -> bool,
) -> Option<(usize, bool)> {
    (0..n)
        .find(|&i| identity(i) && couple(i))
        .map(|i| (i, false))
        .or_else(|| (0..n).find(|&i| window(i) && couple(i)).map(|i| (i, true)))
}
fn same_packet(e: &KillFeedEvent, c: &KillSourceCandidate) -> bool {
    e.packet.is_some() && e.packet == c.packet
}
impl KillMatchingContext<'_> {
    pub fn match_feed(
        &self,
        c: &KillSourceCandidate,
        victim_only: bool,
    ) -> Option<(&KillFeedEvent, bool)> {
        choose(
            self.feed.pairs.len(),
            |i| same_packet(&self.feed.pairs[i], c),
            |i| self.feed.pairs[i].time_ms.abs_diff(c.time_ms) <= 2500,
            |i| {
                self.feed.pairs[i].victim == self.roster.name_of(c.victim)
                    && (victim_only || self.feed.pairs[i].killer == self.roster.name_of(c.killer))
            },
        )
        .map(|(i, fallback)| (&self.feed.pairs[i], fallback))
    }
    pub fn is_bot_side(&self, c: &KillSourceCandidate) -> bool {
        self.roster.evidence.is_bot_index(c.victim) || self.roster.evidence.is_bot_index(c.killer)
    }
    pub fn bot_deaths(&self) -> Vec<KillCandidateMatch> {
        self.feed
            .bot_read
            .iter()
            .map(|b| (&b.event, b.victim_index, false))
            .chain(self.feed.orphan_kills.iter().map(|e| (e, -1, false)))
            .chain(self.feed.neighboring_fallback.iter().map(|e| (e, -1, true)))
            .map(|(e, victim, fabricated)| {
                let selected = choose(
                    self.scan.len(),
                    |i| same_packet(e, &self.scan[i]),
                    |i| e.time_ms.abs_diff(self.scan[i].time_ms) <= 2500,
                    |i| {
                        let c = &self.scan[i];
                        (if victim >= 0 {
                            c.victim == victim
                        } else {
                            self.roster.evidence.is_bot_index(c.victim)
                        }) && self.roster.name_of(c.killer) == e.killer
                    },
                );
                KillCandidateMatch {
                    event: e.clone(),
                    fabricated,
                    window_fallback: selected.is_some_and(|(_, fallback)| fallback),
                    candidate: selected.map(|(i, _)| SourcedKillCandidate {
                        candidate: self.scan[i].clone(),
                        path: KillReadPath::Scan,
                    }),
                }
            })
            .collect()
    }
    pub fn bot_killer_deaths(&self, all: &[SourcedKillCandidate]) -> Vec<KillCandidateMatch> {
        self.feed
            .orphan_deaths
            .iter()
            .map(|e| {
                let selected = choose(
                    all.len(),
                    |i| same_packet(e, &all[i].candidate),
                    |i| e.time_ms.abs_diff(all[i].candidate.time_ms) <= 2500,
                    |i| {
                        self.roster.evidence.is_bot_index(all[i].candidate.killer)
                            && self.roster.name_of(all[i].candidate.victim) == e.victim
                    },
                );
                KillCandidateMatch {
                    event: e.clone(),
                    fabricated: false,
                    window_fallback: selected.is_some_and(|(_, fallback)| fallback),
                    candidate: selected.map(|(i, _)| all[i].clone()),
                }
            })
            .collect()
    }
    pub fn ghost_pairs(&self) -> BTreeSet<i64> {
        self.bot_deaths()
            .into_iter()
            .filter(|m| m.fabricated && m.candidate.is_some())
            .map(|m| m.event.time_ms)
            .collect()
    }
    /// Unclaimed deaths use the closest temporal candidate after trying identity.
    /// All other native matching paths use the first candidate in the window.
    pub fn unclaimed<'a>(
        &self,
        e: &KillFeedEvent,
        all: &'a [SourcedKillCandidate],
    ) -> Option<(&'a SourcedKillCandidate, bool)> {
        let eligible = |c: &&SourcedKillCandidate| {
            c.candidate.victim == c.candidate.killer
                && self.roster.name_of(c.candidate.victim) == e.victim
        };
        if let Some(c) = all
            .iter()
            .filter(eligible)
            .find(|c| same_packet(e, &c.candidate))
        {
            return Some((c, false));
        }
        all.iter()
            .filter(eligible)
            .filter(|c| e.time_ms.abs_diff(c.candidate.time_ms) <= 2500)
            .min_by_key(|c| e.time_ms.abs_diff(c.candidate.time_ms))
            .map(|c| (c, true))
    }
}
pub fn kill_candidate_multiplicity(
    counts: &BTreeMap<KillSourceMultiplicityKey, usize>,
    c: &KillSourceCandidate,
) -> usize {
    counts
        .get(&KillSourceMultiplicityKey {
            bit: c.bit,
            tag: c.tag,
            victim: c.victim,
        })
        .copied()
        .unwrap_or(0)
}
pub fn kill_self_source_allowed(
    c: &KillSourceCandidate,
    counts: &BTreeMap<KillSourceMultiplicityKey, usize>,
    maximum: usize,
    strong_required: bool,
) -> bool {
    kill_candidate_multiplicity(counts, c) < maximum
        && (!strong_required || strong_kill_damage_tag(c.tag))
}
/// Prefer walked records at the exact packet/bit position, retaining scan-only evidence.
pub fn combine_kill_candidates(
    walk: &[KillSourceCandidate],
    scan: &[KillSourceCandidate],
) -> (
    Vec<SourcedKillCandidate>,
    usize,
    BTreeMap<KillSourceMultiplicityKey, usize>,
) {
    let key = |c: &KillSourceCandidate| {
        let p = c.packet.map(|p| (p.chunk, p.packet)).unwrap_or((0, 0));
        (p, c.bit)
    };
    let positions: BTreeSet<_> = walk.iter().filter(|c| c.bit >= 0).map(key).collect();
    let mut all: Vec<_> = walk
        .iter()
        .map(|c| SourcedKillCandidate {
            candidate: c.clone(),
            path: KillReadPath::Walk,
        })
        .collect();
    let mut redundant = 0;
    for c in scan {
        if positions.contains(&key(c)) {
            redundant += 1;
        } else {
            all.push(SourcedKillCandidate {
                candidate: c.clone(),
                path: KillReadPath::Scan,
            });
        }
    }
    let counts =
        kill_source_multiplicity(&all.iter().map(|c| c.candidate.clone()).collect::<Vec<_>>());
    (all, redundant, counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    #[test]
    fn native_kill_matching_oracle() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kill-matching-v41.json.zlib")[..])
            .read_to_end(&mut bytes)
            .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
        for (index, row) in rows.iter().enumerate() {
            let health: super::super::KillSourceHealth =
                serde_json::from_value(row["health"].clone()).unwrap();
            let mut actual = json!({"alerts":health.alerts(),"degradations":health.degradations(),"verdict":health.verdict(),"metrics":health.metric_pairs(),"unexplained_total":health.unexplained_total(),"unexplained_ratio":health.unexplained_ratio(),"coverage_ratio":health.coverage_ratio(),"publishable":health.line_by_line_publishable(row["margin"].as_i64().unwrap(),row["determined"].as_bool().unwrap())});
            let mut expected = row["health_expected"].clone();
            for key in ["unexplained_ratio", "coverage_ratio"] {
                assert_eq!(
                    actual[key].as_f64(),
                    expected[key].as_f64(),
                    "health ratio {index}"
                );
                actual.as_object_mut().unwrap().remove(key);
                expected.as_object_mut().unwrap().remove(key);
            }
            assert_eq!(actual, expected, "health {index}");
            let parse =
                |key: &str| serde_json::from_value::<Vec<KillFeedEvent>>(row[key].clone()).unwrap();
            let feed = KillFeedPairs {
                bot_read: serde_json::from_value(row["read_bots"].clone()).unwrap(),
                pairs: parse("pairs"),
                orphan_deaths: parse("orphan_deaths"),
                orphan_kills: parse("orphan_kills"),
                neighboring_fallback: parse("fabricated"),
                ..Default::default()
            };
            let roster = KillRoster {
                permutation: vec![0, 1, 2, 3],
                evidence: super::super::KillRosterPins {
                    names: ["A", "B", "C", "Bot"].map(str::to_string).to_vec(),
                    pins: BTreeMap::from([(3, 3)]),
                    ..Default::default()
                },
                ..Default::default()
            };
            let scan: Vec<KillSourceCandidate> =
                serde_json::from_value(row["scan"].clone()).unwrap();
            let walk: Vec<KillSourceCandidate> =
                serde_json::from_value(row["walk"].clone()).unwrap();
            let ctx = KillMatchingContext {
                roster: &roster,
                feed: &feed,
                scan: &scan,
            };
            let (all, redundant, counts) = combine_kill_candidates(&walk, &scan);
            assert_eq!(json!(all), row["all"], "all {index}");
            assert_eq!(json!(redundant), row["redundant"]);
            let matched: Vec<_> = scan.iter().map(|c| {
                let m = |v| ctx.match_feed(c,v).map(|(e,f)| json!({"event":e,"fallback":f}));
                json!({"exact":m(false),"victim":m(true),"self_ok":kill_self_source_allowed(c,&counts,row["multiplicity_max"].as_u64().unwrap() as usize,row["strong_required"].as_bool().unwrap()),"bot":ctx.is_bot_side(c)})
            }).collect();
            assert_eq!(json!(matched), row["matches"], "matches {index}");
            assert_eq!(json!(ctx.bot_deaths()), row["bots"], "bots {index}");
            assert_eq!(
                json!(ctx.bot_killer_deaths(&all)),
                row["killers"],
                "killers {index}"
            );
            let unclaimed: Vec<_> = feed
                .orphan_deaths
                .iter()
                .map(|e| {
                    ctx.unclaimed(e, &all)
                        .map(|(c, f)| json!({"candidate":c,"fallback":f}))
                })
                .collect();
            assert_eq!(json!(unclaimed), row["unclaimed"], "unclaimed {index}");
            let ghosts: BTreeSet<i64> = serde_json::from_value(row["ghosts"].clone()).unwrap();
            assert_eq!(ctx.ghost_pairs(), ghosts, "ghosts {index}");
            assert_eq!(
                json!(super::super::attribute_kill_sources(
                    &ctx,
                    &walk,
                    0,
                    row["self_source"].as_bool().unwrap(),
                    row["multiplicity_max"].as_u64().unwrap() as usize,
                    row["strong_required"].as_bool().unwrap()
                )),
                row["hybrid"],
                "hybrid {index}"
            );
        }
    }
}
