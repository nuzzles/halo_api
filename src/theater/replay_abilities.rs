//! Publish carried ability ranks, classify their palette, and retain used labels.
use super::{BipedAbilityEmission, KeyframeInventory};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};
fn is_false(v: &bool) -> bool {
    !v
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLabel {
    pub en: String,
    pub fr: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub img: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub tinted: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub family: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityPalette {
    pub id: String,
    pub markers: Vec<i64>,
    pub ranks: BTreeMap<i64, ReplayLabel>,
    pub families: BTreeMap<i64, String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityRead {
    pub t: i64,
    pub slot: u32,
    pub r: i64,
    pub src: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayAbilityCoverage {
    pub reads: usize,
    pub scan_noise: usize,
    pub unpublished: usize,
    pub published: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityPublication {
    pub reads: Vec<ReplayAbilityRead>,
    pub coverage: ReplayAbilityCoverage,
    pub palette: Option<ReplayAbilityPalette>,
    pub labels: BTreeMap<String, ReplayLabel>,
}
/// Preserve both channels without deduplication. Stable frame/slot/source ordering
/// puts i48 ahead of keyframes on ties; readings before the origin are omitted.
pub fn build_replay_ability_reads(
    ranks: &[BipedAbilityEmission],
    inventory: &[KeyframeInventory],
    origin_us: u64,
    step: NonZeroU64,
) -> Vec<ReplayAbilityRead> {
    build_ability_values(
        ranks.iter().filter_map(|r| {
            r.rank
                .map(|rank| (r.source.timestamp_us, r.slot, i64::from(rank)))
        }),
        inventory
            .iter()
            .map(|r| (r.timestamp_us, r.slot, i64::from(r.ability_rank))),
        origin_us,
        step,
    )
}
/// Publish cached ability ranks without narrowing the native signed domain.
pub fn build_facts_replay_ability_reads(
    ranks: &[super::FactsAbilityRank],
    inventory: &[super::FactsKeyframeInventory],
    origin_us: u64,
    step: NonZeroU64,
) -> Vec<ReplayAbilityRead> {
    build_ability_values(
        ranks.iter().map(|r| (r.timestamp_us, r.slot, r.rank)),
        inventory
            .iter()
            .map(|r| (r.timestamp_us, r.slot, r.ability_rank)),
        origin_us,
        step,
    )
}
fn build_ability_values(
    ranks: impl Iterator<Item = (u64, u32, i64)>,
    inventory: impl Iterator<Item = (u64, u32, i64)>,
    origin_us: u64,
    step: NonZeroU64,
) -> Vec<ReplayAbilityRead> {
    let mut out = Vec::new();
    for (time, slot, rank) in ranks {
        if time >= origin_us {
            out.push(ReplayAbilityRead {
                t: ((time - origin_us) / step.get()) as i64,
                slot,
                r: rank,
                src: "i48".into(),
            });
        }
    }
    for (time, slot, rank) in inventory {
        if time >= origin_us && rank >= 0 {
            out.push(ReplayAbilityRead {
                t: ((time - origin_us) / step.get()) as i64,
                slot,
                r: rank,
                src: "kf".into(),
            });
        }
    }
    out.sort_by(|a, b| (a.t, a.slot, &a.src).cmp(&(b.t, b.slot, &b.src)));
    out
}
/// Each reading votes for the first palette containing its marker. Fewer than ten
/// reads require unanimity; otherwise the first palette with >=90% wins.
pub fn classify_replay_ability_palette<'a>(
    reads: &[ReplayAbilityRead],
    palettes: &'a [ReplayAbilityPalette],
) -> Option<&'a ReplayAbilityPalette> {
    if reads.is_empty() {
        return None;
    }
    let mut counts = vec![0usize; palettes.len()];
    for read in reads {
        if let Some(index) = palettes.iter().position(|p| p.markers.contains(&read.r)) {
            counts[index] += 1;
        }
    }
    let threshold = if reads.len() < 10 { 1. } else { 0.9 };
    palettes
        .iter()
        .zip(counts)
        .find(|(_, n)| *n as f64 / reads.len() as f64 >= threshold)
        .map(|(p, _)| p)
}
pub fn replay_ability_labels_used(
    reads: &[ReplayAbilityRead],
    palette: Option<&ReplayAbilityPalette>,
) -> BTreeMap<String, ReplayLabel> {
    let mut out = BTreeMap::new();
    if let Some(palette) = palette {
        for read in reads {
            if let Some(label) = palette.ranks.get(&read.r) {
                let mut label = label.clone();
                label.family = palette.families.get(&read.r).cloned().unwrap_or_default();
                out.insert(read.r.to_string(), label);
            }
        }
    }
    out
}
/// Reject ranks above 27 before the published-slot filter, then classify only the
/// surviving readings. Pre-origin readings are not part of the coverage denominator.
pub fn publish_replay_abilities(
    mut reads: Vec<ReplayAbilityRead>,
    published_slots: &BTreeSet<u32>,
    palettes: &[ReplayAbilityPalette],
) -> ReplayAbilityPublication {
    let mut coverage = ReplayAbilityCoverage {
        reads: reads.len(),
        ..Default::default()
    };
    reads.retain(|r| {
        if r.r > 27 {
            coverage.scan_noise += 1;
            false
        } else {
            true
        }
    });
    reads.retain(|r| {
        if !published_slots.contains(&r.slot) {
            coverage.unpublished += 1;
            false
        } else {
            true
        }
    });
    coverage.published = reads.len();
    let palette = classify_replay_ability_palette(&reads, palettes).cloned();
    let labels = replay_ability_labels_used(&reads, palette.as_ref());
    ReplayAbilityPublication {
        reads,
        coverage,
        palette,
        labels,
    }
}

impl ReplayAbilityCoverage {
    /// Emit the pinned reference parser's coverage observation.
    pub fn log(&self) {
        tracing::info!(
            lectures = self.reads,
            bruitDeBalayage = self.scan_noise,
            sansTrajectoirePubliee = self.unpublished,
            publiees = self.published,
            "rejeu : identite de capacite portee"
        );
    }
}

/// Report the selected palette, including an explicitly unclassified film.
pub(crate) fn log_replay_ability_palette(
    palette: Option<&ReplayAbilityPalette>,
    reads: usize,
    labels: usize,
) {
    tracing::info!(
        "a.palette" = palette.map_or("non classee", |p| p.id.as_str()),
        lectures = reads,
        rangsNommes = labels,
        "rejeu : a.palette de capacites"
    );
}
