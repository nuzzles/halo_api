//! Written placement-origin evidence and contemporaneous owner/heading selection.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplayEquipmentOriginIndex {
    spawns: BTreeMap<EquipmentLifeKey, Vec<u64>>,
    deaths: BTreeMap<u32, Vec<u64>>,
    taken: BTreeMap<u32, Vec<u64>>,
    pub spawn_events: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEquipmentOrigin {
    pub origin: String,
    pub cause: String,
    /// The native manifest-only spawned-piece fallback fired for this placement.
    pub manifest_fallback: bool,
}
impl ReplayEquipmentOriginIndex {
    pub fn new(
        spawns: &[EquipmentSpawnEvent],
        lives: &[IdentityLife],
        changes: &[EquipmentChange],
    ) -> Self {
        Self::from_values(
            spawns.iter().map(|s| {
                (
                    s.packet.timestamp_us,
                    s.spawned.map(|life| EquipmentLifeKey {
                        slot: life.slot,
                        generation: u32::from(life.generation),
                    }),
                )
            }),
            lives,
            changes
                .iter()
                .filter(|c| c.kind == EquipmentChangeKind::Taken)
                .map(|c| (c.slot, c.source.timestamp_us)),
        )
    }
    /// Index exact cached life generations and only explicitly recorded taken kinds.
    pub fn from_facts(
        spawns: &[FactsSpawnEvent],
        lives: &[IdentityLife],
        changes: &[FactsEquipmentChange],
    ) -> Self {
        Self::from_values(
            spawns.iter().map(|s| {
                (
                    s.timestamp_us,
                    s.spawned_valid.then_some(EquipmentLifeKey {
                        slot: s.spawned.slot,
                        generation: s.spawned.generation,
                    }),
                )
            }),
            lives,
            changes
                .iter()
                .filter(|c| c.kind == b"taken")
                .map(|c| (c.slot, c.timestamp_us)),
        )
    }
    fn from_values(
        spawns: impl ExactSizeIterator<Item = (u64, Option<EquipmentLifeKey>)>,
        lives: &[IdentityLife],
        changes: impl Iterator<Item = (u32, u64)>,
    ) -> Self {
        let mut out = Self {
            spawn_events: spawns.len(),
            ..Default::default()
        };
        for (timestamp, life) in spawns {
            if let Some(life) = life {
                out.spawns.entry(life).or_default().push(timestamp);
            }
        }
        for l in lives {
            if l.cause == "death" && l.to >= 0 {
                out.deaths.entry(l.slot).or_default().push(l.to as u64);
            }
        }
        for (slot, timestamp) in changes {
            out.taken.entry(slot).or_default().push(timestamp);
        }
        out
    }
    /// A matching spawn event wins even without an owner. Only a manifest-confirmed
    /// spawned piece may fall back to deployed; carried objects require written
    /// death/taken evidence, and simultaneous evidence is retained as a contradiction.
    pub fn classify(
        &self,
        p: &EquipmentPlacement,
        spawned_piece: bool,
        owner: Option<u32>,
    ) -> ReplayEquipmentOrigin {
        self.classify_values(p.life, p.t0_us, spawned_piece, owner)
    }
    pub fn classify_facts(
        &self,
        p: &FactsPlacement,
        spawned_piece: bool,
        owner: Option<u32>,
    ) -> ReplayEquipmentOrigin {
        self.classify_values(
            EquipmentLifeKey {
                slot: p.life.slot,
                generation: p.life.generation,
            },
            p.start_us,
            spawned_piece,
            owner,
        )
    }
    pub(super) fn classify_values(
        &self,
        life: EquipmentLifeKey,
        timestamp: u64,
        spawned_piece: bool,
        owner: Option<u32>,
    ) -> ReplayEquipmentOrigin {
        let (origin, cause) = if self.spawns.get(&life).is_some_and(|ts| {
            ts.iter()
                .any(|&t| t >= timestamp && t - timestamp <= 200_000)
        }) {
            ("deployed", "spawn_event")
        } else if spawned_piece {
            ("deployed", "manifest_piece")
        } else if let Some(slot) = owner {
            let near = |times: Option<&Vec<u64>>, tol| {
                times.is_some_and(|ts| ts.iter().any(|&t| t.abs_diff(timestamp) <= tol))
            };
            match (
                near(self.deaths.get(&slot), 200_000),
                near(self.taken.get(&slot), 50_000),
            ) {
                (true, true) => ("dropped", "both"),
                (true, false) => ("dropped", "death_written"),
                (false, true) => ("dropped", "taken_written"),
                (false, false) => ("unknown", "none"),
            }
        } else {
            ("unknown", "no_owner")
        };
        ReplayEquipmentOrigin {
            origin: origin.into(),
            cause: cause.into(),
            manifest_fallback: cause == "manifest_piece",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayEquipmentOwner {
    pub slot: u32,
    pub heading: Option<f32>,
}
/// Positions must be sorted by timestamp. Choose each slot's nearest time sample
/// within 250ms, then the nearest owner within 3m. Equal distance favors the lower
/// slot. Heading comes from that same slot within 200ms, independently of position.
pub fn replay_equipment_owner(
    positions: &[ReplayPositionSample],
    p: &EquipmentPlacement,
) -> Option<ReplayEquipmentOwner> {
    equipment_owner_values(positions, p.t0_us, [p.x, p.y, p.z])
}
pub(super) fn equipment_owner_values(
    positions: &[ReplayPositionSample],
    timestamp: u64,
    world: [f32; 3],
) -> Option<ReplayEquipmentOwner> {
    let lo =
        positions.partition_point(|s| s.position.timestamp_us.wrapping_add(250_000) < timestamp);
    let (mut best, mut aim) = (
        BTreeMap::<u32, &ReplayPositionSample>::new(),
        BTreeMap::<u32, &ReplayPositionSample>::new(),
    );
    for s in positions
        .iter()
        .skip(lo)
        .take_while(|s| s.position.timestamp_us <= timestamp.wrapping_add(250_000))
    {
        let pos = &s.position;
        if !pos.has_world {
            continue;
        }
        let closer = |b: &&ReplayPositionSample| {
            pos.timestamp_us.abs_diff(timestamp) < b.position.timestamp_us.abs_diff(timestamp)
        };
        if best.get(&pos.slot).is_none_or(closer) {
            best.insert(pos.slot, s);
        }
        if s.aim.is_some()
            && pos.timestamp_us.abs_diff(timestamp) <= 200_000
            && aim.get(&pos.slot).is_none_or(closer)
        {
            aim.insert(pos.slot, s);
        }
    }
    let mut near: Option<(u32, f32)> = None;
    for (slot, s) in best {
        let dx = f64::from(world[0] - s.position.x);
        let dy = f64::from(world[1] - s.position.y);
        let dz = f64::from(world[2] - s.position.z);
        let distance = dz.mul_add(dz, dx.mul_add(dx, dy * dy)).sqrt() as f32;
        if distance > 3. {
            continue;
        }
        if near.is_none_or(|(other, d)| distance < d || (distance == d && slot < other)) {
            near = Some((slot, distance));
        }
    }
    let (slot, _) = near?;
    let heading = aim
        .get(&slot)
        .and_then(|s| s.aim)
        .map(|a| (360. * (f64::from(a[0]) + 0.5) / 4096.) as f32);
    Some(ReplayEquipmentOwner { slot, heading })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Spawn {
        slot: u32,
        generation: u8,
        time: u64,
        valid: bool,
    }
    #[derive(Deserialize)]
    struct Change {
        slot: u32,
        time: u64,
        kind: EquipmentChangeKind,
    }
    #[derive(Deserialize)]
    struct Probe {
        placement: EquipmentPlacement,
        piece: bool,
        owner: Option<ReplayEquipmentOwner>,
        origin: ReplayEquipmentOrigin,
    }
    #[derive(Deserialize)]
    struct Case {
        spawns: Vec<Spawn>,
        lives: Vec<IdentityLife>,
        changes: Vec<Change>,
        positions: Vec<ReplayPositionSample>,
        probes: Vec<Probe>,
        events: usize,
        raw: Vec<EquipmentPlacement>,
        census: WorldObjectKeyframes,
        clock: IdentityClock,
        ends: Vec<ReplayEquipmentEnd>,
        publication: ReplayEquipmentPlacements,
        fallback_report: serde_json::Value,
    }
    #[test]
    fn native_equipment_owner_and_origin() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-equipment-origin-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
        let packet = |timestamp_us| FilmPacket {
            chunk_index: 1,
            packet_type: 0,
            byte_2: 0,
            byte_3: 0,
            payload_offset: 0,
            payload_size: 0,
            timestamp_us,
        };
        for (i, c) in cases.into_iter().enumerate() {
            let spawns: Vec<_> = c
                .spawns
                .into_iter()
                .map(|s| EquipmentSpawnEvent {
                    packet_index: None,
                    packet: packet(s.time),
                    source: None,
                    spawned: s.valid.then_some(ObjectLife {
                        slot: s.slot,
                        generation: s.generation,
                    }),
                    reference_2_present: false,
                })
                .collect();
            let changes: Vec<_> = c
                .changes
                .into_iter()
                .map(|c| EquipmentChange {
                    chunk_number: None,
                    packet_index: None,
                    source: packet(c.time),
                    slot: c.slot,
                    counter: 0,
                    rank: None,
                    previous: None,
                    kind: c.kind,
                    recovered: false,
                    gap: 0,
                })
                .collect();
            if let Some(step) = std::num::NonZeroU64::new(c.clock.step_us) {
                assert_eq!(
                    replay_equipment_ends(
                        &c.raw,
                        &c.census,
                        c.clock.origin_us,
                        step,
                        c.clock.frame_count
                    ),
                    c.ends,
                    "placement ends {i}"
                );
            }
            let stats = EquipmentPlacementStats {
                scanned: i % 3 != 0,
                calibration: EquipmentMppCalibration {
                    widths: EquipmentMppWidths {
                        lead: i % 6,
                        index: i % 9,
                    },
                    ..Default::default()
                },
                lives: i % 23,
                anchors: i % 17,
                confirmed: i % 31,
                ..Default::default()
            };
            let families = [(1, "grapple".into()), (0x528fce46, "wall".into())].into();
            let pieces = [0x528fce46, 0x686b40c9].into();
            let publication = build_replay_equipment_placements(
                ReplayEquipmentPlacementInput {
                    raw: &c.raw,
                    stats: &stats,
                    positions: &c.positions,
                    census: &c.census,
                    spawns: &spawns,
                    spawn_lists: i % 11,
                    lives: &c.lives,
                    changes: &changes,
                    families: &families,
                    spawned_piece_ids: &pieces,
                },
                c.clock,
            );
            assert_eq!(publication, c.publication, "placement publication {i}");
            // Compare the actual native counter; coverage ByCause alone does
            // not prove that the named fallback was triggered.
            let expected_report = if publication.manifest_fallbacks == 0 {
                serde_json::json!([])
            } else {
                serde_json::json!([{
                    "Nom": "repli_piece_engendree_sans_evenement",
                    "Declenchements": publication.manifest_fallbacks,
                }])
            };
            assert_eq!(
                expected_report, c.fallback_report,
                "native fallback counter {i}"
            );
            let source = ReplayEquipmentOriginIndex::new(&spawns, &c.lives, &changes);
            assert_eq!(source.spawn_events, c.events, "events {i}");
            for (j, p) in c.probes.into_iter().enumerate() {
                let owner = replay_equipment_owner(&c.positions, &p.placement);
                assert_eq!(owner, p.owner, "owner {i}/{j}");
                assert_eq!(
                    source.classify(&p.placement, p.piece, owner.map(|o| o.slot)),
                    p.origin,
                    "origin {i}/{j}"
                );
            }
        }
    }
}
