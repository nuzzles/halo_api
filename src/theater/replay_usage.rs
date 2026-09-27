//! Native document-to-equipment-usage projection.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const USAGE_SUMMARY_REV: &str = "us6";
pub const USAGE_FAMILY_WALL_KEY: &str = "wall";
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsagePlayerSummary {
    #[serde(rename = "XUID")]
    pub xuid: String,
    #[serde(rename = "GrapplePulls")]
    pub grapple_pulls: i64,
    #[serde(rename = "CamoEpisodes")]
    pub camo_episodes: i64,
    #[serde(rename = "CamoMS")]
    pub camo_ms: i64,
    #[serde(rename = "CamoKills")]
    pub camo_kills: i64,
    #[serde(rename = "OvershieldEpisodes")]
    pub overshield_episodes: i64,
    #[serde(rename = "OvershieldMS")]
    pub overshield_ms: i64,
    #[serde(rename = "OvershieldKills")]
    pub overshield_kills: i64,
    #[serde(rename = "DeployedByFamily")]
    pub deployed_by_family: Option<BTreeMap<String, i64>>,
    #[serde(rename = "DroppedObjects")]
    pub dropped_objects: i64,
    #[serde(rename = "DroppedByFamily")]
    pub dropped_by_family: Option<BTreeMap<String, i64>>,
    #[serde(rename = "TakenByFamily")]
    pub taken_by_family: Option<BTreeMap<String, i64>>,
    #[serde(rename = "SpentByFamily")]
    pub spent_by_family: Option<BTreeMap<String, i64>>,
    #[serde(rename = "KeptByFamily")]
    pub kept_by_family: Option<BTreeMap<String, i64>>,
    #[serde(rename = "GrenadesThrown")]
    pub grenades_thrown: i64,
    #[serde(rename = "PadPickups")]
    pub pad_pickups: i64,
    #[serde(rename = "PadPickupsByWeapon")]
    pub pad_pickups_by_weapon: Option<BTreeMap<String, i64>>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageWeaponPad {
    #[serde(rename = "weapon")]
    pub weapon: String,
    #[serde(rename = "occupations")]
    pub occupations: i64,
    #[serde(rename = "named")]
    pub named: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageMatchSummary {
    #[serde(rename = "SchemaVersion")]
    pub schema_version: i64,
    #[serde(rename = "FrameIntervalMS")]
    pub frame_interval_ms: i64,
    #[serde(rename = "FrameCount")]
    pub frame_count: i64,
    #[serde(rename = "DurationMS")]
    pub duration_ms: i64,
    #[serde(rename = "PadOccupancies")]
    pub pad_occupancies: i64,
    #[serde(rename = "PadNamed")]
    pub pad_named: i64,
    #[serde(rename = "PadUnnamed")]
    pub pad_unnamed: i64,
    #[serde(rename = "PowerupPadPickups")]
    pub powerup_pad_pickups: Option<BTreeMap<String, i64>>,
    #[serde(rename = "WeaponPads")]
    pub weapon_pads: Vec<UsageWeaponPad>,
    #[serde(rename = "EquipmentChanges")]
    pub equipment_changes: UsageChangeCoverage,
    #[serde(rename = "Fallbacks")]
    pub fallbacks: Vec<FallbackTrigger>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageSummary {
    #[serde(rename = "Match")]
    pub match_summary: UsageMatchSummary,
    #[serde(rename = "Players")]
    pub players: Vec<UsagePlayerSummary>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UsageChangeCoverage {
    pub unnamed_rank_taken: i64,
    pub unattributed_slot: i64,
    pub spent_unreliable_from: i64,
}
impl UsageChangeCoverage {
    pub fn total(&self) -> i64 {
        self.unnamed_rank_taken + self.unattributed_slot + self.spent_unreliable_from
    }
}
const OUTCOME_FAMILIES: [&str; 8] = [
    "wall",
    "sensor",
    "translocator_beacon",
    "shroud_screen",
    "threat_seeker",
    "repair_field",
    "powerup_camo",
    "powerup_overshield",
];
fn bump(m: &mut Option<BTreeMap<String, i64>>, key: &str) {
    *m.get_or_insert_default().entry(key.into()).or_default() += 1;
}
fn count(m: &Option<BTreeMap<String, i64>>, key: &str) -> i64 {
    m.as_ref()
        .and_then(|m| m.get(key))
        .copied()
        .unwrap_or_default()
}
fn player<'a>(
    players: &'a mut BTreeMap<String, UsagePlayerSummary>,
    xuid: &str,
) -> Option<&'a mut UsagePlayerSummary> {
    if xuid.is_empty() {
        return None;
    }
    Some(
        players
            .entry(xuid.into())
            .or_insert_with(|| UsagePlayerSummary {
                xuid: xuid.into(),
                ..Default::default()
            }),
    )
}
#[derive(Default)]
struct UsageOwners {
    lives: BTreeMap<u32, Vec<(i64, i64, String)>>,
    last: BTreeMap<u32, String>,
}
impl UsageOwners {
    fn new(doc: &ReplayDocumentContent) -> Self {
        let mut order = Vec::<(ReplayByteString, String, Vec<&ReplayTrack>)>::new();
        let mut anonymous = Vec::new();
        let add = |order: &mut Vec<(ReplayByteString, String, Vec<&ReplayTrack>)>,
                   key: ReplayByteString,
                   xuid: String| {
            if let Some(i) = order.iter().position(|j| j.0 == key) {
                i
            } else {
                order.push((key, xuid, Vec::new()));
                order.len() - 1
            }
        };
        for e in &doc.roster {
            if !e.xuid.is_empty() {
                add(&mut order, e.xuid.clone().into(), e.xuid.clone());
            } else if e.bot && !e.name.is_empty() {
                add(
                    &mut order,
                    ReplayByteString([b"bot:".as_slice(), e.name.as_ref()].concat()),
                    String::new(),
                );
            }
        }
        for t in doc.tracks.as_deref().unwrap_or_default() {
            let i = if !t.xuid.is_empty() {
                add(&mut order, t.xuid.clone().into(), t.xuid.clone())
            } else if !t.bot.is_empty() {
                add(
                    &mut order,
                    ReplayByteString([b"bot:".as_slice(), t.bot.as_ref()].concat()),
                    String::new(),
                )
            } else {
                anonymous.push(t);
                continue;
            };
            order[i].2.push(t);
        }
        let mut out = Self::default();
        for (_, xuid, mut tracks) in order {
            tracks.sort_by_key(|t| t.start_frame);
            for t in tracks {
                out.lives.entry(t.slot).or_default().push((
                    t.start_frame,
                    t.end_frame,
                    xuid.clone(),
                ));
                out.last.insert(t.slot, xuid.clone());
            }
        }
        for t in anonymous {
            out.lives
                .entry(t.slot)
                .or_default()
                .push((t.start_frame, t.end_frame, String::new()));
        }
        for lives in out.lives.values_mut() {
            lives.sort_by_key(|v| v.0);
        }
        out
    }
    fn last(&self, slot: u32, fb: &FallbackCounter) -> &str {
        let x = self.last.get(&slot).map_or("", String::as_str);
        if !x.is_empty() {
            fb.trigger("repli_geste_dernier_occupant_du_match");
        }
        x
    }
    fn at(&self, slot: u32, frame: i64, before: bool, fb: &FallbackCounter) -> &str {
        let lives = self.lives.get(&slot).map_or(&[][..], Vec::as_slice);
        if !before {
            for (from, to, x) in lives {
                if frame >= *from && frame <= *to {
                    return x;
                }
            }
            return self.last(slot, fb);
        }
        let mut last = None;
        for (from, to, x) in lives {
            if frame < *from {
                break;
            }
            if frame <= *to {
                return x;
            }
            last = Some(x.as_str());
        }
        if let Some(x) = last {
            return x;
        }
        if let Some((_, _, x)) = lives.first() {
            if !x.is_empty() {
                fb.trigger(super::FIRST_SLOT_LIFE_FALLBACK);
            }
            return x;
        }
        self.last(slot, fb)
    }
}
pub fn usage_family_is_droppable(family: &str) -> bool {
    !matches!(
        family,
        "grenade_frag" | "grenade_plasma" | "grenade_dynamo" | "grenade_spike"
    )
}
pub fn usage_family_is_deployable(family: &str) -> bool {
    usage_family_is_droppable(family)
        && !matches!(
            family,
            "grapple" | "thruster" | "repulsor" | "powerup_camo" | "powerup_overshield"
        )
}
pub fn usage_equipment_is_spawned_piece(id: &str) -> bool {
    matches!(id, "0x528fce46" | "0x686b40c9")
}
pub fn usage_deployed_counts(p: &ReplayEquipmentPlacement) -> bool {
    p.origin == "deployed"
        && usage_family_is_deployable(&p.family)
        && (p.family != "wall" || usage_equipment_is_spawned_piece(&p.id))
}
pub fn usage_equipment_used(player: &UsagePlayerSummary, family: &str) -> i64 {
    match family {
        "powerup_camo" => player.camo_episodes,
        "powerup_overshield" => player.overshield_episodes,
        "wall" => count(&player.deployed_by_family, family),
        _ => count(&player.spent_by_family, family),
    }
}
/// Project a published document. Projection fallback counts are independent of
/// the decode's coverage: these decisions occur after document assembly.
pub fn build_usage_summary(document: &ReplayDocument) -> UsageSummary {
    let doc = &document.content;
    let fb = FallbackCounter::default();
    let owners = UsageOwners::new(doc);
    let mut players = BTreeMap::new();
    let mut out = UsageSummary {
        match_summary: UsageMatchSummary {
            schema_version: doc.schema_version,
            frame_interval_ms: doc.frame_interval_ms,
            frame_count: doc.frame_count,
            duration_ms: doc.duration_ms,
            ..Default::default()
        },
        players: Vec::new(),
    };
    for g in &doc.grapple_lines {
        if let Some(p) = player(&mut players, owners.at(g.slot, g.t0, false, &fb)) {
            p.grapple_pulls += 1;
        }
    }
    for e in &doc.equipment_episodes {
        let Some(p) = player(&mut players, owners.at(e.slot, e.t0, false, &fb)) else {
            continue;
        };
        let ms = if doc.frame_interval_ms > 0 && e.t1 > e.t0 {
            e.t1.wrapping_sub(e.t0).wrapping_mul(doc.frame_interval_ms)
        } else {
            0
        };
        match e.fam.as_str() {
            "camo" => {
                p.camo_episodes += 1;
                p.camo_ms = p.camo_ms.wrapping_add(ms);
                p.camo_kills += e.k as i64;
            }
            "overshield" => {
                p.overshield_episodes += 1;
                p.overshield_ms = p.overshield_ms.wrapping_add(ms);
                p.overshield_kills += e.k as i64;
            }
            _ => {}
        }
    }
    for e in &doc.equipment_placements {
        if e.owner < 0 {
            continue;
        }
        let Some(p) = player(&mut players, owners.at(e.owner as u32, e.t0, true, &fb)) else {
            continue;
        };
        if usage_deployed_counts(e) {
            bump(&mut p.deployed_by_family, &e.family);
        } else if e.origin == "dropped" && usage_family_is_droppable(&e.family) {
            p.dropped_objects += 1;
            bump(&mut p.dropped_by_family, &e.family);
        }
    }
    let with_life: BTreeSet<_> = owners
        .lives
        .values()
        .flatten()
        .filter(|v| !v.2.is_empty())
        .map(|v| v.2.as_str())
        .collect();
    let mut index = BTreeMap::new();
    for r in &doc.roster {
        if !r.xuid.is_empty() && with_life.contains(r.xuid.as_str()) {
            index.insert(r.film_index, r.xuid.as_str());
        }
    }
    for g in &doc.grenades {
        if let Some(p) = player(
            &mut players,
            index.get(&g.film_index).copied().unwrap_or_default(),
        ) {
            p.grenades_thrown += 1;
        }
    }
    let m = &mut out.match_summary;
    let mut pads = BTreeMap::new();
    for (i, p) in doc.weapon_pads.iter().enumerate() {
        if let Some(key) = pad_weapon_family_key(&p.weapon) {
            pads.insert(i, m.weapon_pads.len());
            m.weapon_pads.push(UsageWeaponPad {
                weapon: key,
                ..Default::default()
            });
        }
    }
    m.pad_occupancies = doc.pad_pickups.len() as i64;
    for pick in &doc.pad_pickups {
        let Some(pad) = usize::try_from(pick.pad)
            .ok()
            .and_then(|i| doc.weapon_pads.get(i))
        else {
            continue;
        };
        let Some(&j) = pads.get(&(pick.pad as usize)) else {
            bump(&mut m.powerup_pad_pickups, &pad.weapon);
            continue;
        };
        m.weapon_pads[j].occupations += 1;
        let Some(xuid) = pick.xuid.as_deref().filter(|x| !x.is_empty()) else {
            m.pad_unnamed += 1;
            continue;
        };
        m.weapon_pads[j].named += 1;
        m.pad_named += 1;
        let p = player(&mut players, xuid).unwrap();
        p.pad_pickups += 1;
        bump(&mut p.pad_pickups_by_weapon, &m.weapon_pads[j].weapon);
    }
    for e in &doc.equipment_changes {
        let Some(p) = player(&mut players, owners.at(e.slot, e.t, false, &fb)) else {
            m.equipment_changes.unattributed_slot += 1;
            continue;
        };
        match e.kind {
            EquipmentChangeKind::Taken => {
                let Some(label) = doc.ability_labels.get(&e.r.to_string()) else {
                    m.equipment_changes.unnamed_rank_taken += 1;
                    continue;
                };
                if OUTCOME_FAMILIES.contains(&label.family.as_str()) {
                    bump(&mut p.taken_by_family, &label.family);
                }
            }
            EquipmentChangeKind::Spent => {
                if e.gap > 0 {
                    m.equipment_changes.spent_unreliable_from += 1;
                    continue;
                }
                if let Some(label) = doc.ability_labels.get(&e.from.to_string())
                    && OUTCOME_FAMILIES.contains(&label.family.as_str())
                {
                    bump(&mut p.spent_by_family, &label.family);
                }
            }
            _ => {}
        }
    }
    for p in players.values_mut() {
        for family in OUTCOME_FAMILIES {
            let taken = count(&p.taken_by_family, family);
            if taken == 0 {
                continue;
            }
            let mut kept = taken
                .wrapping_sub(usage_equipment_used(p, family))
                .wrapping_sub(count(&p.dropped_by_family, family));
            if kept < 0 {
                fb.trigger(super::NEGATIVE_EQUIPMENT_KEPT_FALLBACK);
                kept = 0;
            }
            p.kept_by_family
                .get_or_insert_default()
                .insert(family.into(), kept);
        }
    }
    out.players = players.into_values().collect();
    m.fallbacks = fb.report();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_usage_summary() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/usage-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let doc: ReplayDocument = serde_json::from_value(row["doc"].clone()).unwrap();
            assert_eq!(
                serde_json::to_value(build_usage_summary(&doc)).unwrap(),
                row["summary"],
                "usage {i}"
            );
        }
    }
}
