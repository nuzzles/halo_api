//! Mode-specific objective counters, redundant-counter checks, and player attribution.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// Kept in component/side order because all emitted counters share one budget.
fn table(mode: &str) -> Vec<(StatborgCounterKey, &'static str, bool)> {
    use StatborgSide::{A, B};
    let rows = match mode {
        "flag" => vec![
            (0, A, "flag_captures", true),
            (2, A, "kills", false),
            (3, A, "assists", false),
            (12, A, "kills", true),
            (12, B, "assists", true),
            (20, B, "flag_capture_assists", false),
            (21, A, "flag_captures", false),
            (21, B, "flag_carriers_killed", false),
            (22, A, "flag_grabs", false),
            (23, A, "flag_returns", false),
            (23, B, "flag_secures", false),
            (24, A, "flag_steals", false),
        ],
        "zone" => vec![
            (2, A, "kills", false),
            (3, A, "assists", false),
            (12, A, "kills", true),
            (12, B, "assists", true),
            (20, B, "zone_captures", false),
            (21, A, "zone_secures", false),
        ],
        "vip" => vec![(22, A, "vip_selected", false)],
        "bomb" => vec![(0, A, "bomb_detonations", false)],
        _ => vec![],
    };
    rows.into_iter()
        .map(|(component, side, name, redundant)| {
            (StatborgCounterKey { component, side }, name, redundant)
        })
        .collect()
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgNamedEvent {
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
    pub slot: i64,
    pub stat: String,
    pub comp: i64,
    pub side: StatborgSide,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatborgIdentifiedEvent {
    #[serde(flatten)]
    pub event: StatborgNamedEvent,
    #[serde(rename = "XUID")]
    pub xuid: String,
}
pub fn statborg_known_stats(mode: &str) -> BTreeSet<&'static str> {
    table(mode).into_iter().map(|(_, name, _)| name).collect()
}
/// Unsupported objective families produce no events. Redundant counters are
/// reserved for cross-checks; they must not double-count actions.
pub fn statborg_named_events(records: &[StatborgRecord], mode: &str) -> Vec<StatborgNamedEvent> {
    named_events_impl(records, mode, None)
}
/// Includes native chronology and budget warnings, with a fresh budget for this pass.
pub fn statborg_named_events_with_diagnostics(
    records: &[StatborgRecord],
    mode: &str,
) -> (Vec<StatborgNamedEvent>, Vec<StatborgBudgetDiagnostic>) {
    let mut diagnostics = StatborgBudgetDiagnostics::new(format!("named_events:{mode}"));
    let out = named_events_impl(records, mode, Some(&mut diagnostics));
    (out, diagnostics.records)
}
fn named_events_impl(
    records: &[StatborgRecord],
    mode: &str,
    mut diagnostics: Option<&mut StatborgBudgetDiagnostics>,
) -> Vec<StatborgNamedEvent> {
    let keys: Vec<_> = table(mode)
        .into_iter()
        .filter(|(_, _, redundant)| !redundant)
        .collect();
    if keys.is_empty() {
        return vec![];
    }
    let real = resolve_statborg_rounds(records).real.into_iter().collect();
    let bounds = resolve_statborg_round_bounds(records);
    let mut raw = vec![StatborgRoundSeries::new(); keys.len()];
    for r in records {
        if r.slot <= 8 || bounds.excludes(r) {
            continue;
        }
        for ((key, _, _), series) in keys.iter().zip(&mut raw) {
            let Some(v) = r.comps.get(&key.component) else {
                continue;
            };
            let value = match key.side {
                StatborgSide::A => v.a,
                StatborgSide::B => v.b,
            };
            if value < 0
                || (key.component == 0 && (!(0..=250).contains(&v.a) || !(0..=250).contains(&v.b)))
            {
                continue;
            }
            series
                .entry(r.slot)
                .or_default()
                .entry(r.round)
                .or_default()
                .push(StatborgScorePoint {
                    time_ms: r.time_ms,
                    slot: r.slot,
                    value,
                });
        }
    }
    let mut budget = StatborgEventBudget::default();
    let mut out = Vec::new();
    for ((key, stat, _), series) in keys.into_iter().zip(raw) {
        for (slot, points) in super::statborg_series::statborg_cumulate_rounds_observed(
            &series,
            &real,
            diagnostics.as_deref_mut().map(|d| &mut d.records),
        ) {
            for time_ms in
                budget.increment_times_diagnostic(&points, key, diagnostics.as_deref_mut())
            {
                out.push(StatborgNamedEvent {
                    time_ms,
                    slot,
                    stat: stat.into(),
                    comp: key.component,
                    side: key.side,
                });
            }
        }
    }
    if let Some(d) = diagnostics {
        d.summarize(&budget);
    }
    out.sort_by(|a, b| {
        (a.time_ms, a.slot, &a.stat, a.comp, a.side as u8).cmp(&(
            b.time_ms,
            b.slot,
            &b.stat,
            b.comp,
            b.side as u8,
        ))
    });
    out
}
pub fn statborg_named_counts(
    events: &[StatborgNamedEvent],
) -> BTreeMap<i64, BTreeMap<String, usize>> {
    let mut out: BTreeMap<i64, BTreeMap<String, usize>> = BTreeMap::new();
    for e in events {
        *out.entry(e.slot)
            .or_default()
            .entry(e.stat.clone())
            .or_default() += 1;
    }
    out
}
/// Canonical/redundant count differences, indexed by slot and statistic name.
pub type StatborgNamedCrossCheck = BTreeMap<i64, BTreeMap<String, [usize; 2]>>;

/// Differences are [canonical count, redundant count]. Only slots emitted by
/// the redundant counter participate, matching the native diagnostic.
pub fn statborg_cross_check_named(
    records: &[StatborgRecord],
    mode: &str,
) -> StatborgNamedCrossCheck {
    cross_check_impl(records, mode, None)
}
/// Includes chronology and budget warnings from redundant and canonical counters.
pub fn statborg_cross_check_named_with_diagnostics(
    records: &[StatborgRecord],
    mode: &str,
) -> (StatborgNamedCrossCheck, Vec<StatborgBudgetDiagnostic>) {
    let mut diagnostics = StatborgBudgetDiagnostics::new(format!("cross_check:{mode}"));
    let out = cross_check_impl(records, mode, Some(&mut diagnostics));
    (out, diagnostics.records)
}
fn cross_check_impl(
    records: &[StatborgRecord],
    mode: &str,
    mut diagnostics: Option<&mut StatborgBudgetDiagnostics>,
) -> StatborgNamedCrossCheck {
    let rows = table(mode);
    let mut budget = StatborgEventBudget::default();
    let mut out: BTreeMap<i64, BTreeMap<String, [usize; 2]>> = BTreeMap::new();
    for &(key, stat, redundant) in &rows {
        if !redundant {
            continue;
        }
        let Some(&(canonical, _, _)) = rows.iter().find(|(_, name, dup)| *name == stat && !dup)
        else {
            continue;
        };
        let duplicates = budget.counts_by_slot_diagnostic(records, key, diagnostics.as_deref_mut());
        let reference =
            budget.counts_by_slot_diagnostic(records, canonical, diagnostics.as_deref_mut());
        for (slot, got) in duplicates {
            let want = reference.get(&slot).copied().unwrap_or_default();
            if want != got {
                out.entry(slot)
                    .or_default()
                    .insert(stat.into(), [want, got]);
            }
        }
    }
    if let Some(d) = diagnostics {
        d.summarize(&budget);
    }
    out
}
fn sort_identified(events: &mut [StatborgIdentifiedEvent]) {
    events.sort_by(|a, b| {
        (a.event.time_ms, &a.xuid, &a.event.stat).cmp(&(b.event.time_ms, &b.xuid, &b.event.stat))
    });
}
/// Flat attribution preserves present-but-empty identities, as the native API does.
pub fn identify_statborg_named_events(
    events: &[StatborgNamedEvent],
    identity: &BTreeMap<i64, String>,
) -> Vec<StatborgIdentifiedEvent> {
    let mut out: Vec<_> = events
        .iter()
        .filter_map(|e| {
            identity.get(&e.slot).map(|xuid| StatborgIdentifiedEvent {
                event: e.clone(),
                xuid: xuid.clone(),
            })
        })
        .collect();
    sort_identified(&mut out);
    out
}
/// Returns attributed events and the number dropped for missing round identities.
pub fn identify_statborg_named_events_by_round(
    events: &[StatborgNamedEvent],
    identity: &StatborgRoundIdentity,
) -> (Vec<StatborgIdentifiedEvent>, usize) {
    let mut out = Vec::new();
    for e in events {
        let xuid = identity.at(e.slot, e.time_ms);
        if !xuid.is_empty() {
            out.push(StatborgIdentifiedEvent {
                event: e.clone(),
                xuid: xuid.into(),
            });
        }
    }
    let dropped = events.len() - out.len();
    sort_identified(&mut out);
    (out, dropped)
}

/// Append a complete named-event pass's ordered diagnostics when requested.
pub(super) fn named_events_with_sink(
    records: &[StatborgRecord],
    mode: &str,
    diagnostics: Option<&mut Vec<super::StatborgDiagnostic>>,
) -> Vec<StatborgNamedEvent> {
    if let Some(sink) = diagnostics {
        let (events, warnings) = statborg_named_events_with_diagnostics(records, mode);
        sink.extend(warnings);
        events
    } else {
        statborg_named_events(records, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        records: Vec<StatborgRecord>,
        mode: String,
        events: Vec<StatborgNamedEvent>,
        known: BTreeMap<String, bool>,
        counts: BTreeMap<i64, BTreeMap<String, usize>>,
        cross: BTreeMap<i64, BTreeMap<String, [usize; 2]>>,
        flat: BTreeMap<i64, String>,
        identified: Vec<StatborgIdentifiedEvent>,
        identity: StatborgRoundIdentity,
        by_round: Vec<StatborgIdentifiedEvent>,
        dropped: usize,
    }
    #[test]
    fn native_named_objectives() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/statborg-named-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                statborg_known_stats(&c.mode),
                c.known.keys().map(String::as_str).collect(),
                "known {i}"
            );
            let events = statborg_named_events(&c.records, &c.mode);
            assert_eq!(events, c.events, "events {i}");
            assert_eq!(statborg_named_counts(&events), c.counts, "counts {i}");
            assert_eq!(
                statborg_cross_check_named(&c.records, &c.mode),
                c.cross,
                "cross {i}"
            );
            assert_eq!(
                identify_statborg_named_events(&events, &c.flat),
                c.identified,
                "flat {i}"
            );
            assert_eq!(
                identify_statborg_named_events_by_round(&events, &c.identity),
                (c.by_round, c.dropped),
                "round {i}"
            );
        }
    }
}
