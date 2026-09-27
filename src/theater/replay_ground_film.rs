//! LegacyFilm ground weapons and recurrent pads share the same creation/census assembly.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayGroundPadCoverage {
    pub scanned: bool,
    pub slots: i64,
    pub anchors: i64,
    pub accepted: i64,
    pub kept: usize,
    pub rejected: usize,
    pub objectives: usize,
    pub dropped: usize,
    pub spawned: usize,
    pub at_rest: usize,
    pub clusters: usize,
    pub pads: usize,
    pub occupancies: usize,
    pub dated: usize,
    pub unknown: usize,
    pub never: usize,
    pub cycles: usize,
    pub powerup_scanned: bool,
    pub powerup_accepted: i64,
    pub powerup_kept: usize,
    pub powerup_pads: usize,
}
impl ReplayGroundPadCoverage {
    pub fn balanced(&self) -> bool {
        (self.kept as i64).wrapping_add(self.rejected as i64) == self.accepted
            && self.dated + self.unknown + self.never == self.occupancies
            && (self.powerup_kept as i64) <= self.powerup_accepted
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilmReplayGround {
    pub pads: Vec<GroundWeaponPad>,
    pub pickups: Vec<GroundPadPickup>,
    pub coverage: ReplayGroundPadCoverage,
    pub dating: PadDatingStats,
    pub items: ReplayGroundWeapons,
}

/// Both complete scans are required for each archetype. A missing motion scan
/// must not turn every creation into a stationary pad candidate.
#[allow(dead_code)]
pub(crate) fn build_film_replay_ground(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    pickups: &[ReplayPickup],
) -> FilmReplayGround {
    build_film_replay_ground_with_catalog(film, players, pickups, &replay_equipment_catalog())
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_ground_with_catalog(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    pickups: &[ReplayPickup],
    catalog: &ReplayEquipmentCatalog,
) -> FilmReplayGround {
    let mut out = ReplayGroundPads::default();
    let mut positions: Vec<_> = film
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
    positions.sort_by_key(|p| p.timestamp_us);
    let powerups: BTreeMap<_, _> = catalog
        .families
        .iter()
        .filter(|(_, name)| name.starts_with("powerup_"))
        .map(|(id, name)| (*id, name.clone()))
        .collect();
    let empty = BTreeSet::new();
    for archetype in [42, 37] {
        let creations = if archetype == 42 {
            film.ground_weapon_creations.as_ref()
        } else {
            film.equipment_pad_creations
                .as_ref()
                .or_else(|| film.equipment_placements.as_ref().map(|s| &s.creations))
        };
        let Some((creations, tracks, census)) = creations
            .zip(film.ground_object_tracks.get(&archetype))
            .zip(film.world_object_keyframes.get(&archetype))
            .map(|((a, b), c)| (a, b, c))
        else {
            continue;
        };
        let c = &mut out.coverage;
        if archetype == 42 {
            c.scanned = true;
            c.slots = creations.stats.slots as i64;
            c.anchors = creations.stats.anchors as i64;
            c.accepted = creations.stats.accepted as i64;
        } else {
            c.powerup_scanned = true;
            c.powerup_accepted = creations.stats.accepted as i64;
        }
        if players.clock.step_us == 0 {
            continue;
        }
        let objects = assemble_ground_objects(
            &creations.records,
            &tracks.tracks,
            census,
            &positions,
            GroundObjectRule {
                kind: if archetype == 42 { "weapon" } else { "powerup" },
                families: if archetype == 42 {
                    v41_weapon_families()
                } else {
                    &powerups
                },
                objectives: if archetype == 42 {
                    &catalog.objective_objects
                } else {
                    &empty
                },
            },
        );
        append_ground_pad_chain(
            &mut out,
            objects,
            archetype,
            GroundPadClock {
                origin_us: players.clock.origin_us,
                step_us: players.clock.step_us,
                frames: players.clock.frame_count,
            },
        );
    }
    let dating = date_pad_pickups(&out.pads, &mut out.pickups, pickups);
    let items = build_replay_ground_weapons(
        &out.weapon_objects,
        film.weapon_changes
            .as_ref()
            .map_or(&[], |s| s.records.as_slice()),
        &positions,
        players.clock,
    );
    FilmReplayGround {
        pads: out.pads,
        pickups: out.pickups,
        coverage: out.coverage,
        dating,
        items,
    }
}

/// The native buildWeaponPads boundary, before event dating and ground-item publication.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayGroundPads {
    pub pads: Vec<GroundWeaponPad>,
    pub pickups: Vec<GroundPadPickup>,
    pub coverage: ReplayGroundPadCoverage,
    pub weapon_objects: Vec<GroundPickupObject>,
}
/// Both cache streams keep their own scan gate. Positions must be timestamp-sorted.
pub fn build_facts_replay_ground_pads(
    weapons: &FactsWorldObjectScan,
    powerups: &FactsWorldObjectScan,
    positions: &[FactsBipedPosition],
    clock: GroundPadClock,
    catalog: &ReplayEquipmentCatalog,
) -> ReplayGroundPads {
    let mut out = ReplayGroundPads {
        coverage: ReplayGroundPadCoverage {
            scanned: weapons.scanned,
            slots: weapons.stats.slots,
            anchors: weapons.stats.anchors,
            accepted: weapons.stats.accepted,
            powerup_scanned: powerups.scanned,
            powerup_accepted: powerups.stats.accepted,
            ..Default::default()
        },
        ..Default::default()
    };
    if clock.step_us == 0 {
        return out;
    }
    let positions: Vec<_> = positions
        .iter()
        .map(|p| ReplayPositionSample::from(p).position)
        .collect();
    let powerup_families: BTreeMap<_, _> = catalog
        .families
        .iter()
        .filter(|(_, name)| name.starts_with("powerup_"))
        .map(|(id, name)| (*id, name.clone()))
        .collect();
    let empty = BTreeSet::new();
    for (archetype, scan) in [(42, weapons), (37, powerups)] {
        if !scan.scanned {
            continue;
        }
        let objects = assemble_facts_ground_objects(
            &scan.creations,
            &scan.tracks,
            &scan.keyframes,
            &positions,
            GroundObjectRule {
                kind: if archetype == 42 { "weapon" } else { "powerup" },
                families: if archetype == 42 {
                    v41_weapon_families()
                } else {
                    &powerup_families
                },
                objectives: if archetype == 42 {
                    &catalog.objective_objects
                } else {
                    &empty
                },
            },
        );
        append_ground_pad_chain(&mut out, objects, archetype, clock);
    }
    out
}
fn append_ground_pad_chain(
    out: &mut ReplayGroundPads,
    objects: GroundObjectAssembly,
    archetype: u32,
    clock: GroundPadClock,
) {
    let c = &mut out.coverage;
    if archetype == 42 {
        c.kept = objects.objects.len();
        c.rejected = objects.rejected.total;
        c.objectives = objects.rejected.objectives;
    } else {
        c.powerup_kept = objects.objects.len();
    }
    if let Some(mut layer) = build_ground_pad_layer(&objects.objects, clock) {
        let n = layer.counts;
        if archetype == 42 {
            c.dropped = n.dropped;
            c.spawned = n.spawned;
            c.at_rest = n.at_rest;
            c.clusters = n.clusters;
            c.pads = n.pads;
        } else {
            c.powerup_pads = n.pads;
        }
        c.occupancies += n.occupancies;
        c.dated += n.dated;
        c.unknown += n.unknown;
        c.never += n.never;
        c.cycles += n.cycles;
        for p in &mut layer.pickups {
            p.pad += out.pads.len() as i64;
        }
        out.pads.extend(layer.pads);
        out.pickups.extend(layer.pickups);
    }
    if archetype == 42 {
        out.weapon_objects = objects.objects;
    }
}
