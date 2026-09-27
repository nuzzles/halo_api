//! Native weapon-change, pickup, pad and pad-dating document pass.
use super::*;

pub struct FactsReplayPickupDocumentInput<'a> {
    pub weapon_changes: &'a [FactsWeaponChange],
    pub pickups: &'a [FactsPickup],
    pub pickup_stats: &'a FactsPickupStats,
    /// Original input order is significant for equal-distance origin matches.
    pub positions: &'a [FactsBipedPosition],
    pub sorted_positions: &'a [FactsBipedPosition],
    pub weapons: &'a FactsWorldObjectScan,
    pub powerups: &'a FactsWorldObjectScan,
    pub spawn_points: &'a [MapSpawnPoint],
    pub spawn_points_state: &'a str,
    pub catalog: &'a ReplayEquipmentCatalog,
    pub identity: &'a ReplayIdentityState,
    pub origin_us: u64,
    pub step_us: u64,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsReplayPickupDocumentReport {
    /// Unmodified weapon objects retained for the later ground-item pass.
    pub weapon_objects: Vec<GroundPickupObject>,
    pub diagnostics: Vec<StatborgDiagnostic>,
}
/// Replace weapon changes, pickups and both pad streams, then date occupations.
/// The origin judge consumes the current document's equipment placements. Each
/// pad stream keeps its own scan gate. No source reads or fallback scans occur.
///
/// # Panics
/// Requires the preceding coverage-envelope pass.
pub fn assemble_facts_replay_pickups_and_pads(
    doc: &mut ReplayDocument,
    input: FactsReplayPickupDocumentInput<'_>,
) -> FactsReplayPickupDocumentReport {
    let coverage = doc
        .coverage
        .as_mut()
        .expect("pickup stage requires coverage envelope");
    let clock = GroundPadClock {
        origin_us: input.origin_us,
        step_us: input.step_us,
        frames: doc.content.frame_count,
    };
    let changes =
        build_facts_replay_weapon_changes(input.weapon_changes, input.origin_us, input.step_us);
    doc.content.weapon_changes = changes.changes;
    let mut diagnostics = Vec::new();
    super::facts_pickup_document_diagnostics::weapons(&mut diagnostics, &changes.coverage);
    coverage.weapon_changes = Some(changes.coverage);
    let positions: Vec<_> = input
        .positions
        .iter()
        .map(|p| ReplayPositionSample::from(p).position)
        .collect();
    let placements: Vec<_> = doc
        .content
        .equipment_placements
        .iter()
        .map(|p| PickupOriginPlacement {
            t0: p.t0,
            until_max: p.until_max,
            end: p.end.clone(),
            origin: p.origin.clone(),
            x: p.x,
            y: p.y,
            z: p.z,
        })
        .collect();
    let judge = PickupOriginJudge::new(
        input.spawn_points_state,
        input.spawn_points.to_vec(),
        &positions,
        &placements,
    );
    let mut resolve = |slot, time, frame| judge.resolve(slot, time, frame);
    let occupant = |slot, time| input.identity.xuid_at(slot, time);
    let pickups = build_facts_replay_pickups(
        input.pickups,
        input.pickup_stats,
        clock,
        ReplayPickupInputs {
            equipment_families: &input.catalog.families,
            weapon_keys: &input.catalog.weapon_keys,
            occupant: Some(&occupant),
            origin: Some(ReplayPickupOrigin {
                state: &judge.state,
                catalog_points: judge.catalog_points(),
                resolve: &mut resolve,
            }),
        },
    );
    doc.content.pickups = pickups.pickups;
    super::facts_pickup_document_diagnostics::pickups(&mut diagnostics, &pickups.coverage);
    coverage.pickups = Some(pickups.coverage);
    let pads = build_facts_replay_ground_pads(
        input.weapons,
        input.powerups,
        input.sorted_positions,
        clock,
        input.catalog,
    );
    doc.content.weapon_pads = pads.pads;
    doc.content.pad_pickups = pads.pickups;
    super::facts_pickup_document_diagnostics::pads(&mut diagnostics, &pads.coverage);
    coverage.ground_weapons = Some(pads.coverage);
    let dating = date_pad_pickups(
        &doc.content.weapon_pads,
        &mut doc.content.pad_pickups,
        &doc.content.pickups,
    );
    super::facts_pickup_document_diagnostics::dating(&mut diagnostics, &dating);
    coverage.pad_dating = Some(dating);
    FactsReplayPickupDocumentReport {
        weapon_objects: pads.weapon_objects,
        diagnostics,
    }
}
