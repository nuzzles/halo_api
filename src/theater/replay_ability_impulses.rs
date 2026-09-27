//! Fold retransmitted impulse reads and attribute each episode to its equipment.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityImpulse {
    pub t: i64,
    pub slot: u32,
    pub family: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityImpulseEpisode {
    pub slot: u32,
    pub timestamp_us: u64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityImpulseScanCoverage {
    pub records: i64,
    #[serde(rename = "withI57")]
    pub with_predicted: i64,
    #[serde(rename = "withI59")]
    pub with_non_predicted: i64,
    pub read: i64,
    pub unread: i64,
    #[serde(rename = "tag1")]
    pub tag_one: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityImpulseCoverage {
    #[serde(flatten)]
    pub attribution: ReplayAbilityChargeCoverage,
    pub episodes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan: Option<ReplayAbilityImpulseScanCoverage>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityImpulsePublication {
    pub impulses: Vec<ReplayAbilityImpulse>,
    pub coverage: ReplayAbilityImpulseCoverage,
}
/// Gaps of at most one second extend the current gesture from its last reading,
/// not from its first. Return the first reading, sorted by time then slot.
pub fn fold_replay_ability_impulses(reads: &[AbilityImpulse]) -> Vec<ReplayAbilityImpulseEpisode> {
    fold_impulse_values(reads.iter().map(|r| (r.slot, r.source.timestamp_us)))
}
pub fn fold_facts_replay_ability_impulses(
    reads: &[FactsAbilityImpulse],
) -> Vec<ReplayAbilityImpulseEpisode> {
    fold_impulse_values(reads.iter().map(|r| (r.slot, r.timestamp_us)))
}
fn fold_impulse_values(
    reads: impl Iterator<Item = (u32, u64)>,
) -> Vec<ReplayAbilityImpulseEpisode> {
    let mut ordered: Vec<_> = reads.collect();
    ordered.sort_by_key(|&(slot, time)| (slot, time));
    let mut last = BTreeMap::new();
    let mut out = Vec::new();
    for (slot, time) in ordered {
        if last
            .insert(slot, time)
            .is_some_and(|previous| time - previous <= 1_000_000)
        {
            continue;
        }
        out.push(ReplayAbilityImpulseEpisode {
            slot,
            timestamp_us: time,
        });
    }
    out.sort_by_key(|e| (e.timestamp_us, e.slot));
    out
}
pub fn build_replay_ability_impulses(
    reads: &[AbilityImpulse],
    stats: &AbilityImpulseStats,
    context: ReplayAbilityContext<'_>,
) -> ReplayAbilityImpulsePublication {
    let index = ReplayAbilityRankIndex::new(context.ranks, context.lives);
    let scan = stats.scanned.then_some(ReplayAbilityImpulseScanCoverage {
        records: stats.records as i64,
        with_predicted: stats.with_predicted as i64,
        with_non_predicted: stats.with_non_predicted as i64,
        read: stats.read as i64,
        unread: stats.unread as i64,
        tag_one: stats.tag_one as i64,
    });
    build_impulse_values(
        reads.iter().map(|r| (r.slot, r.source.timestamp_us)),
        stats.absent,
        scan,
        context.with_index(index),
    )
}
pub fn build_facts_replay_ability_impulses(
    reads: &[FactsAbilityImpulse],
    stats: &FactsAbilityImpulseStats,
    context: FactsReplayAbilityContext<'_>,
) -> ReplayAbilityImpulsePublication {
    let index = ReplayAbilityRankIndex::from_facts(context.ranks, context.lives);
    let scan = stats.scanned.then_some(ReplayAbilityImpulseScanCoverage {
        records: stats.records,
        with_predicted: stats.with_i57,
        with_non_predicted: stats.with_i59,
        read: stats.read,
        unread: stats.unread,
        tag_one: stats.tag1,
    });
    build_impulse_values(
        reads.iter().map(|r| (r.slot, r.timestamp_us)),
        stats.absent,
        scan,
        context.with_index(index),
    )
}
fn build_impulse_values(
    reads: impl ExactSizeIterator<Item = (u32, u64)>,
    absent: bool,
    scan: Option<ReplayAbilityImpulseScanCoverage>,
    context: super::replay_ability_charges::AbilityResolutionContext<'_>,
) -> ReplayAbilityImpulsePublication {
    let mut out = ReplayAbilityImpulsePublication::default();
    out.coverage.attribution.reads = reads.len();
    out.coverage.attribution.component_absent = absent;
    out.coverage.scan = scan;
    if reads.len() == 0 || context.step_us == 0 {
        return out;
    }
    let index = &context.index;
    let resolvable =
        context.palette.is_some() && !context.measured_families.is_empty() && context.has_lives;
    for episode in fold_impulse_values(reads) {
        out.coverage.episodes += 1;
        let coverage = &mut out.coverage.attribution;
        if episode.timestamp_us < context.origin_us {
            coverage.before_origin += 1;
            continue;
        }
        if !context.published_slots.contains(&episode.slot) {
            coverage.unpublished += 1;
            continue;
        }
        if !resolvable {
            coverage.no_resolver += 1;
            continue;
        }
        let Some(rank) = index.rank_in_life(episode.slot, episode.timestamp_us) else {
            coverage.no_identity += 1;
            continue;
        };
        let family = context
            .palette
            .and_then(|p| p.get(&rank))
            .map_or("", String::as_str);
        if family.is_empty() || !context.measured_families.contains(family) {
            coverage.other_family += 1;
            continue;
        }
        out.impulses.push(ReplayAbilityImpulse {
            t: ((episode.timestamp_us - context.origin_us) / context.step_us) as i64,
            slot: episode.slot,
            family: family.into(),
        });
    }
    out.coverage.attribution.published = out.impulses.len();
    out
}

impl ReplayAbilityImpulseCoverage {
    /// Emit the pinned reference parser's coverage observation.
    pub fn log(&self) {
        tracing::info!(
            lectures = self.attribution.reads,
            episodes = self.episodes,
            publiees = self.attribution.published,
            sansIdentite = self.attribution.no_identity,
            familleNonMesuree = self.attribution.other_family,
            attributionIndisponible = self.attribution.no_resolver,
            avantOrigine = self.attribution.before_origin,
            sansPiste = self.attribution.unpublished,
            composantAbsent = self.attribution.component_absent,
            "rejeu : impulsions de capacite"
        );
    }
}
