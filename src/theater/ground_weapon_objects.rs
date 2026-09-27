//! Identity-filtered ground-object assembly from native creation and motion scans.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GroundObjectAppearance {
    pub kind: String,
    pub family: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(rename = "TUS")]
    pub timestamp_us: u64,
    pub class: String,
    pub has_delta: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GroundPickupObject {
    pub key: EquipmentLifeKey,
    #[serde(rename = "Appar")]
    pub appearance: GroundObjectAppearance,
    #[serde(rename = "FamilyID")]
    pub family_id: u32,
    #[serde(rename = "Pos")]
    pub position: [f32; 3],
    pub moved: bool,
    pub bounds: GroundWeaponPickupBounds,
    pub picker: GroundWeaponPickupHit,
    pub status: GroundWeaponPickupStatus,
    pub dropper_slot: i64,
    pub has_ammo: bool,
    pub ammo: GroundWeaponAmmo,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundObjectRejections {
    pub total: usize,
    pub objectives: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundObjectAssembly {
    pub objects: Vec<GroundPickupObject>,
    pub rejected: GroundObjectRejections,
}
/// Catalog names are already canonical. Objectives are excluded before catalog
/// membership, even when an objective identifier also appears in the catalog.
pub struct GroundObjectRule<'a> {
    pub kind: &'a str,
    pub families: &'a BTreeMap<u32, String>,
    pub objectives: &'a BTreeSet<u32>,
}
fn cmp_float(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}
pub(super) fn appearance_order(a: &GroundObjectAppearance, b: &GroundObjectAppearance) -> Ordering {
    a.timestamp_us
        .cmp(&b.timestamp_us)
        .then(a.kind.cmp(&b.kind))
        .then(a.family.cmp(&b.family))
        .then_with(|| cmp_float(a.x, b.x))
        .then_with(|| cmp_float(a.y, b.y))
        .then_with(|| cmp_float(a.z, b.z))
        .then(a.class.cmp(&b.class))
        .then(a.has_delta.cmp(&b.has_delta))
}
fn status_key(s: GroundWeaponPickupStatus) -> u8 {
    match s {
        GroundWeaponPickupStatus::Dated => 0,
        GroundWeaponPickupStatus::Never => 1,
        GroundWeaponPickupStatus::Unknown => 2,
    }
}
fn object_order(a: &GroundPickupObject, b: &GroundPickupObject) -> Ordering {
    appearance_order(&a.appearance, &b.appearance)
        .then(a.key.cmp(&b.key))
        .then(a.family_id.cmp(&b.family_id))
        .then_with(|| cmp_float(a.position[0], b.position[0]))
        .then_with(|| cmp_float(a.position[1], b.position[1]))
        .then_with(|| cmp_float(a.position[2], b.position[2]))
        .then(a.moved.cmp(&b.moved))
        .then(status_key(a.status).cmp(&status_key(b.status)))
        .then(a.dropper_slot.cmp(&b.dropper_slot))
        .then(a.bounds.low_us.cmp(&b.bounds.low_us))
        .then(a.bounds.high_us.cmp(&b.bounds.high_us))
        .then(a.bounds.seen_kf.cmp(&b.bounds.seen_kf))
        .then(a.bounds.never_picked.cmp(&b.bounds.never_picked))
        .then(a.bounds.no_later_kf.cmp(&b.bounds.no_later_kf))
        .then(a.picker.found.cmp(&b.picker.found))
        .then(a.picker.timestamp_us.cmp(&b.picker.timestamp_us))
        .then(a.picker.slot.cmp(&b.picker.slot))
        .then_with(|| {
            a.picker
                .distance_m
                .partial_cmp(&b.picker.distance_m)
                .unwrap_or(Ordering::Equal)
        })
}
/// Assemble one archetype's objects. Positions must be sorted by time, and the
/// census and tracks must come from that same archetype's slot band.
pub fn assemble_ground_objects(
    creations: &[EquipmentCreation],
    tracks: &[WorldObjectTrack],
    keyframes: &WorldObjectKeyframes,
    positions: &[ReplayPlayerPosition],
    rule: GroundObjectRule<'_>,
) -> GroundObjectAssembly {
    assemble_ground_values(
        creations.iter().map(|c| GroundCreationValue {
            key: EquipmentLifeKey {
                slot: c.slot,
                generation: c.generation,
            },
            timestamp_us: c.timestamp_us,
            position: [c.x, c.y, c.z],
            has_id: c.mpp_present[1],
            id: c.mpp_val[1] as u32,
            has_ammo: c.has_ammo,
            ammo: &c.ammo,
        }),
        tracks.iter().filter_map(|t| {
            Some((
                EquipmentLifeKey {
                    slot: t.slot,
                    generation: t.generation,
                },
                super::ground_weapon_lifetimes::source_ground_motion(t)?,
            ))
        }),
        &keyframes.times_us,
        keyframes
            .seen_us
            .iter()
            .map(|s| {
                (
                    EquipmentLifeKey {
                        slot: s.slot,
                        generation: s.generation,
                    },
                    s.times_us.as_slice(),
                )
            })
            .collect(),
        positions,
        rule,
    )
}
/// Assemble cache ground objects without narrowing or fabricating source fields.
pub fn assemble_facts_ground_objects(
    creations: &[FactsEquipmentCreation],
    tracks: &[FactsProjectileTrack],
    keyframes: &FactsWorldKeyframes,
    positions: &[ReplayPlayerPosition],
    rule: GroundObjectRule<'_>,
) -> GroundObjectAssembly {
    assemble_ground_values(
        creations.iter().map(|c| GroundCreationValue {
            key: EquipmentLifeKey {
                slot: c.slot,
                generation: c.generation,
            },
            timestamp_us: c.timestamp_us,
            position: c.position,
            has_id: c.mpp_present[1],
            id: c.mpp_val[1] as u32,
            has_ammo: c.has_ammo,
            ammo: &c.ammo,
        }),
        tracks.iter().filter_map(|t| {
            Some((
                EquipmentLifeKey {
                    slot: t.slot,
                    generation: t.generation,
                },
                super::ground_weapon_lifetimes::facts_ground_motion(t)?,
            ))
        }),
        &keyframes.times_us,
        keyframes
            .seen_us
            .iter()
            .map(|(&(slot, generation), times)| {
                (EquipmentLifeKey { slot, generation }, times.as_slice())
            })
            .collect(),
        positions,
        rule,
    )
}
struct GroundCreationValue<'a> {
    key: EquipmentLifeKey,
    timestamp_us: u64,
    position: [f32; 3],
    has_id: bool,
    id: u32,
    has_ammo: bool,
    ammo: &'a GroundWeaponAmmo,
}
fn assemble_ground_values<'a>(
    creations: impl Iterator<Item = GroundCreationValue<'a>>,
    tracks: impl Iterator<
        Item = (
            EquipmentLifeKey,
            super::ground_weapon_lifetimes::GroundMotion,
        ),
    >,
    keyframe_times: &[u64],
    sightings: BTreeMap<EquipmentLifeKey, &[u64]>,
    positions: &[ReplayPlayerPosition],
    rule: GroundObjectRule<'_>,
) -> GroundObjectAssembly {
    let mut out = GroundObjectAssembly {
        objects: Vec::new(),
        rejected: Default::default(),
    };
    let mut groups = BTreeMap::<EquipmentLifeKey, Vec<GroundCreationValue<'_>>>::new();
    let mut film_end = positions
        .iter()
        .map(|p| p.timestamp_us)
        .fold(keyframe_times.last().copied().unwrap_or(0), u64::max);
    for c in creations {
        film_end = film_end.max(c.timestamp_us);
        if c.has_id && rule.objectives.contains(&c.id) {
            out.rejected.total += 1;
            out.rejected.objectives += 1;
            continue;
        }
        if !c.has_id || !rule.families.contains_key(&c.id) {
            out.rejected.total += 1;
            continue;
        }
        groups.entry(c.key).or_default().push(c);
    }
    let lives = replay_player_lives(positions);
    let mut tracks_by_key =
        BTreeMap::<EquipmentLifeKey, Vec<super::ground_weapon_lifetimes::GroundMotion>>::new();
    for (key, track) in tracks {
        tracks_by_key.entry(key).or_default().push(track);
    }
    for (key, mut list) in groups {
        native_sort::sort_by(&mut list, |a, b| a.timestamp_us.cmp(&b.timestamp_us));
        let tracks = tracks_by_key.get(&key).map_or(&[][..], Vec::as_slice);
        let seen = sightings.get(&key).copied().unwrap_or_default();
        for (i, c) in list.iter().enumerate() {
            let life_end = list.get(i + 1).map_or(film_end, |c| c.timestamp_us);
            let r = super::ground_weapon_lifetimes::resolve_ground_values(
                c.timestamp_us,
                c.position,
                life_end,
                film_end,
                tracks.iter().copied(),
                super::ground_weapon_lifetimes::GroundEvidence {
                    keyframe_times,
                    seen,
                    positions,
                },
            );
            let dropper = ground_weapon_dropper(&lives, c.position, c.timestamp_us);
            let id = c.id;
            out.objects.push(GroundPickupObject {
                key,
                appearance: GroundObjectAppearance {
                    kind: rule.kind.into(),
                    family: rule.families[&id].clone(),
                    x: c.position[0],
                    y: c.position[1],
                    z: c.position[2],
                    timestamp_us: c.timestamp_us,
                    class: if dropper.is_some() {
                        "dropped"
                    } else {
                        "spawned"
                    }
                    .into(),
                    has_delta: r.moved,
                },
                family_id: id,
                position: r.position,
                moved: r.moved,
                bounds: r.bounds,
                picker: r.picker,
                status: r.status,
                dropper_slot: dropper.map_or(-1, i64::from),
                has_ammo: c.has_ammo,
                ammo: c.ammo.clone(),
            });
        }
    }
    native_sort::sort_by(&mut out.objects, object_order);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_ground_object_assembly() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/ground-objects-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let families = [
            (1, "family".into()),
            (2, "family".into()),
            (3, "family".into()),
        ]
        .into();
        let objectives = [2].into();
        for (i, row) in rows.as_array().unwrap().iter().enumerate() {
            let mut input = row["creations"].clone();
            for c in input.as_array_mut().unwrap() {
                if c["Mask"].is_null() {
                    c["Mask"] = serde_json::json!([]);
                }
            }
            let creations: Vec<EquipmentCreation> = serde_json::from_value(input).unwrap();
            let tracks: Vec<WorldObjectTrack> =
                serde_json::from_value(row["tracks"].clone()).unwrap();
            let kf: WorldObjectKeyframes =
                serde_json::from_value(row["keyframes"].clone()).unwrap();
            let positions: Vec<ReplayPlayerPosition> =
                serde_json::from_value(row["positions"].clone()).unwrap();
            let actual = assemble_ground_objects(
                &creations,
                &tracks,
                &kf,
                &positions,
                GroundObjectRule {
                    kind: "weapon",
                    families: &families,
                    objectives: &objectives,
                },
            );
            let mut expected: Vec<GroundPickupObject> =
                serde_json::from_value(row["objects"].clone()).unwrap();
            for (e, bits) in expected
                .iter_mut()
                .zip(row["distance_bits"].as_array().unwrap())
            {
                e.picker.distance_m = f64::from_bits(bits.as_u64().unwrap());
            }
            assert_eq!(actual.objects, expected, "objects {i}");
            let rejected: GroundObjectRejections =
                serde_json::from_value(row["rejected"].clone()).unwrap();
            assert_eq!(actual.rejected, rejected, "rejected {i}");
        }
    }
}
