//! Inventory publication and death corroboration on the shared replay clock.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayAmmoSlot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mag: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub res: Option<u32>,
    /// Consumed fraction; absent is distinct from measured zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gauge: Option<f32>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayInventory {
    pub t: i64,
    pub slot: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub g: Vec<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gs: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub d: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub am: Vec<ReplayAmmoSlot>,
    #[serde(default, skip_serializing_if = "inventory_zero")]
    pub cand: i64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub empty: String,
}
fn inventory_zero(v: &i64) -> bool {
    *v == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayInventoryCoverage {
    pub decoded: usize,
    pub dropped_before_origin: usize,
    pub unpublished: usize,
    pub published: usize,
}
/// Retain every post-origin read, including empty reads and equal-frame ties.
/// Only the first two ammo slots describe carried weapons.
pub fn build_replay_inventory(
    raw: &[KeyframeInventory],
    origin: u64,
    step: NonZeroU64,
) -> (Vec<ReplayInventory>, usize) {
    build_inventory_values(
        raw.iter().map(|r| InventoryInput {
            timestamp_us: r.timestamp_us,
            slot: r.slot,
            grenades_read: r.grenades_read,
            grenades: &r.grenades,
            ammo_read: r.ammo_read,
            ammo: &r.ammo,
            selected_grenade_rank: i64::from(r.selected_grenade_rank),
            drawn_slot: i64::from(r.drawn_slot),
            ammo_candidates: r.ammo_candidates as i64,
        }),
        origin,
        step,
    )
}
/// Publish exact native cache values, including signed candidate counts.
pub fn build_facts_replay_inventory(
    raw: &[FactsKeyframeInventory],
    origin: u64,
    step: NonZeroU64,
) -> (Vec<ReplayInventory>, usize) {
    build_inventory_values(
        raw.iter().map(|r| InventoryInput {
            timestamp_us: r.timestamp_us,
            slot: r.slot,
            grenades_read: r.grenades_read,
            grenades: &r.grenades,
            ammo_read: r.ammo_read,
            ammo: &r.ammo,
            selected_grenade_rank: r.selected_grenade_rank,
            drawn_slot: r.drawn_slot,
            ammo_candidates: r.ammo_candidates,
        }),
        origin,
        step,
    )
}
struct InventoryInput<'a> {
    timestamp_us: u64,
    slot: u32,
    grenades_read: bool,
    grenades: &'a [u32; 4],
    selected_grenade_rank: i64,
    drawn_slot: i64,
    ammo_candidates: i64,
    ammo_read: bool,
    ammo: &'a [KeyframeSlotAmmo; 4],
}
fn build_inventory_values<'a>(
    raw: impl Iterator<Item = InventoryInput<'a>>,
    origin: u64,
    step: NonZeroU64,
) -> (Vec<ReplayInventory>, usize) {
    let mut out = Vec::new();
    let mut dropped = 0;
    for r in raw {
        if r.timestamp_us < origin {
            dropped += 1;
            continue;
        }
        out.push(ReplayInventory {
            t: ((r.timestamp_us - origin) / step.get()) as i64,
            slot: r.slot,
            g: if r.grenades_read {
                r.grenades.to_vec()
            } else {
                Vec::new()
            },
            gs: (r.selected_grenade_rank >= 0).then_some(r.selected_grenade_rank),
            d: (r.drawn_slot >= 0).then_some(r.drawn_slot),
            am: if r.ammo_read {
                r.ammo[..2]
                    .iter()
                    .map(|a| ReplayAmmoSlot {
                        mag: a.mag,
                        res: a.res,
                        gauge: a.gauge.map(|g| g as f32),
                    })
                    .collect()
            } else {
                Vec::new()
            },
            cand: r.ammo_candidates,
            empty: if !r.grenades_read && !r.ammo_read {
                "unknown".into()
            } else {
                String::new()
            },
        });
    }
    out.sort_by_key(|r| (r.t, r.slot));
    (out, dropped)
}
/// A frame before zero or a zero clock step cannot name an occupant.
pub fn replay_occupant_at_frame(
    identity: &ReplayIdentityState,
    clock: IdentityClock,
    slot: u32,
    frame: i64,
) -> u64 {
    if frame < 0 || clock.step_us == 0 {
        return 0;
    }
    identity.xuid_at(
        slot,
        clock
            .origin_us
            .wrapping_add((frame as u64).wrapping_mul(clock.step_us)),
    )
}
/// Corroborate only unknown empty reads within the native inclusive eight-second
/// window after the most recent death of their occupant at that frame.
pub fn mark_replay_inventory_dead(
    reads: &mut [ReplayInventory],
    deaths: &[IdentityDeath],
    identity: &ReplayIdentityState,
    offset_ms: i64,
    clock: IdentityClock,
) -> usize {
    if reads.is_empty() || deaths.is_empty() || !identity.bridge_established() {
        return 0;
    }
    let mut by_victim = BTreeMap::<u64, Vec<i64>>::new();
    for d in deaths {
        by_victim
            .entry(d.xuid)
            .or_default()
            .push(d.time_ms.wrapping_add(offset_ms));
    }
    for times in by_victim.values_mut() {
        times.sort_unstable();
    }
    let mut marked = 0;
    for r in reads {
        if r.empty != "unknown" {
            continue;
        }
        let xuid = replay_occupant_at_frame(identity, clock, r.slot, r.t);
        if xuid == 0 {
            continue;
        }
        let Some(times) = by_victim.get(&xuid) else {
            continue;
        };
        let time = (clock
            .origin_us
            .wrapping_add((r.t as u64).wrapping_mul(clock.step_us))
            / 1000) as i64;
        let i = times.partition_point(|&t| t <= time);
        if i > 0 && time.wrapping_sub(times[i - 1]) <= 8000 {
            r.empty = "dead".into();
            marked += 1;
        }
    }
    marked
}

/// Native diagnostic after death attribution. Silence means no empty reads;
/// it does not mean that all empty reads were explained by a recorded death.
pub fn log_replay_inventory_empty(reads: &[ReplayInventory], marked: usize) {
    let empty = reads.iter().filter(|r| !r.empty.is_empty()).count();
    if empty == 0 {
        return;
    }
    tracing::info!(
        lectures = reads.len(),
        vides = empty,
        morts = marked,
        inexpliquees = (empty as i64).wrapping_sub(marked as i64),
        "rejeu : lectures d'inventaire vides"
    );
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilmReplayInventory {
    pub reads: Vec<ReplayInventory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<ReplayInventoryCoverage>,
    pub marked_dead: usize,
}
/// Missing scanner input leaves coverage absent; a successful empty scan publishes
/// zero counts. Reuse the player layer's clock, deaths and temporal identity.
#[allow(dead_code)]
pub(crate) fn build_film_replay_inventory(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> Option<FilmReplayInventory> {
    let step = NonZeroU64::new(players.clock.step_us)?;
    let Some(stream) = film
        .keyframe_inventory
        .as_ref()
        .filter(|_| film.keyframe_inventory_error.is_none())
    else {
        return Some(FilmReplayInventory::default());
    };
    let (reads, dropped_before_origin) =
        build_replay_inventory(&stream.records, players.clock.origin_us, step);
    Some(publish_player_inventory(
        reads,
        stream.records.len(),
        dropped_before_origin,
        players,
    ))
}
/// Inventory from decoded cache inputs. Native cache decoding allocates a
/// non-nil inventory slice even for zero records, so zero coverage is published.
pub fn build_facts_player_inventory(
    inventory: &FactsInventory,
    players: &FilmReplayPlayers,
) -> Option<FilmReplayInventory> {
    let step = NonZeroU64::new(players.clock.step_us)?;
    let (reads, dropped) =
        build_facts_replay_inventory(&inventory.inventory, players.clock.origin_us, step);
    Some(publish_player_inventory(
        reads,
        inventory.inventory.len(),
        dropped,
        players,
    ))
}
fn publish_player_inventory(
    mut reads: Vec<ReplayInventory>,
    decoded: usize,
    dropped_before_origin: usize,
    players: &FilmReplayPlayers,
) -> FilmReplayInventory {
    let built = reads.len();
    let slots: BTreeSet<_> = players
        .players
        .publication
        .tracks
        .iter()
        .map(|t| t.slot)
        .collect();
    reads.retain(|r| slots.contains(&r.slot));
    let marked_dead = mark_replay_inventory_dead(
        &mut reads,
        &players.evidence.deaths,
        &players.registry.owners.state,
        players.registry.owners.clock.offset_ms,
        players.clock,
    );
    log_replay_inventory_empty(&reads, marked_dead);
    let coverage = Some(ReplayInventoryCoverage {
        decoded,
        dropped_before_origin,
        unpublished: built - reads.len(),
        published: reads.len(),
    });
    FilmReplayInventory {
        reads,
        coverage,
        marked_dead,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        logs: Vec<serde_json::Value>,
        raw: Vec<KeyframeInventory>,
        origin: u64,
        step: u64,
        lives: Vec<IdentityLife>,
        indices: BTreeMap<u64, i64>,
        offset: i64,
        deaths: Vec<IdentityDeath>,
        slots: BTreeSet<u32>,
        before: Vec<ReplayInventory>,
        reads: Vec<ReplayInventory>,
        coverage: ReplayInventoryCoverage,
        marked: usize,
    }
    #[test]
    fn native_inventory_publication_and_death_attribution() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-inventory-publication-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let (mut reads, dropped) =
                build_replay_inventory(&c.raw, c.origin, NonZeroU64::new(c.step).unwrap());
            let built = reads.len();
            reads.retain(|r| c.slots.contains(&r.slot));
            assert_eq!(reads, c.before, "projection {i}");
            assert_eq!(
                ReplayInventoryCoverage {
                    decoded: c.raw.len(),
                    dropped_before_origin: dropped,
                    unpublished: built - reads.len(),
                    published: reads.len()
                },
                c.coverage,
                "coverage {i}"
            );
            let identity = ReplayIdentityState::from_lives(c.lives, &c.indices);
            let marked = mark_replay_inventory_dead(
                &mut reads,
                &c.deaths,
                &identity,
                c.offset,
                IdentityClock {
                    origin_us: c.origin,
                    step_us: c.step,
                    frame_count: 30000,
                },
            );
            assert_eq!(marked, c.marked, "marked {i}");
            assert_eq!(
                crate::theater::log_test_support::capture_logs(|| log_replay_inventory_empty(
                    &reads, marked
                )),
                c.logs,
                "logs {i}"
            );
            assert_eq!(reads, c.reads, "attribution {i}");
        }
    }
}
