//! Native label, neutral-death, inventory and carried-grenade document pass.
use super::*;
use std::{collections::BTreeSet, num::NonZeroU64};

pub struct FactsReplayInventoryDocumentInput<'a> {
    pub labels: &'a ReplayLabelCatalog,
    pub neutral_deaths: &'a [ReplayNeutralDeath],
    /// None means no caller input; Some(empty) means a completed empty scan.
    pub inventory: Option<&'a [FactsKeyframeInventory]>,
    pub deltas: &'a [FactsInventoryDelta],
    pub ammo_refused: bool,
    pub deaths: &'a [IdentityDeath],
    pub identity: &'a ReplayIdentityState,
    pub death_offset_ms: i64,
    pub origin_us: u64,
    /// The assembler has already applied its nonzero interval default.
    pub step_us: NonZeroU64,
}

/// Replace labels and published reads using current tracks. Missing inventory
/// input or an empty built grenade stream leaves existing coverage untouched.
/// Nonempty caller kill effects replace the table; empty effects preserve it.
/// Returns the number of empty inventory readings corroborated by death evidence.
/// Emits diagnostics at their native mutation boundaries.
pub fn assemble_facts_replay_labels_and_inventory(
    doc: &mut ReplayDocument,
    input: FactsReplayInventoryDocumentInput<'_>,
) -> usize {
    let content = &mut doc.content;
    content.weapon_labels = build_replay_weapon_labels(
        &content.loadouts,
        &content.shots,
        &content.weapon_pads,
        input.labels,
    );
    if !input.labels.effects.is_empty() {
        content.kill_effects.clone_from(&input.labels.effects);
    }
    let tracks = content.tracks.as_deref().unwrap_or_default();
    content.neutral_deaths = input.neutral_deaths.to_vec();
    retain_replay_neutral_deaths(
        &mut content.neutral_deaths,
        tracks,
        &input.identity.naming_bridge(),
    );
    let raw = input.inventory.unwrap_or_default();
    let (mut reads, dropped_before_origin) =
        build_facts_replay_inventory(raw, input.origin_us, input.step_us);
    let built = reads.len();
    let slots: BTreeSet<_> = tracks.iter().map(|t| t.slot).collect();
    reads.retain(|r| slots.contains(&r.slot));
    content.inventory = reads;
    if let Some(coverage) = doc.coverage.as_mut().filter(|_| input.inventory.is_some()) {
        let c = ReplayInventoryCoverage {
            decoded: raw.len(),
            dropped_before_origin,
            unpublished: built - content.inventory.len(),
            published: content.inventory.len(),
        };
        tracing::info!(
            decodees = c.decoded,
            ecarteesAvantOrigine = c.dropped_before_origin,
            ecarteesSansPiste = c.unpublished,
            publiees = c.published,
            "rejeu : couverture inventaire"
        );
        coverage.inventory = Some(c);
    }
    let marked = mark_replay_inventory_dead(
        &mut content.inventory,
        input.deaths,
        input.identity,
        input.death_offset_ms,
        IdentityClock {
            origin_us: input.origin_us,
            step_us: input.step_us.get(),
            frame_count: content.frame_count,
        },
    );
    log_replay_inventory_empty(&content.inventory, marked);
    let grenades = publish_replay_grenade_reads(
        build_facts_replay_grenade_reads(raw, input.deltas, input.origin_us, input.step_us),
        tracks,
        input.ammo_refused,
    );
    content.grenade_reads = grenades.reads;
    if let Some((coverage, c)) = doc.coverage.as_mut().zip(grenades.coverage) {
        tracing::info!(
            imagesCles = c.from_keyframe,
            delta = c.from_delta,
            ecarteesSansPiste = c.unpublished,
            canalMunitionsRefuse = c.ammo_refused,
            "rejeu : couverture des grenades portees"
        );
        coverage.grenade_reads = Some(c);
    }
    marked
}
