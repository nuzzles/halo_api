//! Native statborg identity joins using death-counter timestamps.
use super::{StatborgPlayerLine, StatborgRecord, resolve_statborg_rounds, statborg_slot_identity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgDeathInstant {
    #[serde(rename = "XUID")]
    pub xuid: String,
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
}
/// Input records retain their existing order, as in the native decoder. Only the
/// death feed is sorted. Counters above 1000 or below zero provide no evidence.
pub fn statborg_death_progressions(
    records: &[StatborgRecord],
    round: Option<i64>,
) -> BTreeMap<i64, Vec<i64>> {
    let mut out = BTreeMap::<i64, Vec<i64>>::new();
    let mut previous = BTreeMap::<i64, i64>::new();
    for r in records {
        if r.slot <= 8 || round.is_some_and(|round| round != r.round) {
            continue;
        }
        let Some(v) = r.comps.get(&2).filter(|v| (0..=1000).contains(&v.b)) else {
            continue;
        };
        let times = out.entry(r.slot).or_default();
        let prev = previous.entry(r.slot).or_default();
        if v.b > *prev {
            times.extend(std::iter::repeat_n(r.time_ms, (v.b - *prev) as usize));
            *prev = v.b;
        }
    }
    out
}
pub fn statborg_death_coincidences(instants: &[i64], thread: &[i64]) -> usize {
    let mut used = vec![false; thread.len()];
    let mut n = 0;
    for &t in instants {
        let mut best = None;
        let mut delta = 151;
        for (i, &u) in thread.iter().enumerate() {
            let d = u.wrapping_sub(t).wrapping_abs();
            if !used[i] && d < delta {
                delta = d;
                best = Some(i);
            }
        }
        if let Some(i) = best {
            used[i] = true;
            n += 1;
        }
    }
    n
}
fn thread_by_xuid(deaths: &[StatborgDeathInstant]) -> BTreeMap<String, Vec<i64>> {
    let mut out = BTreeMap::<String, Vec<i64>>::new();
    for d in deaths.iter().filter(|d| !d.xuid.is_empty()) {
        out.entry(d.xuid.clone()).or_default().push(d.time_ms);
    }
    for times in out.values_mut() {
        times.sort_unstable();
    }
    out
}
fn identity_claims(
    progressions: BTreeMap<i64, Vec<i64>>,
    thread: &BTreeMap<String, Vec<i64>>,
) -> BTreeMap<i64, String> {
    let mut claims = BTreeMap::<String, Vec<i64>>::new();
    for (slot, times) in progressions {
        let (mut best, mut second, mut winner) = (0, 0, "");
        for (x, feed) in thread {
            let n = statborg_death_coincidences(&times, feed);
            if n > best {
                second = best;
                best = n;
                winner = x;
            } else if n > second {
                second = n;
            }
        }
        if best >= 3 && best >= 2 * second {
            claims.entry(winner.to_owned()).or_default().push(slot);
        }
    }
    claims
        .into_iter()
        .filter_map(|(x, slots)| {
            if slots.len() == 1 {
                Some((slots[0], x))
            } else {
                None
            }
        })
        .collect()
}
pub fn statborg_identity_by_deaths(
    records: &[StatborgRecord],
    deaths: &[StatborgDeathInstant],
) -> BTreeMap<i64, String> {
    if deaths.is_empty() {
        return BTreeMap::new();
    }
    identity_claims(
        statborg_death_progressions(records, None),
        &thread_by_xuid(deaths),
    )
}
pub fn statborg_identity_by_round(
    records: &[StatborgRecord],
    deaths: &[StatborgDeathInstant],
) -> BTreeMap<i64, BTreeMap<i64, String>> {
    let rounds = resolve_statborg_rounds(records).real;
    if rounds.len() <= 1 {
        return BTreeMap::from([(
            rounds.first().copied().unwrap_or(0),
            statborg_identity_by_deaths(records, deaths),
        )]);
    }
    let thread = thread_by_xuid(deaths);
    rounds
        .into_iter()
        .map(|r| {
            (
                r,
                identity_claims(statborg_death_progressions(records, Some(r)), &thread),
            )
        })
        .collect()
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgIdentityStats {
    pub by_totals: usize,
    pub by_deaths: usize,
    pub conflicts: usize,
    pub source: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgResolvedIdentity {
    pub by_slot: BTreeMap<i64, String>,
    pub stats: StatborgIdentityStats,
}
/// Totals win unless timestamps identify strictly more slots. In that case,
/// conflicting assignments are removed from the death-based result.
pub fn resolve_statborg_identity(
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
    deaths: &[StatborgDeathInstant],
) -> StatborgResolvedIdentity {
    let totals = statborg_slot_identity(records, lines).by_slot;
    resolve_identity_with_totals(records, deaths, totals)
}

/// Retain the totals-pass diagnostics before resolving the death-based alternative.
pub fn resolve_statborg_identity_with_diagnostics(
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
    deaths: &[StatborgDeathInstant],
) -> (StatborgResolvedIdentity, Vec<super::StatborgDiagnostic>) {
    let (totals, diagnostics) = super::statborg_slot_identity_with_diagnostics(records, lines);
    (
        resolve_identity_with_totals(records, deaths, totals.by_slot),
        diagnostics,
    )
}

fn resolve_identity_with_totals(
    records: &[StatborgRecord],
    deaths: &[StatborgDeathInstant],
    totals: BTreeMap<i64, String>,
) -> StatborgResolvedIdentity {
    let by_deaths = statborg_identity_by_deaths(records, deaths);
    let mut stats = StatborgIdentityStats {
        by_totals: totals.len(),
        by_deaths: by_deaths.len(),
        conflicts: 0,
        source: "totals".into(),
    };
    if by_deaths.len() <= totals.len() {
        return StatborgResolvedIdentity {
            by_slot: totals,
            stats,
        };
    }
    stats.source = "deaths".into();
    let by_slot = by_deaths
        .into_iter()
        .filter(|(s, x)| {
            if totals.get(s).is_some_and(|other| other != x) {
                stats.conflicts += 1;
                false
            } else {
                true
            }
        })
        .collect();
    StatborgResolvedIdentity { by_slot, stats }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgRoundStart {
    pub round: i64,
    pub start_ms: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgRoundIdentity {
    pub publication: super::IdentityStatborgPublication,
    pub starts: Vec<StatborgRoundStart>,
}
pub fn resolve_statborg_round_identity(
    records: &[StatborgRecord],
    deaths: &[StatborgDeathInstant],
) -> StatborgRoundIdentity {
    let by_round = statborg_identity_by_round(records, deaths);
    let origins = by_round
        .iter()
        .map(|(&round, ids)| {
            (
                round,
                ids.keys()
                    .map(|&slot| (slot, "instants_de_mort".into()))
                    .collect(),
            )
        })
        .collect();
    let mut starts = Vec::new();
    if by_round.len() > 1 {
        let consensus = super::statborg_round_starts_ms(records);
        let mut times = BTreeMap::<i64, i64>::new();
        for r in records {
            if by_round.contains_key(&r.round) && !consensus.contains_key(&r.round) {
                times
                    .entry(r.round)
                    .and_modify(|t| *t = (*t).min(r.time_ms))
                    .or_insert(r.time_ms);
            }
        }
        for &round in by_round.keys() {
            if let Some(&t) = consensus.get(&round) {
                times.insert(round, t);
            }
        }
        starts = times
            .into_iter()
            .map(|(round, start_ms)| StatborgRoundStart { round, start_ms })
            .collect();
        starts.sort_by_key(|s| (s.start_ms, s.round));
    }
    StatborgRoundIdentity {
        publication: super::IdentityStatborgPublication { by_round, origins },
        starts,
    }
}
impl StatborgRoundIdentity {
    /// Native flat identity belongs to round zero, even when its map is empty.
    /// It supplies no naming provenance or round-start observations.
    pub fn from_flat(identity: BTreeMap<i64, String>) -> Self {
        Self {
            publication: super::IdentityStatborgPublication {
                by_round: BTreeMap::from([(0, identity)]),
                origins: BTreeMap::new(),
            },
            starts: Vec::new(),
        }
    }
    pub fn origin(&self, round: i64, slot: i64) -> &str {
        self.publication
            .origins
            .get(&round)
            .and_then(|m| m.get(&slot))
            .map(String::as_str)
            .unwrap_or_default()
    }
    pub fn at_round(&self, round: i64, slot: i64) -> &str {
        self.publication
            .by_round
            .get(&round)
            .and_then(|m| m.get(&slot))
            .map(String::as_str)
            .unwrap_or_default()
    }
    /// Native resolution means a round map exists, not that every slot is named.
    pub fn resolved(&self) -> bool {
        !self.publication.by_round.is_empty()
    }
    pub fn rounds(&self) -> Vec<i64> {
        self.publication.by_round.keys().copied().collect()
    }
    /// Count stored slot entries, including empty values, as in native NamedCount.
    pub fn named_count(&self) -> usize {
        self.publication.by_round.values().map(BTreeMap::len).sum()
    }
    pub fn at(&self, slot: i64, time_ms: i64) -> &str {
        let round = if self.publication.by_round.len() == 1 {
            self.publication.by_round.keys().next().copied()
        } else {
            self.starts
                .iter()
                .take_while(|s| s.start_ms <= time_ms)
                .last()
                .or(self.starts.first())
                .map(|s| s.round)
        };
        round
            .and_then(|r| self.publication.by_round.get(&r))
            .and_then(|m| m.get(&slot))
            .map(String::as_str)
            .unwrap_or_default()
    }
    /// Complete only a single-round identity, never replacing an occupied slot
    /// or assigning one player to two slots. New links retain scoreboard provenance.
    pub fn completed_by_lines(
        &self,
        records: &[StatborgRecord],
        lines: &[StatborgPlayerLine],
    ) -> Self {
        self.completed_by_lines_observed(records, lines, None)
    }

    /// Retain native identity-pass warnings only when the single-round/nonempty
    /// scoreboard gates allow the pass to run. Source records are not rescanned.
    pub fn completed_by_lines_with_diagnostics(
        &self,
        records: &[StatborgRecord],
        lines: &[StatborgPlayerLine],
    ) -> (Self, Vec<super::StatborgDiagnostic>) {
        let mut diagnostics = Vec::new();
        let out = self.completed_by_lines_observed(records, lines, Some(&mut diagnostics));
        (out, diagnostics)
    }

    fn completed_by_lines_observed(
        &self,
        records: &[StatborgRecord],
        lines: &[StatborgPlayerLine],
        diagnostics: Option<&mut Vec<super::StatborgDiagnostic>>,
    ) -> Self {
        let mut out = self.clone();
        if lines.is_empty() || out.publication.by_round.len() != 1 {
            return out;
        }
        let triplet = if let Some(sink) = diagnostics {
            let (identity, warnings) =
                super::statborg_slot_identity_with_diagnostics(records, lines);
            sink.extend(warnings);
            identity.by_slot
        } else {
            statborg_slot_identity(records, lines).by_slot
        };
        if triplet.is_empty() {
            return out;
        }
        let (&round, ids) = out.publication.by_round.iter_mut().next().unwrap();
        let mut used: std::collections::BTreeSet<_> = ids.values().cloned().collect();
        // Native reconstruction retains provenance only for the returned links.
        // Early returns above preserve the original maps verbatim.
        let mut origins: BTreeMap<_, _> = ids
            .keys()
            .map(|&slot| {
                (
                    slot,
                    self.publication
                        .origins
                        .get(&round)
                        .and_then(|m| m.get(&slot))
                        .cloned()
                        .unwrap_or_default(),
                )
            })
            .collect();
        for (slot, x) in triplet {
            if ids.contains_key(&slot) || !used.insert(x.clone()) {
                continue;
            }
            ids.insert(slot, x);
            origins.insert(slot, "triplet_feuille".into());
        }
        out.publication.origins = [(round, origins)].into();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_identity_completion_diagnostics_and_provenance() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-completion-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 512);
        let identity = |mut value: serde_json::Value| -> StatborgRoundIdentity {
            // Native provenance maps are private; nil and empty both implement
            // Origin() as the empty string. Rust represents both as an empty map.
            if value["publication"]["origins"].is_null() {
                value["publication"]["origins"] = serde_json::json!({});
            }
            serde_json::from_value(value).unwrap()
        };
        let mut warnings = 0;
        let mut rebuilt = 0;
        for (i, row) in rows.into_iter().enumerate() {
            let input = identity(row["input"].clone());
            let records: Vec<StatborgRecord> =
                serde_json::from_value(row["records"].clone()).unwrap();
            let lines: Vec<StatborgPlayerLine> =
                serde_json::from_value(row["lines"].clone()).unwrap();
            let (actual, diagnostics) = input.completed_by_lines_with_diagnostics(&records, &lines);
            assert_eq!(actual, identity(row["result"].clone()), "result {i}");
            assert_eq!(
                input,
                identity(row["after"].clone()),
                "input remains unchanged {i}"
            );
            assert_eq!(
                actual,
                input.completed_by_lines(&records, &lines),
                "unobserved {i}"
            );
            assert_eq!(
                diagnostics,
                serde_json::from_value::<Vec<super::super::StatborgDiagnostic>>(
                    row["warnings"].clone()
                )
                .unwrap(),
                "warnings {i}"
            );
            warnings += diagnostics.len();
            rebuilt += usize::from(actual.publication.origins != input.publication.origins);
            if lines.is_empty() || input.publication.by_round.len() != 1 {
                assert!(diagnostics.is_empty(), "gate {i}");
                assert_eq!(actual, input);
            }
        }
        assert!(warnings > 0 && rebuilt > 0);
        println!("identity completion: {warnings} warnings, {rebuilt} provenance rebuilds");
    }
    #[derive(Deserialize)]
    struct Case {
        records: Vec<StatborgRecord>,
        deaths: Vec<StatborgDeathInstant>,
        lines: Vec<StatborgPlayerLine>,
        progressions: BTreeMap<i64, Vec<i64>>,
        round: i64,
        round_progressions: BTreeMap<i64, Vec<i64>>,
        by_deaths: BTreeMap<i64, String>,
        by_round: BTreeMap<i64, BTreeMap<i64, String>>,
        resolved: StatborgResolvedIdentity,
        resolved_logs: Vec<super::super::StatborgDiagnostic>,
        round_identity: StatborgRoundIdentity,
        completed: StatborgRoundIdentity,
        lookups: Vec<(i64, i64, String)>,
        a: Vec<i64>,
        b: Vec<i64>,
        coincidences: usize,
    }
    #[test]
    fn native_statborg_death_identity() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/statborg-identity-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let round_identity = resolve_statborg_round_identity(&c.records, &c.deaths);
            assert_eq!(round_identity, c.round_identity, "round identity {i}");
            let completed = round_identity.completed_by_lines(&c.records, &c.lines);
            assert_eq!(completed, c.completed, "completed {i}");
            for (slot, time, x) in &c.lookups {
                assert_eq!(completed.at(*slot, *time), x, "lookup {i}");
            }
            assert_eq!(
                statborg_death_progressions(&c.records, None),
                c.progressions,
                "progressions {i}"
            );
            assert_eq!(
                statborg_death_progressions(&c.records, Some(c.round)),
                c.round_progressions,
                "round progressions {i}"
            );
            assert_eq!(
                statborg_death_coincidences(&c.a, &c.b),
                c.coincidences,
                "coincidences {i}"
            );
            assert_eq!(
                statborg_identity_by_deaths(&c.records, &c.deaths),
                c.by_deaths,
                "deaths {i}"
            );
            assert_eq!(
                statborg_identity_by_round(&c.records, &c.deaths),
                c.by_round,
                "rounds {i}"
            );
            let (observed, logs) =
                resolve_statborg_identity_with_diagnostics(&c.records, &c.lines, &c.deaths);
            assert_eq!(observed, c.resolved, "observed resolved {i}");
            assert_eq!(logs, c.resolved_logs, "resolved logs {i}");
            assert_eq!(
                resolve_statborg_identity(&c.records, &c.lines, &c.deaths),
                c.resolved,
                "resolved {i}"
            );
        }
    }
}

#[cfg(test)]
mod query_tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Query {
        round: i64,
        slot: i64,
        value: String,
        origin: String,
        flat_value: String,
        flat_origin: String,
    }
    #[derive(Deserialize)]
    struct Case {
        by_round: BTreeMap<i64, BTreeMap<i64, String>>,
        origins: BTreeMap<i64, BTreeMap<i64, String>>,
        flat: BTreeMap<i64, String>,
        queries: Vec<Query>,
        resolved: bool,
        rounds: Vec<i64>,
        named: usize,
        flat_resolved: bool,
        flat_rounds: Vec<i64>,
        flat_named: usize,
    }
    #[test]
    fn native_round_identity_queries() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/round-identity-queries-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 256);
        for (i, c) in cases.into_iter().enumerate() {
            let identity = StatborgRoundIdentity {
                publication: crate::theater::IdentityStatborgPublication {
                    by_round: c.by_round,
                    origins: c.origins,
                },
                starts: Vec::new(),
            };
            let flat = StatborgRoundIdentity::from_flat(c.flat);
            assert_eq!(identity.resolved(), c.resolved, "resolved {i}");
            assert_eq!(identity.rounds(), c.rounds, "rounds {i}");
            assert_eq!(identity.named_count(), c.named, "named {i}");
            assert_eq!(flat.resolved(), c.flat_resolved);
            assert_eq!(flat.rounds(), c.flat_rounds);
            assert_eq!(flat.named_count(), c.flat_named);
            assert!(flat.starts.is_empty());
            for q in c.queries {
                assert_eq!(identity.at_round(q.round, q.slot), q.value, "value {i}");
                assert_eq!(identity.origin(q.round, q.slot), q.origin, "origin {i}");
                assert_eq!(
                    flat.at_round(q.round, q.slot),
                    q.flat_value,
                    "flat value {i}"
                );
                assert_eq!(
                    flat.origin(q.round, q.slot),
                    q.flat_origin,
                    "flat origin {i}"
                );
            }
            assert_eq!(
                serde_json::from_slice::<StatborgRoundIdentity>(
                    &serde_json::to_vec(&flat).unwrap()
                )
                .unwrap(),
                flat
            );
        }
    }
}
