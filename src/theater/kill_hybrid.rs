//! Native ordered walk/scan attribution, preserving feed credit and fatal source.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillSourceProvenance {
    pub path: KillReadPath,
    pub origin: String,
    pub multiplicity: usize,
}
impl std::fmt::Display for KillSourceProvenance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = match self.path {
            KillReadPath::Walk => "marche",
            KillReadPath::Scan => "scan",
        };
        write!(f, "{} / {path}", self.origin)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributedFilmKill {
    pub time_ms: i64,
    pub victim: String,
    pub killer: String,
    pub feed_present: bool,
    pub source: KillSourceTruth,
    pub diverges: bool,
    pub read: KillSourceProvenance,
    pub packet: Option<KillPacketIdentity>,
    pub assist: KillAssist,
    pub killer_damage: KillDamageShare,
    pub assist_damage: KillDamageShare,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnclaimedFilmDeath {
    pub time_ms: i64,
    pub victim: String,
    pub victim_xuid: u64,
    pub source: KillSourceTruth,
    pub read: KillSourceProvenance,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillPathStats {
    pub population: usize,
    pub matched: usize,
    pub published: usize,
}
impl KillPathStats {
    pub fn ratio(&self) -> f64 {
        if self.population == 0 {
            0.0
        } else {
            self.matched as f64 / self.population as f64
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillMatchingStats {
    pub identity: usize,
    pub window: usize,
    pub bot_window: usize,
    pub unclaimed_window: usize,
    pub pairs_without_identity: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillHybridResult {
    pub kills: Vec<AttributedFilmKill>,
    pub unclaimed: Vec<UnclaimedFilmDeath>,
    pub walk: KillPathStats,
    pub scan: KillPathStats,
    pub self_walk: KillPathStats,
    pub self_scan: KillPathStats,
    pub bot: KillPathStats,
    pub bot_killer: KillPathStats,
    pub unclaimed_stats: KillPathStats,
    pub matching: KillMatchingStats,
    pub redundant: usize,
    pub no_bit: usize,
    pub agree: usize,
    pub disagree: usize,
    pub multi_candidate: usize,
    pub unexplained_pair: usize,
    pub unexplained_self: usize,
    pub unexplained_bot: usize,
}
fn position(c: &KillSourceCandidate) -> (i64, i64, i64) {
    let (chunk, packet) = c.packet.map(|p| (p.chunk, p.packet)).unwrap_or((0, 0));
    (chunk, packet, c.bit)
}
fn note(stats: &mut KillMatchingStats, fallback: bool, kind: u8) {
    if !fallback {
        stats.identity += 1;
    } else {
        match kind {
            1 => stats.bot_window += 1,
            2 => stats.unclaimed_window += 1,
            _ => stats.window += 1,
        }
    }
}
/// Sort walked candidates by chunk, packet and bit before preferring their evidence.
/// Publication gates and assist attachment run after this pass.
pub fn attribute_kill_sources(
    ctx: &KillMatchingContext<'_>,
    walk: &[KillSourceCandidate],
    no_bit: usize,
    self_source: bool,
    multiplicity_max: usize,
    strong_tag_required: bool,
) -> KillHybridResult {
    let mut walk = walk.to_vec();
    native_sort::sort_by(&mut walk, |a, b| position(a).cmp(&position(b)));
    let walk = walk.as_slice();
    let (all, redundant, counts) = combine_kill_candidates(walk, ctx.scan);
    let mut out = KillHybridResult {
        redundant,
        no_bit,
        ..Default::default()
    };
    let mut by_time = BTreeMap::new();
    let mut bot_used = BTreeSet::new();
    let provenance = |c: &SourcedKillCandidate, origin: &str| KillSourceProvenance {
        path: c.path,
        origin: origin.into(),
        multiplicity: kill_candidate_multiplicity(&counts, &c.candidate),
    };
    let build = |e: &KillFeedEvent,
                 c: &SourcedKillCandidate,
                 origin: &str,
                 present: bool,
                 diverges: bool| AttributedFilmKill {
        time_ms: e.time_ms,
        victim: e.victim.clone(),
        killer: e.killer.clone(),
        feed_present: present,
        source: kill_source_truth(c.candidate.tag, c.candidate.category),
        diverges,
        read: provenance(c, origin),
        packet: c.candidate.packet,
        assist: KillAssist::default(),
        killer_damage: KillDamageShare::default(),
        assist_damage: KillDamageShare::default(),
    };
    for c in &all {
        if c.candidate.victim == c.candidate.killer || ctx.is_bot_side(&c.candidate) {
            continue;
        }
        let st = if c.path == KillReadPath::Walk {
            &mut out.walk
        } else {
            &mut out.scan
        };
        st.population += 1;
        let Some((e, f)) = ctx.match_feed(&c.candidate, false) else {
            out.unexplained_pair += 1;
            continue;
        };
        st.matched += 1;
        if by_time.contains_key(&e.time_ms) {
            continue;
        }
        st.published += 1;
        note(&mut out.matching, f, 0);
        by_time.insert(e.time_ms, build(e, c, "credit-concordant", true, false));
    }
    if self_source {
        for c in &all {
            if c.candidate.victim != c.candidate.killer {
                continue;
            }
            let st = if c.path == KillReadPath::Walk {
                &mut out.self_walk
            } else {
                &mut out.self_scan
            };
            st.population += 1;
            let matched = ctx.match_feed(&c.candidate, true).filter(|_| {
                kill_self_source_allowed(
                    &c.candidate,
                    &counts,
                    multiplicity_max,
                    strong_tag_required,
                )
            });
            let Some((e, f)) = matched else {
                out.unexplained_self += 1;
                continue;
            };
            st.matched += 1;
            if by_time.contains_key(&e.time_ms) {
                out.unexplained_self += 1;
                continue;
            }
            st.published += 1;
            note(&mut out.matching, f, 0);
            by_time.insert(e.time_ms, build(e, c, "source-victime", true, true));
        }
    }
    for m in ctx.bot_deaths() {
        out.bot.population += 1;
        let Some(c) = m.candidate else {
            continue;
        };
        out.bot.matched += 1;
        if !bot_used.insert(position(&c.candidate)) {
            continue;
        }
        out.bot.published += 1;
        note(&mut out.matching, m.window_fallback, 1);
        let e = KillFeedEvent {
            victim: ctx.roster.name_of(c.candidate.victim).into(),
            ..m.event
        };
        by_time.insert(e.time_ms, build(&e, &c, "bot", false, false));
    }
    for m in ctx.bot_killer_deaths(&all) {
        out.bot_killer.population += 1;
        let Some(c) = m.candidate else {
            continue;
        };
        out.bot_killer.matched += 1;
        if by_time.contains_key(&m.event.time_ms) || !bot_used.insert(position(&c.candidate)) {
            continue;
        }
        out.bot_killer.published += 1;
        note(&mut out.matching, m.window_fallback, 1);
        let e = KillFeedEvent {
            killer: ctx.roster.name_of(c.candidate.killer).into(),
            ..m.event
        };
        by_time.insert(e.time_ms, build(&e, &c, "tueur-bot", true, false));
    }
    let mut used = BTreeSet::new();
    for e in &ctx.feed.orphan_deaths {
        out.unclaimed_stats.population += 1;
        if by_time.contains_key(&e.time_ms) {
            continue;
        }
        let Some((c, f)) = ctx.unclaimed(e, &all) else {
            continue;
        };
        out.unclaimed_stats.matched += 1;
        if !used.insert(position(&c.candidate)) {
            continue;
        }
        out.unclaimed_stats.published += 1;
        note(&mut out.matching, f, 2);
        out.unclaimed.push(UnclaimedFilmDeath {
            time_ms: e.time_ms,
            victim: e.victim.clone(),
            victim_xuid: e.victim_xuid,
            source: kill_source_truth(c.candidate.tag, c.candidate.category),
            read: provenance(c, "sans-revendication"),
        });
    }
    native_sort::sort_by(&mut out.unclaimed, |a, b| a.time_ms.cmp(&b.time_ms));
    out.unexplained_bot = ctx
        .scan
        .iter()
        .filter(|c| ctx.is_bot_side(c) && !bot_used.contains(&position(c)))
        .count();
    let mut instants = BTreeMap::<
        i64,
        (
            usize,
            Option<&KillSourceCandidate>,
            Option<&KillSourceCandidate>,
        ),
    >::new();
    for (cs, path) in [(walk, KillReadPath::Walk), (ctx.scan, KillReadPath::Scan)] {
        for c in cs {
            if c.victim == c.killer {
                continue;
            }
            let Some((e, _)) = ctx.match_feed(c, false) else {
                continue;
            };
            let b = instants.entry(e.time_ms).or_default();
            b.0 += 1;
            if path == KillReadPath::Walk {
                b.1 = Some(c);
            } else {
                b.2 = Some(c);
            }
        }
    }
    for (n, w, s) in instants.into_values() {
        if n > 2 || (n == 2 && (w.is_none() || s.is_none())) {
            out.multi_candidate += 1;
        }
        if let (Some(w), Some(s)) = (w, s) {
            if w.tag == s.tag && w.category == s.category {
                out.agree += 1;
            } else {
                out.disagree += 1;
            }
        }
    }
    out.matching.pairs_without_identity =
        ctx.feed.pairs.iter().filter(|e| e.packet.is_none()).count();
    out.kills = by_time.into_values().collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_kill_provenance_display() {
        let cases: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("fixtures/kill-provenance-v41.json")).unwrap();
        assert_eq!(cases.len(), 48);
        for (i, case) in cases.into_iter().enumerate() {
            let provenance: KillSourceProvenance = serde_json::from_value(case.clone()).unwrap();
            assert_eq!(provenance.to_string(), case["display"], "provenance {i}");
        }
    }
}
