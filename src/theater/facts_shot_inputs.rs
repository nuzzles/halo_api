//! Cached shots preserve their full signed identity and exact source cache record.
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};
#[derive(Debug, Clone, PartialEq)]
pub struct FactsReplayOrphanShot {
    pub event: FactsFireEvent,
    pub reason: ReplayShotOrphanReason,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsReplayShotPublication {
    pub shots: Vec<ReplayShot>,
    pub orphans: Vec<FactsReplayOrphanShot>,
    pub coverage: ReplayLayerCoverage,
}
impl FactsReplayShotPublication {
    /// Native publication checks slot membership, not the individual track time.
    /// Rejected publications do not become vehicle-recovery orphans.
    pub fn retain_published_tracks(&mut self, tracks: &[ReplayTrack]) {
        let slots: BTreeSet<_> = tracks.iter().map(|t| t.slot).collect();
        let before = self.shots.len();
        self.shots.retain(|s| slots.contains(&s.slot));
        self.coverage.unpublished = self
            .coverage
            .unpublished
            .wrapping_add((before - self.shots.len()) as i64);
        self.coverage.attached = self.shots.len() as i64;
    }
}
/// Use the same attachment computation as the recording API, without narrowing
/// native cache player indices or inventing chunk/packet/bit metadata.
pub fn build_facts_replay_shots(
    positions: &[ReplayPlayerPosition],
    events: &[FactsFireEvent],
    origin_us: u64,
    step_us: NonZeroU64,
    owners: &BTreeMap<u32, i64>,
) -> FactsReplayShotPublication {
    let values: Vec<_> = events.iter().map(facts_shot_value).collect();
    let result =
        super::replay_shots::assemble_shots(positions, &values, origin_us, step_us, owners);
    FactsReplayShotPublication {
        shots: result.shots,
        coverage: result.coverage,
        orphans: result
            .orphans
            .into_iter()
            .map(|(i, reason)| FactsReplayOrphanShot {
                event: events[i].clone(),
                reason,
            })
            .collect(),
    }
}

fn facts_shot_value(e: &FactsFireEvent) -> super::replay_shots::ShotEventValue {
    super::replay_shots::ShotEventValue {
        timestamp_us: e.timestamp_us,
        film_index: e.film_index,
        weapon_id: e.weapon_id,
        heading: e.has_aim.then(|| {
            let h = (e.aim[1] as f64).atan2(e.aim[0] as f64) * 180.0 / std::f64::consts::PI;
            if h < 0.0 { h + 360.0 } else { h }
        }),
    }
}
/// Recover cached orphans once through published vehicle rides. Original cache
/// events remain available, and signed identity values are never narrowed.
#[allow(clippy::too_many_arguments)]
pub fn attach_facts_replay_vehicle_shots(
    shots: &mut FactsReplayShotPublication,
    vehicles: &[ReplayVehicleTrack],
    owners: &BTreeMap<u32, i64>,
    published_slots: &BTreeSet<u32>,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
    coverage: Option<&mut ReplayVehicleCoverage>,
) -> Option<&'static str> {
    let orphans: Vec<_> = shots
        .orphans
        .iter()
        .map(|o| (facts_shot_value(&o.event), o.reason))
        .collect();
    super::replay_vehicle_shots::recover_vehicle_shots(
        &mut shots.shots,
        &mut shots.coverage,
        &orphans,
        vehicles,
        owners,
        published_slots,
        origin,
        step,
        frames,
        coverage,
    )
}

/// Native document-stage vehicle recovery, called after vehicle publication and
/// coverage attachment. Cached orphan records remain unchanged. An absent coverage
/// envelope is a no-op, matching attachVehicleShots rather than inventing one.
pub fn attach_facts_replay_vehicle_shots_to_document(
    doc: &mut ReplayDocument,
    orphans: &[FactsReplayOrphanShot],
    owners: &BTreeMap<u32, i64>,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
) {
    let Some(coverage) = doc.coverage.as_mut() else {
        return;
    };
    let values: Vec<_> = orphans
        .iter()
        .map(|o| (facts_shot_value(&o.event), o.reason))
        .collect();
    let published = doc
        .content
        .tracks
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|t| t.slot)
        .collect();
    let before = doc.content.shots.len();
    let verdict = super::replay_vehicle_shots::recover_vehicle_shots(
        &mut doc.content.shots,
        &mut coverage.shots,
        &values,
        &doc.content.vehicles,
        owners,
        &published,
        origin,
        step,
        frames,
        coverage.vehicles.as_mut(),
    );
    if let Some(verdict) = verdict {
        coverage.verdict.insert("shots".into(), verdict.into());
        if let Some(c) = &coverage.vehicles {
            tracing::info!(
                publies = doc.content.shots.len() - before,
                poses = c.shots,
                ambigus = c.shots_ambiguous,
                sansPosition = c.shots_unplaced,
                orphelinsHorsEpisode = c.shots_no_ride,
                "rejeu : tirs en vehicule"
            );
        }
    }
}
