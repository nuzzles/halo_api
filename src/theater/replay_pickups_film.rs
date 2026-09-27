//! LegacyFilm pickup publication using temporal identity and observed placement origins.
use super::*;
/// Optional map spawn points are separate evidence. An empty state uses the
/// native `map_absent` marker; supplied empty point sets preserve their state.
#[derive(Default)]
pub struct FilmReplayPickupOptions<'a> {
    pub spawn_points_state: &'a str,
    pub spawn_points: &'a [MapSpawnPoint],
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_pickups(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    placements: &ReplayEquipmentPlacements,
    options: FilmReplayPickupOptions<'_>,
) -> ReplayPickupOutput {
    build_film_replay_pickups_with_catalog(
        film,
        players,
        placements,
        options,
        &replay_equipment_catalog(),
    )
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_pickups_with_catalog(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    placements: &ReplayEquipmentPlacements,
    options: FilmReplayPickupOptions<'_>,
    catalog: &ReplayEquipmentCatalog,
) -> ReplayPickupOutput {
    let positions: Vec<_> = film
        .biped_positions
        .as_ref()
        .into_iter()
        .flat_map(|s| s.accepted())
        .map(|p| ReplayPlayerPosition {
            slot: p.record.slot,
            timestamp_us: p.source.timestamp_us,
            x: p.record.world[0],
            y: p.record.world[1],
            z: p.record.world[2],
            has_world: true,
        })
        .collect();
    let dropped: Vec<_> = placements
        .placements
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
        options.spawn_points_state,
        options.spawn_points.to_vec(),
        &positions,
        &dropped,
    );
    let mut resolve = |slot, time, frame| judge.resolve(slot, time, frame);
    let occupant = |slot, time| players.registry.owners.state.xuid_at(slot, time);
    build_replay_pickups(
        &film.pickups,
        GroundPadClock {
            origin_us: players.clock.origin_us,
            step_us: players.clock.step_us,
            frames: players.clock.frame_count,
        },
        ReplayPickupInputs {
            equipment_families: &catalog.families,
            weapon_keys: &catalog.weapon_keys,
            occupant: Some(&occupant),
            origin: Some(ReplayPickupOrigin {
                state: &judge.state,
                catalog_points: judge.catalog_points(),
                resolve: &mut resolve,
            }),
        },
    )
}
