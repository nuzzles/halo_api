//! Native ability palette publication followed by its dependent action passes.
use super::*;
use std::{collections::BTreeSet, num::NonZeroU64};

pub struct FactsReplayAbilityDocumentInput<'a> {
    pub labels: &'a ReplayLabelCatalog,
    pub ranks: &'a [FactsAbilityRank],
    pub inventory: &'a [FactsKeyframeInventory],
    pub equipment_changes: &'a [FactsEquipmentChange],
    pub equipment_stats: &'a FactsEquipmentChangeStats,
    pub translocations: &'a [FactsTranslocation],
    pub origin_us: u64,
    pub step_us: NonZeroU64,
}

/// Publish reads, equipment changes and translocations, then publish the selected
/// palette's labels. Return that exact palette for the subsequent action pass.
///
/// # Panics
/// Requires the preceding coverage-envelope pass.
pub fn assemble_facts_replay_abilities_and_translocations(
    doc: &mut ReplayDocument,
    input: FactsReplayAbilityDocumentInput<'_>,
) -> Option<ReplayAbilityPalette> {
    let coverage = doc
        .coverage
        .as_mut()
        .expect("ability stage requires coverage envelope");
    let c = &mut doc.content;
    if !c.inventory.is_empty() || !c.grenades.is_empty() || !c.grenade_reads.is_empty() {
        c.grenade_labels.clone_from(&input.labels.grenades);
    }
    let slots = c
        .tracks
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|t| t.slot)
        .collect();
    let abilities = publish_replay_abilities(
        build_facts_replay_ability_reads(
            input.ranks,
            input.inventory,
            input.origin_us,
            input.step_us,
        ),
        &slots,
        &input.labels.abilities,
    );
    c.abilities = abilities.reads;
    abilities.coverage.log();
    coverage.abilities = Some(abilities.coverage);
    let changes = build_facts_replay_equipment_changes(
        input.equipment_changes,
        input.equipment_stats,
        input.origin_us,
        input.step_us.get(),
        &slots,
    );
    c.equipment_changes = changes.changes;
    changes.coverage.log();
    coverage.equipment_changes = Some(changes.coverage);
    let translocations = build_facts_replay_translocations(
        input.translocations,
        &slots,
        input.origin_us,
        input.step_us.get(),
    );
    c.translocations = translocations.jumps;
    translocations.coverage.log();
    coverage.translocations = Some(translocations.coverage);
    c.ability_labels = abilities.labels;
    super::replay_abilities::log_replay_ability_palette(
        abilities.palette.as_ref(),
        c.abilities.len(),
        c.ability_labels.len(),
    );
    abilities.palette
}

pub struct FactsReplayAbilityActionDocumentInput<'a> {
    pub ranks: &'a [FactsAbilityRank],
    pub lives: &'a [IdentityLife],
    pub palette: Option<&'a ReplayAbilityPalette>,
    pub impulse_families: &'a BTreeSet<String>,
    pub charge_families: &'a BTreeSet<String>,
    pub impulses: &'a [FactsAbilityImpulse],
    pub impulse_stats: &'a FactsAbilityImpulseStats,
    pub charges: &'a [FactsAbilityCharge],
    pub charge_stats: &'a FactsAbilityChargeStats,
    pub origin_us: u64,
    pub step_us: u64,
}

/// Always replace both action layers. Independently scanned channels replace
/// their coverage; an unscanned channel preserves seeded coverage and warns.
/// The palette must be the one returned by the preceding ability pass.
///
/// # Panics
/// Requires the preceding coverage-envelope pass.
pub fn assemble_facts_replay_ability_actions(
    doc: &mut ReplayDocument,
    input: FactsReplayAbilityActionDocumentInput<'_>,
) {
    let coverage = doc
        .coverage
        .as_mut()
        .expect("ability action stage requires coverage envelope");
    let c = &mut doc.content;
    let slots = c
        .tracks
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|t| t.slot)
        .collect();
    let context = |measured_families| FactsReplayAbilityContext {
        ranks: input.ranks,
        lives: input.lives,
        palette: input.palette.map(|p| &p.families),
        measured_families,
        published_slots: &slots,
        origin_us: input.origin_us,
        step_us: input.step_us,
    };
    let impulses = build_facts_replay_ability_impulses(
        input.impulses,
        input.impulse_stats,
        context(input.impulse_families),
    );
    c.ability_impulses = impulses.impulses;
    if input.impulse_stats.scanned {
        impulses.coverage.log();
        coverage.ability_impulses = Some(impulses.coverage);
    } else {
        tracing::warn!(
            lectures = input.impulses.len(),
            "rejeu : impulsions de capacite NON BALAYEES — aucune couverture publiee"
        );
    }
    let charges = build_facts_replay_ability_charges(
        input.charges,
        input.charge_stats,
        context(input.charge_families),
    );
    c.ability_charges = charges.charges;
    if input.charge_stats.scanned {
        charges.coverage.log();
        coverage.ability_charges = Some(charges.coverage);
    } else {
        tracing::warn!(
            lectures = input.charges.len(),
            "rejeu : charges d equipement NON BALAYEES — aucune couverture publiee"
        );
    }
}
