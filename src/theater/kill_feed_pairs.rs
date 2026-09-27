//! Native kill-feed pair reconstruction using read (not inferred) roster pins.
use super::{FilmKillEvent, KillFeed, KillFeedEvent};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Direct roster evidence used before solving the index permutation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillRosterPins {
    pub names: Vec<String>,
    pub pins: BTreeMap<i32, i32>,
    pub seat_pins: BTreeSet<i32>,
    pub motif_pins: BTreeSet<i32>,
}
impl KillRosterPins {
    pub fn pinned_name(&self, index: i32) -> Option<&str> {
        let pos = usize::try_from(*self.pins.get(&index)?).ok()?;
        self.names.get(pos).map(String::as_str)
    }
    pub fn is_bot_index(&self, index: i32) -> bool {
        !self.seat_pins.contains(&index)
            && !self.motif_pins.contains(&index)
            && self.pins.contains_key(&index)
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillPairStats {
    pub same_instant: usize,
    pub read: usize,
    pub neighboring_fallback: usize,
    pub lost: usize,
    pub bot_victims_read: usize,
    pub silent: usize,
    pub ambiguous: usize,
    pub agree: usize,
    pub contradict: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadBotKill {
    pub event: KillFeedEvent,
    pub victim_index: i32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillFeedPairs {
    /// Original chronological feed with read packet identities attached.
    pub events: Vec<KillFeedEvent>,
    pub pairs: Vec<KillFeedEvent>,
    pub same_instant: Vec<KillFeedEvent>,
    pub read: Vec<KillFeedEvent>,
    pub neighboring_fallback: Vec<KillFeedEvent>,
    pub bot_read: Vec<ReadBotKill>,
    pub orphan_kills: Vec<KillFeedEvent>,
    pub orphan_deaths: Vec<KillFeedEvent>,
    pub stats: KillPairStats,
}
/// Reconstruct in input order; callers normally pass `build_kill_feed` output and
/// `scan_film_kill_events` records. Inferred roster names must not become pins.
pub fn resolve_kill_feed_pairs(
    feed: &KillFeed,
    recs: &[FilmKillEvent],
    roster: &KillRosterPins,
) -> KillFeedPairs {
    let mut out = KillFeedPairs {
        events: feed.events.clone(),
        ..Default::default()
    };
    let mut used = vec![false; recs.len()];
    let mut used_death = vec![false; out.events.len()];
    // Reserve same-instant couples before unmatched kills can consume their events.
    for e in &mut out.events {
        if e.killer.is_empty() || e.victim.is_empty() {
            continue;
        }
        if let Some(j) = recs.iter().enumerate().position(|(j, r)| {
            !used[j]
                && r.time_ms.abs_diff(e.time_ms) <= 2500
                && roster.pinned_name(r.fields.killer) == Some(e.killer.as_str())
                && roster.pinned_name(r.fields.victim) == Some(e.victim.as_str())
        }) {
            used[j] = true;
            e.packet = Some(recs[j].packet);
        }
    }
    for i in 0..out.events.len() {
        let e = out.events[i].clone();
        if e.killer.is_empty() {
            continue;
        }
        if !e.victim.is_empty() {
            out.pairs.push(e.clone());
            out.same_instant.push(e);
            used_death[i] = true;
            out.stats.same_instant += 1;
            continue;
        }
        let mut selected = None;
        let mut victim = -1;
        let mut ambiguous = false;
        for (j, r) in recs.iter().enumerate() {
            if used[j]
                || r.time_ms.abs_diff(e.time_ms) > 2500
                || roster.pinned_name(r.fields.killer) != Some(e.killer.as_str())
                || roster.pinned_name(r.fields.victim).is_none()
            {
                continue;
            }
            if selected.is_some() && r.fields.victim != victim {
                ambiguous = true;
                break;
            }
            victim = r.fields.victim;
            selected = Some(j);
        }
        if let Some(j) = selected.filter(|_| !ambiguous) {
            used[j] = true;
            let name = roster.pinned_name(victim).unwrap();
            let mut pair = KillFeedEvent {
                time_ms: e.time_ms,
                killer: e.killer,
                victim: name.into(),
                packet: Some(recs[j].packet),
                ..Default::default()
            };
            if roster.is_bot_index(victim) {
                out.stats.bot_victims_read += 1;
                out.bot_read.push(ReadBotKill {
                    event: pair,
                    victim_index: victim,
                });
                continue;
            }
            out.stats.read += 1;
            let neighbor = (i + 1..(i + 3).min(out.events.len())).find(|&v| {
                out.events[v].victim == name && out.events[v].killer.is_empty() && !used_death[v]
            });
            if let Some(v) = neighbor {
                used_death[v] = true;
                pair.victim_xuid = out.events[v].victim_xuid;
                out.stats.agree += 1;
            } else {
                pair.victim_xuid = feed.xuid_by_name.get(name).copied().unwrap_or(0);
                out.stats.contradict += 1;
            }
            out.pairs.push(pair.clone());
            out.read.push(pair);
            continue;
        }
        if ambiguous {
            out.stats.ambiguous += 1;
        } else {
            out.stats.silent += 1;
        }
        // Native fallback is the next two feed entries, with no time bound. Unlike
        // read-couple corroboration it deliberately does not test used_death.
        let neighbor = (i + 1..(i + 3).min(out.events.len()))
            .find(|&v| !out.events[v].victim.is_empty() && out.events[v].killer.is_empty());
        if let Some(v) = neighbor {
            let pair = KillFeedEvent {
                time_ms: e.time_ms,
                killer: e.killer,
                victim: out.events[v].victim.clone(),
                victim_xuid: out.events[v].victim_xuid,
                ..Default::default()
            };
            out.pairs.push(pair.clone());
            out.neighboring_fallback.push(pair);
            used_death[v] = true;
            out.stats.neighboring_fallback += 1;
        } else {
            out.orphan_kills.push(e);
            out.stats.lost += 1;
        }
    }
    out.orphan_deaths = out
        .events
        .iter()
        .enumerate()
        .filter(|(i, e)| !e.victim.is_empty() && !used_death[*i])
        .map(|(_, e)| e.clone())
        .collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Row {
        feed: KillFeed,
        recs: Vec<FilmKillEvent>,
        roster: KillRosterPins,
        expected: KillFeedPairs,
    }
    #[test]
    fn native_kill_feed_pairs_oracle() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/kill-feed-pairs-v41.json.zlib")[..],
        )
        .read_to_end(&mut json)
        .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&json).unwrap();
        assert_eq!(rows.len(), 2048);
        let (mut read, mut bot, mut ambiguous, mut fallback) = (0, 0, 0, 0);
        for (i, row) in rows.into_iter().enumerate() {
            let actual = resolve_kill_feed_pairs(&row.feed, &row.recs, &row.roster);
            assert_eq!(actual, row.expected, "pairs {i}");
            read += actual.stats.read;
            bot += actual.stats.bot_victims_read;
            ambiguous += actual.stats.ambiguous;
            fallback += actual.stats.neighboring_fallback;
        }
        assert!(read > 0 && bot > 0 && ambiguous > 0 && fallback > 0);
    }
}
