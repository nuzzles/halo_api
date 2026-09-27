//! Native kill-source health gates and catalog diagnostics.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
/// Pinned native health thresholds. These assess decoder coverage, not an
/// individual recorded action's certainty.
pub const KILL_SOURCE_UNEXPLAINED_WARN_RATIO: f64 = 0.180;
pub const KILL_SOURCE_UNEXPLAINED_ALERT_RATIO: f64 = 0.360;
pub const KILL_SOURCE_COVERAGE_WARN_RATIO: f64 = 1.00;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillSourceHealth {
    pub film: String,
    pub candidates: usize,
    pub published: usize,
    pub unexplained_pair: usize,
    pub unexplained_self: usize,
    pub unexplained_bot: usize,
    pub out_of_roster: usize,
    pub tag_out_of_catalogue_walk: usize,
    pub tag_out_of_catalogue_scan: usize,
    pub deaths_real: i64,
    pub deaths_covered: usize,
}
impl KillSourceHealth {
    pub fn unexplained_total(&self) -> usize {
        self.unexplained_pair + self.unexplained_self + self.unexplained_bot
    }
    pub fn unexplained_ratio(&self) -> f64 {
        if self.candidates == 0 {
            0.0
        } else {
            self.unexplained_total() as f64 / self.candidates as f64
        }
    }
    pub fn coverage_ratio(&self) -> f64 {
        if self.deaths_real <= 0 {
            0.0
        } else {
            self.deaths_covered as f64 / self.deaths_real as f64
        }
    }
    pub fn alerts(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.tag_out_of_catalogue_walk > 0 {
            out.push(format!("{} enregistrement(s) de la marche portent un tag HORS catalogue : la table `jpt!` est PERIMEE, regenerer (recette : paquet damagetag)",self.tag_out_of_catalogue_walk));
        }
        if self.unexplained_ratio() > KILL_SOURCE_UNEXPLAINED_ALERT_RATIO {
            out.push(format!(
                "taux d'inexpliques {:.1}% au-dela du DOUBLE du maximum observe (18.0%)",
                100.0 * self.unexplained_ratio()
            ));
        }
        out
    }
    pub fn degradations(&self) -> Vec<String> {
        if self.out_of_roster == 0 {
            Vec::new()
        } else {
            vec![format!(
                "{} dead-state(s) a tag `jpt!` valide portent un indice hors du roster retenu : un participant n'est pas compte (remplacant que les trois lectures d'identite ne nomment pas ? bot non declare ?). CES MORTS-LA SONT REFUSEES, les autres publient",
                self.out_of_roster
            )]
        }
    }
    pub fn verdict(&self) -> &'static str {
        if !self.alerts().is_empty() {
            "ALERTE"
        } else if !self.degradations().is_empty()
            || self.unexplained_ratio() > KILL_SOURCE_UNEXPLAINED_WARN_RATIO
            || self.coverage_ratio() < KILL_SOURCE_COVERAGE_WARN_RATIO
        {
            "HORS DOMAINE MESURE"
        } else {
            "NOMINAL"
        }
    }
    pub fn line_by_line_publishable(&self, margin: i64, determined: bool) -> bool {
        self.alerts().is_empty() && (margin > 0 || determined)
    }
    pub fn metric_pairs(&self) -> Vec<(&'static str, i64)> {
        vec![
            ("killsource_candidates_total", self.candidates as i64),
            ("killsource_published_total", self.published as i64),
            ("killsource_unexplained_pair", self.unexplained_pair as i64),
            ("killsource_unexplained_self", self.unexplained_self as i64),
            ("killsource_unexplained_botidx", self.unexplained_bot as i64),
            ("killsource_out_of_roster", self.out_of_roster as i64),
            (
                "killsource_tag_out_of_catalogue_walk",
                self.tag_out_of_catalogue_walk as i64,
            ),
            (
                "killsource_tag_out_of_catalogue_scan",
                self.tag_out_of_catalogue_scan as i64,
            ),
            ("killsource_deaths_real", self.deaths_real),
            ("killsource_deaths_covered", self.deaths_covered as i64),
        ]
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillSourceCoverage {
    pub covered: usize,
    pub real_pairs: i64,
    pub reconstructed_pairs: usize,
    pub ghost_pairs: usize,
    pub same_instant_pairs: usize,
    pub feed_kills: usize,
    pub feed_deaths: usize,
    pub bot_deaths: usize,
    pub bot_killer_deaths: usize,
}
pub fn kill_source_coverage(
    ctx: &KillMatchingContext<'_>,
    feed: &KillFeed,
    hybrid: &KillHybridResult,
) -> KillSourceCoverage {
    let ghosts = ctx.ghost_pairs().len();
    KillSourceCoverage {
        covered: hybrid
            .kills
            .iter()
            .filter(|k| !matches!(k.read.origin.as_str(), "bot" | "tueur-bot"))
            .count(),
        real_pairs: ctx.feed.pairs.len() as i64 - ghosts as i64,
        reconstructed_pairs: ctx.feed.pairs.len(),
        ghost_pairs: ghosts,
        same_instant_pairs: ctx.feed.same_instant.len(),
        feed_kills: feed.kill_count,
        feed_deaths: feed.death_count,
        bot_deaths: hybrid.bot.published,
        bot_killer_deaths: hybrid.bot_killer.published,
    }
}
pub fn kill_walk_out_of_catalogue(
    ctx: &KillMatchingContext<'_>,
    walk: &[KillSourceCandidate],
) -> (usize, Vec<u32>) {
    let mut n = 0;
    let mut tags = BTreeSet::new();
    for c in walk {
        if !pinned_kill_damage_catalog().is_damage_effect(c.tag)
            && ctx.match_feed(c, false).is_some()
        {
            n += 1;
            tags.insert(c.tag);
        }
    }
    (n, tags.into_iter().collect())
}
pub fn measure_kill_source_health(
    name: &str,
    ctx: &KillMatchingContext<'_>,
    walk: &KillWalkResult,
    hybrid: &KillHybridResult,
    coverage: &KillSourceCoverage,
) -> KillSourceHealth {
    let (candidates, _) = walk.candidates();
    KillSourceHealth {
        film: name.into(),
        candidates: candidates.len() + ctx.scan.len() - hybrid.redundant,
        published: hybrid.kills.len(),
        unexplained_pair: hybrid.unexplained_pair,
        unexplained_self: hybrid.unexplained_self,
        unexplained_bot: hybrid.unexplained_bot,
        out_of_roster: walk
            .deaths
            .iter()
            .filter(|d| {
                i64::from(d.slot) >= walk.biped_low
                    && i64::from(d.slot) <= walk.biped_high
                    && (i64::from(d.dead.enum_a) >= ctx.roster.player_count as i64
                        || i64::from(d.dead.enum_b) >= ctx.roster.player_count as i64)
                    && pinned_kill_damage_catalog().is_damage_effect(d.dead.src_tag0)
            })
            .count(),
        tag_out_of_catalogue_walk: kill_walk_out_of_catalogue(ctx, &candidates).0,
        tag_out_of_catalogue_scan: 0,
        deaths_real: coverage.real_pairs,
        deaths_covered: coverage.covered,
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillRelaxedProbe {
    pub candidates: usize,
    pub out_of_catalogue: usize,
    pub paired: usize,
    pub uncovered: usize,
    pub tags: Vec<u32>,
}
/// Probe undeduplicated relaxed candidates; matching remains fully constrained.
pub fn probe_relaxed_kill_candidates(
    ctx: &KillMatchingContext<'_>,
    candidates: &[KillSourceCandidate],
    covered: &BTreeSet<i64>,
) -> KillRelaxedProbe {
    let mut out = KillRelaxedProbe::default();
    let mut tags = BTreeSet::new();
    for c in candidates {
        out.candidates += 1;
        if pinned_kill_damage_catalog().is_damage_effect(c.tag) {
            continue;
        }
        out.out_of_catalogue += 1;
        let Some((e, _)) = ctx.match_feed(c, false) else {
            continue;
        };
        out.paired += 1;
        if covered.contains(&e.time_ms) {
            continue;
        }
        out.uncovered += 1;
        tags.insert(c.tag);
    }
    out.tags = tags.into_iter().collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_kill_health_methods() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/kill-health-methods-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 2800);
        let mut verdicts = std::collections::BTreeSet::new();
        for (i, row) in rows.iter().enumerate() {
            let health: KillSourceHealth = serde_json::from_value(row["input"].clone()).unwrap();
            assert_eq!(
                health.unexplained_total(),
                row["total"].as_u64().unwrap() as usize,
                "total {i}"
            );
            assert_eq!(
                health.unexplained_ratio().to_bits(),
                row["unexplained_ratio"].as_f64().unwrap().to_bits(),
                "ratio {i}"
            );
            assert_eq!(
                health.coverage_ratio().to_bits(),
                row["coverage_ratio"].as_f64().unwrap().to_bits(),
                "coverage {i}"
            );
            for (name, actual) in [
                ("alerts", health.alerts()),
                ("degradations", health.degradations()),
            ] {
                let expected: Vec<String> = if row[name].is_null() {
                    Vec::new()
                } else {
                    serde_json::from_value(row[name].clone()).unwrap()
                };
                assert_eq!(actual, expected, "{name} {i}");
            }
            assert_eq!(
                health.verdict(),
                row["verdict"].as_str().unwrap(),
                "verdict {i}"
            );
            verdicts.insert(health.verdict());
            assert_eq!(
                serde_json::to_value(health.metric_pairs()).unwrap(),
                row["metrics"],
                "ordered metrics {i}"
            );
            for gate in row["gates"].as_array().unwrap() {
                assert_eq!(
                    health.line_by_line_publishable(
                        gate["margin"].as_i64().unwrap(),
                        gate["determined"].as_bool().unwrap()
                    ),
                    gate["publishable"].as_bool().unwrap(),
                    "publication {i}"
                );
            }
            let path = KillPathStats {
                population: health.candidates,
                matched: health.unexplained_total(),
                published: health.published,
            };
            assert_eq!(
                path.ratio().to_bits(),
                row["path_ratio"].as_f64().unwrap().to_bits(),
                "path ratio {i}"
            );
        }
        assert_eq!(
            verdicts,
            ["ALERTE", "HORS DOMAINE MESURE", "NOMINAL"].into()
        );
    }
}
