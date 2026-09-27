//! LegacyFilm-facing carried abilities, labels, impulse episodes and charge readings.
use super::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, num::NonZeroU64};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityCatalog {
    pub palettes: Vec<ReplayAbilityPalette>,
    pub impulse_families: BTreeSet<String>,
    pub charge_families: BTreeSet<String>,
}
/// The pinned title's production-loaded names, HUD URLs and measured families.
pub fn replay_ability_catalog() -> ReplayAbilityCatalog {
    serde_json::from_str(include_str!("reference/ability_catalog.json"))
        .expect("pinned ability catalog is valid")
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmReplayAbilities {
    pub abilities: ReplayAbilityPublication,
    pub impulses: Vec<ReplayAbilityImpulse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impulse_coverage: Option<ReplayAbilityImpulseCoverage>,
    pub charges: Vec<ReplayAbilityCharge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charge_coverage: Option<ReplayAbilityChargeCoverage>,
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_abilities(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> Option<FilmReplayAbilities> {
    build_film_replay_abilities_with_catalog(film, players, &replay_ability_catalog())
}
/// Classify the palette from published ability reads, then attribute charges and
/// impulses with raw rank readings and the player registry's life boundaries.
/// Coverage is absent when its underlying scanner did not run.
#[allow(dead_code)]
pub(crate) fn build_film_replay_abilities_with_catalog(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    catalog: &ReplayAbilityCatalog,
) -> Option<FilmReplayAbilities> {
    let step = NonZeroU64::new(players.clock.step_us)?;
    let ranks: Vec<_> = film
        .biped_channels
        .as_ref()
        .map(|c| c.ability_ranks().collect())
        .unwrap_or_default();
    let inventory = film
        .keyframe_inventory
        .as_ref()
        .filter(|_| film.keyframe_inventory_error.is_none())
        .map_or(&[][..], |s| s.records.as_slice());
    let reads = build_replay_ability_reads(&ranks, inventory, players.clock.origin_us, step);
    let published = players
        .players
        .publication
        .tracks
        .iter()
        .map(|t| t.slot)
        .collect();
    let abilities = publish_replay_abilities(reads, &published, &catalog.palettes);
    let palette = abilities.palette.as_ref().map(|p| &p.families);
    let lives = players.registry.owners.state.lives();
    let context = |measured_families| ReplayAbilityContext {
        ranks: &ranks,
        lives,
        palette,
        measured_families,
        published_slots: &published,
        origin_us: players.clock.origin_us,
        step_us: players.clock.step_us,
    };
    let empty_charge_stats = AbilityChargeStats::default();
    let (reads, stats) = film
        .ability_charges
        .as_ref()
        .map_or((&[][..], &empty_charge_stats), |s| {
            (s.records.as_slice(), &s.stats)
        });
    let charges = build_replay_ability_charges(reads, stats, context(&catalog.charge_families));
    let charge_coverage = stats.scanned.then_some(charges.coverage);
    let empty_impulse_stats = AbilityImpulseStats::default();
    let (reads, stats) = film
        .ability_states
        .as_ref()
        .map_or((&[][..], &empty_impulse_stats), |s| {
            (s.impulses.as_slice(), &s.impulse_stats)
        });
    let impulses = build_replay_ability_impulses(reads, stats, context(&catalog.impulse_families));
    let impulse_coverage = stats.scanned.then_some(impulses.coverage);
    Some(FilmReplayAbilities {
        abilities,
        impulses: impulses.impulses,
        impulse_coverage,
        charges: charges.charges,
        charge_coverage,
    })
}

/// Preserve the native scan gates and impulse-before-charge observation order.
pub(crate) fn log_replay_ability_scans(
    impulses: Option<&ReplayAbilityImpulseCoverage>,
    impulse_reads: usize,
    charges: Option<&ReplayAbilityChargeCoverage>,
    charge_reads: usize,
) {
    if let Some(coverage) = impulses {
        coverage.log();
    } else {
        tracing::warn!(
            lectures = impulse_reads,
            "rejeu : impulsions de capacite NON BALAYEES — aucune couverture publiee"
        );
    }
    if let Some(coverage) = charges {
        coverage.log();
    } else {
        tracing::warn!(
            lectures = charge_reads,
            "rejeu : charges d equipement NON BALAYEES — aucune couverture publiee"
        );
    }
}
