//! Native combat passes, including recovery of shots from published vehicle rides.
use super::*;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmReplayCombat {
    pub shots: ReplayShotPublication,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vehicles: Option<ReplayVehiclePublication>,
    /// Replacement shot verdict when vehicle recovery attaches additional shots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vehicle_shot_verdict: Option<String>,
    pub loadouts: Vec<ReplayLoadout>,
    pub grenade_reads: ReplayGrenadeReads,
    /// None retains unavailable scan context rather than inventing an empty scan.
    pub projectiles: Option<ReplayProjectilePublication>,
    pub grenades: Option<ReplayGrenadePublication>,
}
/// Apply the native shot, carried-weapon, projectile and grenade passes using the
/// automatic player's identity bridge, frame grid and published-track set.
#[allow(dead_code)]
pub(crate) fn build_film_replay_combat(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> Option<FilmReplayCombat> {
    let step = NonZeroU64::new(players.clock.step_us)?;
    let mut positions: Vec<_> = film
        .biped_positions
        .as_ref()?
        .accepted()
        .map(|p| {
            let [x, y, z] = p.record.world;
            ReplayPlayerPosition {
                slot: p.record.slot,
                timestamp_us: p.source.timestamp_us,
                x,
                y,
                z,
                has_world: true,
            }
        })
        .collect();
    positions.sort_by_key(|p| p.timestamp_us);
    let origin = players.clock.origin_us;
    let owners = players.registry.owners.state.indices_by_slot();
    let tracks = &players.players.publication.tracks;
    let mut shots = build_replay_shots(&positions, &film.fire_events, origin, step, owners);
    shots.retain_published_tracks(tracks);
    let mut vehicles = build_film_replay_vehicles(film, players);
    let vehicle_shot_verdict = vehicles.as_mut().and_then(|vehicles| {
        attach_replay_vehicle_shots(
            &mut shots,
            &vehicles.tracks,
            owners,
            &tracks.iter().map(|t| t.slot).collect(),
            origin,
            step,
            players.clock.frame_count,
            Some(&mut vehicles.coverage),
        )
        .map(str::to_owned)
    });
    let mut loadouts = build_replay_loadouts(&film.keyframe_loadouts, origin, step);
    retain_replay_loadouts(&mut loadouts, tracks);
    let projectiles = film
        .native_projectiles
        .as_ref()
        .map(|p| build_replay_projectiles(&p.tracks, origin, step));
    let empty_links = Default::default();
    let links = projectiles
        .as_ref()
        .map_or(&empty_links, |p| &p.published_by_raw);
    let raw = film
        .native_projectiles
        .as_ref()
        .map_or(&[][..], |p| p.tracks.as_slice());
    let grenades = film.grenade_throws.as_ref().map(|g| {
        let mut out =
            build_replay_grenades(&positions, &g.records, origin, step, owners, raw, links);
        out.retain_published_tracks(tracks);
        out
    });
    let inventory = film
        .keyframe_inventory
        .as_ref()
        .filter(|_| film.keyframe_inventory_error.is_none())
        .map_or(&[][..], |s| s.records.as_slice());
    let deltas = film
        .inventory_deltas
        .as_ref()
        .map_or(&[][..], |s| s.records.as_slice());
    let ammo_refused = film
        .inventory_deltas
        .as_ref()
        .is_some_and(|s| s.stats.ammo_refused);
    let grenade_reads = publish_replay_grenade_reads(
        build_replay_grenade_reads(inventory, deltas, origin, step),
        tracks,
        ammo_refused,
    );
    Some(FilmReplayCombat {
        shots,
        vehicles,
        vehicle_shot_verdict,
        loadouts,
        grenade_reads,
        projectiles,
        grenades,
    })
}
