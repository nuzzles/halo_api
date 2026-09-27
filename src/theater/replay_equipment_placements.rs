//! Published equipment placements with written origins and census disappearance bounds.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayEquipmentPlacement {
    pub t0: i64,
    pub t1: i64,
    pub x: f32,
    pub y: f32,
    #[serde(default, skip_serializing_if = "placement_zero_f")]
    pub z: f32,
    pub family: String,
    pub id: String,
    pub owner: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<f32>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub origin: String,
    #[serde(default, skip_serializing_if = "placement_zero_i")]
    pub until: i64,
    #[serde(default, rename = "untilMax", skip_serializing_if = "placement_zero_i")]
    pub until_max: i64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub end: String,
}
fn placement_zero_f(v: &f32) -> bool {
    *v == 0.
}
fn placement_zero_i(v: &i64) -> bool {
    *v == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayEquipmentPlacementCoverage {
    pub scanned: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub widths: String,
    pub calibrated: bool,
    pub lives: i64,
    pub anchors: i64,
    pub confirmed: i64,
    pub placements: usize,
    pub named: usize,
    pub other: usize,
    pub with_owner: usize,
    pub with_heading: usize,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_family: BTreeMap<String, usize>,
    pub deployed: usize,
    pub dropped: usize,
    pub unknown: usize,
    pub end_seen: usize,
    pub end_open: usize,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_family_origin: BTreeMap<String, usize>,
    pub spawn_events: usize,
    pub spawn_lists: i64,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_cause: BTreeMap<String, usize>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayEquipmentPlacements {
    pub placements: Vec<ReplayEquipmentPlacement>,
    pub coverage: ReplayEquipmentPlacementCoverage,
    pub manifest_fallbacks: usize,
}
pub struct ReplayEquipmentPlacementInput<'a> {
    pub raw: &'a [EquipmentPlacement],
    pub stats: &'a EquipmentPlacementStats,
    pub positions: &'a [ReplayPositionSample],
    pub census: &'a WorldObjectKeyframes,
    pub spawns: &'a [EquipmentSpawnEvent],
    pub spawn_lists: usize,
    pub lives: &'a [IdentityLife],
    pub changes: &'a [EquipmentChange],
    pub families: &'a BTreeMap<u32, String>,
    pub spawned_piece_ids: &'a BTreeSet<u32>,
}
fn placement_frame(time: u64, origin: u64, step: u64) -> i64 {
    if time >= origin {
        ((time - origin) / step) as i64
    } else {
        (((origin - time).wrapping_add(step).wrapping_sub(1) / step) as i64).wrapping_neg()
    }
}
fn placement_clamp(t: i64, frames: i64) -> i64 {
    if t < 0 {
        0
    } else if t >= frames {
        frames.wrapping_sub(1)
    } else {
        t
    }
}
/// Publish supplied placements without rounding coordinates or requiring a player
/// track for the owner. Coverage is tallied before stable time/ID ordering.
pub fn build_replay_equipment_placements(
    input: ReplayEquipmentPlacementInput<'_>,
    clock: IdentityClock,
) -> ReplayEquipmentPlacements {
    let st = input.stats;
    let coverage = placement_coverage(
        st.scanned,
        (
            st.calibration.widths.lead as i64,
            st.calibration.widths.index as i64,
        ),
        st.lives as i64,
        st.anchors as i64,
        st.confirmed as i64,
    );
    let Some(step) = NonZeroU64::new(clock.step_us).filter(|_| !input.raw.is_empty()) else {
        return ReplayEquipmentPlacements {
            coverage,
            ..Default::default()
        };
    };
    let source = ReplayEquipmentOriginIndex::new(input.spawns, input.lives, input.changes);
    let ends = replay_equipment_ends(
        input.raw,
        input.census,
        clock.origin_us,
        step,
        clock.frame_count,
    );
    publish_placements(
        input.raw.iter().map(|p| PlacementValue {
            life: p.life,
            start: p.t0_us,
            end: p.t1_us,
            world: [p.x, p.y, p.z],
            id: p.global_id,
        }),
        ends,
        source,
        PlacementPublication {
            positions: input.positions,
            families: input.families,
            pieces: input.spawned_piece_ids,
            spawn_lists: input.spawn_lists as i64,
        },
        coverage,
        clock,
    )
}
/// Exact cached placements plus caller-provided identity and catalog context.
/// Positions retain native input order and must be sorted by timestamp.
pub struct FactsReplayEquipmentPlacementInput<'a> {
    pub raw: &'a [FactsPlacement],
    pub stats: &'a FactsPlacementStats,
    pub positions: &'a [FactsBipedPosition],
    pub census: &'a FactsWorldKeyframes,
    pub spawns: &'a [FactsSpawnEvent],
    pub spawn_lists: i64,
    pub lives: &'a [IdentityLife],
    pub changes: &'a [FactsEquipmentChange],
    pub families: &'a BTreeMap<u32, String>,
    pub spawned_piece_ids: &'a BTreeSet<u32>,
}
pub fn build_facts_replay_equipment_placements(
    input: FactsReplayEquipmentPlacementInput<'_>,
    clock: IdentityClock,
) -> ReplayEquipmentPlacements {
    let st = input.stats;
    let coverage = placement_coverage(
        st.scanned,
        st.calibration.widths,
        st.lives,
        st.anchors,
        st.confirmed,
    );
    let Some(step) = NonZeroU64::new(clock.step_us).filter(|_| !input.raw.is_empty()) else {
        return ReplayEquipmentPlacements {
            coverage,
            ..Default::default()
        };
    };
    let source = ReplayEquipmentOriginIndex::from_facts(input.spawns, input.lives, input.changes);
    let ends = replay_facts_equipment_ends(
        input.raw,
        input.census,
        clock.origin_us,
        step,
        clock.frame_count,
    );
    let positions: Vec<_> = input
        .positions
        .iter()
        .map(ReplayPositionSample::from)
        .collect();
    publish_placements(
        input.raw.iter().map(|p| PlacementValue {
            life: EquipmentLifeKey {
                slot: p.life.slot,
                generation: p.life.generation,
            },
            start: p.start_us,
            end: p.end_us,
            world: p.position,
            id: p.global_id,
        }),
        ends,
        source,
        PlacementPublication {
            positions: &positions,
            families: input.families,
            pieces: input.spawned_piece_ids,
            spawn_lists: input.spawn_lists,
        },
        coverage,
        clock,
    )
}
fn placement_coverage(
    scanned: bool,
    widths: (i64, i64),
    lives: i64,
    anchors: i64,
    confirmed: i64,
) -> ReplayEquipmentPlacementCoverage {
    let calibrated = widths.0 > 0 && widths.1 > 0;
    ReplayEquipmentPlacementCoverage {
        scanned,
        calibrated,
        widths: if calibrated {
            format!("{}/{}", widths.0, widths.1)
        } else {
            String::new()
        },
        lives,
        anchors,
        confirmed,
        ..Default::default()
    }
}
struct PlacementValue {
    life: EquipmentLifeKey,
    start: u64,
    end: u64,
    world: [f32; 3],
    id: u32,
}
struct PlacementPublication<'a> {
    positions: &'a [ReplayPositionSample],
    families: &'a BTreeMap<u32, String>,
    pieces: &'a BTreeSet<u32>,
    spawn_lists: i64,
}
fn publish_placements(
    raw: impl Iterator<Item = PlacementValue>,
    ends: Vec<ReplayEquipmentEnd>,
    source: ReplayEquipmentOriginIndex,
    input: PlacementPublication<'_>,
    coverage: ReplayEquipmentPlacementCoverage,
    clock: IdentityClock,
) -> ReplayEquipmentPlacements {
    let mut out = ReplayEquipmentPlacements {
        coverage,
        ..Default::default()
    };
    out.coverage.spawn_events = source.spawn_events;
    out.coverage.spawn_lists = input.spawn_lists;
    for (p, end) in raw.zip(ends) {
        let t0 = placement_frame(p.start, clock.origin_us, clock.step_us);
        let t1 = placement_frame(p.end, clock.origin_us, clock.step_us);
        if t1 < 0 || t0 >= clock.frame_count {
            continue;
        }
        let owner = super::replay_equipment_origin::equipment_owner_values(
            input.positions,
            p.start,
            p.world,
        );
        let origin = source.classify_values(
            p.life,
            p.start,
            input.pieces.contains(&p.id),
            owner.as_ref().map(|o| o.slot),
        );
        out.manifest_fallbacks += usize::from(origin.manifest_fallback);
        let family = input
            .families
            .get(&p.id)
            .filter(|f| !f.is_empty())
            .cloned()
            .unwrap_or_else(|| "other".into());
        let pl = ReplayEquipmentPlacement {
            t0: placement_clamp(t0, clock.frame_count),
            t1: placement_clamp(t1, clock.frame_count),
            x: p.world[0],
            y: p.world[1],
            z: p.world[2],
            family,
            id: format!("0x{:08x}", p.id),
            owner: owner.as_ref().map_or(-1, |o| i64::from(o.slot)),
            h: owner.and_then(|o| o.heading),
            origin: origin.origin,
            until: end.until,
            until_max: end.until_max,
            end: end.end,
        };
        let c = &mut out.coverage;
        c.placements += 1;
        *c.by_cause.entry(origin.cause).or_default() += 1;
        *c.by_family.entry(pl.family.clone()).or_default() += 1;
        *c.by_family_origin
            .entry(format!("{}/{}", pl.family, pl.origin))
            .or_default() += 1;
        if pl.family == "other" {
            c.other += 1;
        } else {
            c.named += 1;
        }
        c.with_owner += usize::from(pl.owner >= 0);
        c.with_heading += usize::from(pl.h.is_some());
        match pl.origin.as_str() {
            "deployed" => c.deployed += 1,
            "dropped" => c.dropped += 1,
            _ => c.unknown += 1,
        }
        match pl.end.as_str() {
            "seen" => c.end_seen += 1,
            "open" => c.end_open += 1,
            _ => {}
        }
        out.placements.push(pl);
    }
    out.placements
        .sort_by(|a, b| (a.t0, &a.id).cmp(&(b.t0, &b.id)));
    out
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEquipmentCatalog {
    #[serde(default)]
    pub objective_labels: BTreeMap<u32, ReplayLabel>,
    #[serde(default)]
    pub objective_families: BTreeMap<u32, String>,
    #[serde(default)]
    pub objective_objects: BTreeSet<u32>,
    #[serde(default)]
    pub weapon_keys: BTreeMap<u32, String>,
    pub families: BTreeMap<u32, String>,
    pub spawned_piece_ids: BTreeSet<u32>,
}
/// Production-loaded title families and the pinned spawned-wall-panel identities.
pub fn replay_equipment_catalog() -> ReplayEquipmentCatalog {
    serde_json::from_str(include_str!("reference/equipment_catalog.json"))
        .expect("pinned equipment catalog is valid")
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_equipment_placements(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> ReplayEquipmentPlacements {
    build_film_replay_equipment_placements_with_catalog(film, players, &replay_equipment_catalog())
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_equipment_placements_with_catalog(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    catalog: &ReplayEquipmentCatalog,
) -> ReplayEquipmentPlacements {
    let mut positions: Vec<_> = film
        .biped_positions
        .as_ref()
        .into_iter()
        .flat_map(|s| s.accepted())
        .map(|p| {
            ReplayPositionSample::from_companions(
                ReplayPlayerPosition {
                    slot: p.record.slot,
                    timestamp_us: p.source.timestamp_us,
                    x: p.record.world[0],
                    y: p.record.world[1],
                    z: p.record.world[2],
                    has_world: true,
                },
                &p.record.companions,
            )
        })
        .collect();
    positions.sort_by_key(|p| p.position.timestamp_us);
    let empty_stats = EquipmentPlacementStats::default();
    let empty_census = WorldObjectKeyframes::default();
    let (raw, stats) = film
        .equipment_placements
        .as_ref()
        .map_or((&[][..], &empty_stats), |s| {
            (s.placements.as_slice(), &s.stats)
        });
    let changes = film
        .equipment_changes
        .as_ref()
        .map_or(&[][..], |s| s.assembly.records.as_slice());
    build_replay_equipment_placements(
        ReplayEquipmentPlacementInput {
            raw,
            stats,
            positions: &positions,
            census: film
                .world_object_keyframes
                .get(&37)
                .unwrap_or(&empty_census),
            spawns: &film.equipment_spawns.records,
            spawn_lists: film.equipment_spawns.stats.lists,
            lives: players.registry.owners.state.lives(),
            changes,
            families: &catalog.families,
            spawned_piece_ids: &catalog.spawned_piece_ids,
        },
        players.clock,
    )
}
