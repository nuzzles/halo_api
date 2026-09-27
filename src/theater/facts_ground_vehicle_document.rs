//! Native ground-item publication, vehicle publication, then orphan-shot recovery.
use super::*;
use std::{collections::BTreeMap, num::NonZeroU64};

pub struct FactsReplayGroundVehicleDocumentInput<'a> {
    /// Objects retained by the preceding pickup/pad pass, in their original order.
    pub weapon_objects: &'a [GroundPickupObject],
    pub weapon_changes: &'a [FactsWeaponChange],
    pub sorted_positions: &'a [FactsBipedPosition],
    pub vehicles: &'a FactsVehicleScan,
    pub identity: &'a ReplayIdentityState,
    /// Original orphans retained by the initial combat pass.
    pub shot_orphans: &'a [FactsReplayOrphanShot],
    pub origin_us: u64,
    pub step_us: u64,
}

/// Publish ground items and vehicles before attempting the second shot gate.
/// Emits native stage diagnostics in publication order. Returned fallback counts
/// belong to assembly, not the scan-only facts cache. Frame count comes from the
/// current document; the caller supplies the assembly clock and sorted positions.
///
/// # Panics
/// Requires the preceding coverage-envelope pass.
pub fn assemble_facts_replay_ground_and_vehicles(
    doc: &mut ReplayDocument,
    input: FactsReplayGroundVehicleDocumentInput<'_>,
) -> BTreeMap<String, usize> {
    let coverage = doc
        .coverage
        .as_mut()
        .expect("ground/vehicle stage requires coverage envelope");
    let frames = doc.content.frame_count;
    let positions: Vec<_> = input
        .sorted_positions
        .iter()
        .map(|p| ReplayPositionSample::from(p).position)
        .collect();
    let ground = build_facts_replay_ground_weapons(
        input.weapon_objects,
        input.weapon_changes,
        &positions,
        IdentityClock {
            origin_us: input.origin_us,
            step_us: input.step_us,
            frame_count: frames,
        },
    );
    doc.content.ground_weapons = ground.weapons;
    let c = &ground.coverage;
    tracing::info!(
        objets = c.objects,
        publiees = c.published,
        auRepos = c.at_rest,
        lacheurNomme = c.dropper_named,
        prisesRecues = c.takes_total,
        ramasseurNomme = c.pickup_linked,
        finPickup = c.end_pickup,
        finVue = c.end_seen,
        finOuverte = c.end_open,
        "rejeu : armes au sol individuelles"
    );
    coverage.ground_weapon_items = Some(ground.coverage);
    let vehicles = build_facts_replay_vehicle_publication(
        input.vehicles,
        input.sorted_positions,
        input.identity,
        input.origin_us,
        input.step_us,
        frames,
    );
    doc.content.vehicles = vehicles.tracks;
    doc.content.vehicle_cycles = vehicles.cycles;
    coverage.vehicles = Some(vehicles.coverage);
    coverage.vehicles.as_ref().unwrap().log();
    replay_facts_vehicle_heading_sources(&input.vehicles.positions).log();
    vehicles.rides.log();
    // A zero interval cannot publish vehicle tracks, so native recovery has
    // already short-circuited before any clock division in that case.
    if let Some(step) = NonZeroU64::new(input.step_us) {
        attach_facts_replay_vehicle_shots_to_document(
            doc,
            input.shot_orphans,
            input.identity.indices_by_slot(),
            input.origin_us,
            step,
            frames,
        );
    }
    vehicles.fallbacks
}
