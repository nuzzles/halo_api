//! Assemble the complete native vehicle layer from decoded film observations.
use super::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, num::NonZeroU64};

pub struct ReplayVehicleScan<'a> {
    pub scanned: bool,
    pub keyframes: &'a WorldObjectKeyframes,
    pub creations: &'a [EquipmentCreation],
    pub positions: &'a [ReplayVehiclePosition],
    pub events: &'a [VehicleEvent],
    pub aims: &'a [BipedAim],
    /// Vehicle death evidence, already filtered to archetype 40.
    pub deaths: &'a [ReplayVehicleDeathEvidence],
    pub occupancy: &'a [VehicleOccupancy],
    pub march_default_retained: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayVehiclePublication {
    pub tracks: Vec<ReplayVehicleTrack>,
    pub cycles: Vec<ReplayVehicleCycle>,
    pub coverage: ReplayVehicleCoverage,
    pub rides: ReplayVehicleRideTally,
    pub fallbacks: BTreeMap<String, usize>,
}
/// Publish census-bounded lives, then merge relays and count the resulting layer.
/// Respawn cycles are computed from the merged publication. Combat shot recovery
/// remains a subsequent stage.
pub fn build_replay_vehicle_publication(
    scan: ReplayVehicleScan<'_>,
    bipeds: &[ReplayPlayerPosition],
    identity: &ReplayIdentityState,
    origin_us: u64,
    step_us: u64,
    frames: i64,
) -> ReplayVehiclePublication {
    let mut out = ReplayVehiclePublication::default();
    out.coverage.scanned = scan.scanned;
    out.coverage.aim_reads = scan.aims.len() as i64;
    let Some(step) = NonZeroU64::new(step_us).filter(|_| scan.scanned) else {
        return out;
    };
    if scan.march_default_retained {
        out.fallbacks
            .insert("repli_cadre_de_marche_par_defaut_conserve".into(), 1);
    }
    let (lives, deaths) = build_replay_vehicle_lives(scan.keyframes, scan.deaths);
    out.coverage.lives = lives.len() as i64;
    out.coverage.deaths_read = deaths.read as i64;
    out.coverage.deaths_matched = deaths.matched as i64;
    out.coverage.deaths_unmatched = deaths.unmatched as i64;
    out.coverage.deaths_tail_desync = deaths.tail_desync as i64;
    let spawns = replay_vehicle_spawns_by_life(scan.creations);
    let positions = replay_vehicle_positions_by_slot(scan.positions);
    let aims = replay_vehicle_aim_by_slot(scan.aims);
    let drawable = replay_vehicle_drawable_lives(&lives, &spawns, &positions);
    let (mut rides, tally) = build_replay_vehicle_rides(ReplayVehicleRidesContext {
        film: ReplayVehicleRideContext {
            bipeds,
            aim_by_slot: &aims,
            identity,
            occupancy: scan.occupancy,
            lives: &lives,
            drawable: &drawable,
            origin_us,
            step_us,
            frames,
        },
        events: scan.events,
        vehicles: &positions,
    });
    out.rides = tally;
    for life in &lives {
        if let Some(track) = build_replay_vehicle_track(
            life,
            spawns.get(&life.key()),
            positions.get(&life.slot).map_or(&[], Vec::as_slice),
            rides.remove(&life.key()).unwrap_or_default(),
            origin_us,
            step,
            frames,
        ) {
            out.tracks.push(track);
        } else {
            out.coverage.no_position += 1;
        }
    }
    finish_vehicle_publication(out, step_us)
}
fn finish_vehicle_publication(
    mut out: ReplayVehiclePublication,
    step_us: u64,
) -> ReplayVehiclePublication {
    out.tracks.sort_by_key(|t| (t.t0, t.slot, t.r#gen));
    let (tracks, merged) = merge_replay_vehicle_relays(out.tracks);
    out.tracks = tracks;
    out.coverage.merged = merged as i64;
    tally_replay_vehicle_coverage(&out.tracks, &mut out.coverage);
    out.cycles = build_replay_vehicle_cycles(&out.tracks, step_us, &mut out.coverage);
    if out.coverage.family_unknown > 0 {
        out.fallbacks.insert(
            "repli_chassis_vehicule_marqueur_neutre".into(),
            out.coverage.family_unknown as usize,
        );
    }
    out
}

/// Assemble the native vehicle layer entirely from cached observations.
/// Relay merging precedes coverage and cycle calculation; shot attachment remains
/// a later document stage. Scan counters are not a substitute for the scanned gate.
pub fn build_facts_replay_vehicle_publication(
    scan: &FactsVehicleScan,
    bipeds: &[FactsBipedPosition],
    identity: &ReplayIdentityState,
    origin_us: u64,
    step_us: u64,
    frames: i64,
) -> ReplayVehiclePublication {
    let mut out = ReplayVehiclePublication::default();
    out.coverage.scanned = scan.scanned;
    out.coverage.aim_reads = scan.aims.len() as i64;
    let Some(step) = NonZeroU64::new(step_us).filter(|_| scan.scanned) else {
        return out;
    };
    if scan.deaths.stats.default_frame {
        out.fallbacks
            .insert("repli_cadre_de_marche_par_defaut_conserve".into(), 1);
    }
    let (lives, deaths) = build_facts_replay_vehicle_lives(
        &scan.keyframes,
        scan.deaths.deaths.as_deref().unwrap_or(&[]),
    );
    out.coverage.lives = lives.len() as i64;
    out.coverage.deaths_read = deaths.read as i64;
    out.coverage.deaths_matched = deaths.matched as i64;
    out.coverage.deaths_unmatched = deaths.unmatched as i64;
    out.coverage.deaths_tail_desync = deaths.tail_desync as i64;
    let spawns = replay_facts_vehicle_spawns_by_life(&scan.creations);
    let positions = replay_facts_vehicle_positions_by_slot(&scan.positions);
    let aims = replay_facts_vehicle_aim_by_slot(&scan.aims);
    let drawable = replay_facts_vehicle_drawable_lives(&lives, &spawns, &positions);
    let (mut rides, tally) = build_facts_replay_vehicle_rides(FactsReplayVehicleRidesContext {
        film: FactsReplayVehicleRideContext {
            bipeds,
            aim_by_slot: &aims,
            identity,
            occupancy: &scan.occupancy,
            lives: &lives,
            drawable: &drawable,
            origin_us,
            step_us,
            frames,
        },
        events: &scan.events,
        vehicles: &positions,
    });
    out.rides = tally;
    for life in &lives {
        if let Some(track) = build_facts_replay_vehicle_track(
            life,
            spawns.get(&life.key()),
            positions.get(&life.slot).map_or(&[], Vec::as_slice),
            rides.remove(&life.key()).unwrap_or_default(),
            origin_us,
            step,
            frames,
        ) {
            out.tracks.push(track);
        } else {
            out.coverage.no_position += 1;
        }
    }
    finish_vehicle_publication(out, step_us)
}
