//! Initial cached combat publication, before coverage and vehicle-shot recovery.
use super::*;
use std::{collections::BTreeMap, num::NonZeroU64};

pub struct FactsReplayCombatContext<'a> {
    /// Chronologically sorted recording positions, prepared by the timeline pass.
    pub positions: &'a [ReplayPlayerPosition],
    pub owners: &'a BTreeMap<u32, i64>,
    pub origin_us: u64,
    pub step_us: NonZeroU64,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsReplayCombatReport {
    pub shots: ReplayLayerCoverage,
    pub grenades: ReplayLayerCoverage,
    pub projectiles: Option<ReplayProjectileCoverage>,
    /// Preserve original cache records for the later vehicle recovery pass.
    pub shot_orphans: Vec<FactsReplayOrphanShot>,
}

/// Native initial combat pass. Coverage is measured and returned here; its
/// document envelope is created by a later pass. Existing coverage is untouched.
pub fn assemble_facts_replay_combat(
    doc: &mut ReplayDocument,
    events: &FactsEvents,
    ctx: FactsReplayCombatContext<'_>,
) -> FactsReplayCombatReport {
    let tracks = doc.content.tracks.as_deref().unwrap_or_default();
    let mut shots = build_facts_replay_shots(
        ctx.positions,
        &events.fire,
        ctx.origin_us,
        ctx.step_us,
        ctx.owners,
    );
    shots.retain_published_tracks(tracks);
    doc.content.shots = shots.shots;
    warn_if_lossy(&shots.coverage, "tirs");
    let mut loadouts = build_facts_replay_loadouts(&events.loadouts, ctx.origin_us, ctx.step_us);
    retain_replay_loadouts(&mut loadouts, tracks);
    doc.content.loadouts = loadouts;
    let projectiles =
        build_facts_replay_projectiles(&events.projectiles, ctx.origin_us, ctx.step_us);
    doc.content.projectiles = projectiles.projectiles;
    let projectile_coverage = (!events.projectiles.is_empty()).then_some(projectiles.coverage);
    if let Some(c) = &projectile_coverage
        && c.truncated > 0
    {
        tracing::info!(
            match_id = doc.content.match_id,
            tronquees = c.truncated,
            pistes = events.projectiles.len(),
            seuil_m = 10,
            "rejeu : trajectoires de projectile coupees a un pas impossible"
        );
    }
    let mut grenades = build_facts_replay_grenades(
        ctx.positions,
        &events.grenades,
        ctx.origin_us,
        ctx.step_us,
        ctx.owners,
        &events.projectiles,
        &projectiles.published_by_raw,
    );
    grenades.retain_published_tracks(tracks);
    doc.content.grenades = grenades.grenades;
    warn_if_lossy(&grenades.coverage, "grenades");
    FactsReplayCombatReport {
        shots: shots.coverage,
        grenades: grenades.coverage,
        projectiles: projectile_coverage,
        shot_orphans: shots.orphans,
    }
}

fn warn_if_lossy(c: &ReplayLayerCoverage, layer: &str) {
    if c.available == 0 {
        return;
    }
    for (cause, n) in [
        ("slotIntrouvable", c.no_slot),
        ("slotAmbigu", c.ambiguous),
        ("horsFenetre", c.out_of_window),
        ("sansTrajectoirePubliee", c.unpublished),
    ] {
        if (n as f64) / (c.available as f64) < 0.10 {
            continue;
        }
        tracing::warn!(
            calque = layer,
            cause,
            rejetes = n,
            disponibles = c.available,
            rattaches = c.attached,
            "rejeu : rejets au-dessus du seuil"
        );
    }
}
