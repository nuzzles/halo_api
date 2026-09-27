//! Publish equipment transitions while retaining the native scanner coverage.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEquipmentChange {
    pub t: i64,
    pub slot: u32,
    pub kind: EquipmentChangeKind,
    pub r: i64,
    pub from: i64,
    #[serde(default, skip_serializing_if = "equipment_false")]
    pub recovered: bool,
    #[serde(default, skip_serializing_if = "equipment_zero")]
    pub gap: i64,
}
fn equipment_false(v: &bool) -> bool {
    !v
}
fn equipment_zero(v: &i64) -> bool {
    *v == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayEquipmentChangeCoverage {
    pub decoded: usize,
    pub published: usize,
    pub taken: usize,
    pub spent: usize,
    pub spawned: usize,
    pub before_origin: usize,
    pub lives: i64,
    pub missed_estimate: i64,
    pub counter_jumps: i64,
    pub lives_first_off_spec: i64,
    pub repeats: i64,
    pub recovered: i64,
}
impl ReplayEquipmentChangeCoverage {
    /// Emit native coverage counted before track filtering changes the rendered list.
    /// Recovered and missed counts describe decoder coverage, not extra actions.
    pub fn log(&self) {
        tracing::info!(
            decodes = self.decoded,
            publies = self.published,
            ramassages = self.taken,
            consommations = self.spent,
            reapparitions = self.spawned,
            avantOrigine = self.before_origin,
            vies = self.lives,
            manqueesEstimees = self.missed_estimate,
            sautsCompteur = self.counter_jumps,
            premiereHorsNorme = self.lives_first_off_spec,
            repetitions = self.repeats,
            recuperees = self.recovered,
            "rejeu : ramassages et consommations d equipement"
        );
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEquipmentChanges {
    pub changes: Vec<ReplayEquipmentChange>,
    pub coverage: ReplayEquipmentChangeCoverage,
}
/// Birth announcements are excluded before the origin check. Output preserves
/// scan order. Native coverage counts publication before the track-slot filter.
pub fn build_replay_equipment_changes(
    raw: &EquipmentChanges,
    origin: u64,
    step: u64,
    published_slots: &BTreeSet<u32>,
) -> ReplayEquipmentChanges {
    let s = &raw.stats;
    let coverage = ReplayEquipmentChangeCoverage {
        decoded: raw.records.len(),
        lives: s.lives as i64,
        missed_estimate: s.missed_estimate as i64,
        counter_jumps: s.counter_jumps as i64,
        lives_first_off_spec: s.lives_first_off_spec as i64,
        repeats: s.repeats as i64,
        recovered: s.recovered as i64,
        ..Default::default()
    };
    let (selected, coverage) = select_equipment_changes(
        raw.records.iter().map(|c| {
            (
                c.source.timestamp_us,
                c.slot,
                match c.kind {
                    EquipmentChangeKind::Taken => &b"taken"[..],
                    EquipmentChangeKind::Spent => &b"spent"[..],
                    EquipmentChangeKind::Spawned => &b"spawned"[..],
                },
            )
        }),
        coverage,
        origin,
        step,
        published_slots,
    );
    ReplayEquipmentChanges {
        changes: selected
            .into_iter()
            .map(|(i, t)| {
                let c = &raw.records[i];
                ReplayEquipmentChange {
                    t,
                    slot: c.slot,
                    kind: c.kind,
                    r: c.rank.map_or(-1, i64::from),
                    from: c.previous.map_or(-1, i64::from),
                    recovered: c.recovered,
                    gap: i64::from(c.gap),
                }
            })
            .collect(),
        coverage,
    }
}
/// Return indices into the caller's original records, keeping their exact kind
/// and payload representation outside this shared selection calculation.
pub(super) fn select_equipment_changes<'a>(
    raw: impl Iterator<Item = (u64, u32, &'a [u8])>,
    mut coverage: ReplayEquipmentChangeCoverage,
    origin: u64,
    step: u64,
    published_slots: &BTreeSet<u32>,
) -> (Vec<(usize, i64)>, ReplayEquipmentChangeCoverage) {
    let mut selected = Vec::new();
    if step == 0 {
        return (selected, coverage);
    }
    for (i, (time, slot, kind)) in raw.enumerate() {
        if kind == b"spawned" {
            coverage.spawned += 1;
            continue;
        }
        if time < origin {
            coverage.before_origin += 1;
            continue;
        }
        coverage.published += 1;
        if kind == b"spent" {
            coverage.spent += 1;
        } else {
            coverage.taken += 1;
        }
        if published_slots.contains(&slot) {
            selected.push((i, ((time - origin) / step) as i64));
        }
    }
    (selected, coverage)
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_equipment_changes(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> ReplayEquipmentChanges {
    let slots = players
        .players
        .publication
        .tracks
        .iter()
        .map(|t| t.slot)
        .collect();
    let empty = EquipmentChanges::default();
    let raw = film
        .equipment_changes
        .as_ref()
        .map_or(&empty, |s| &s.assembly);
    build_replay_equipment_changes(raw, players.clock.origin_us, players.clock.step_us, &slots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Raw {
        #[serde(rename = "TimestampUS")]
        time: u64,
        slot: u32,
        rank: i64,
        previous: i64,
        kind: EquipmentChangeKind,
        recovered: bool,
        gap: u8,
    }
    #[derive(Deserialize)]
    struct Case {
        log: serde_json::Value,
        raw: Vec<Raw>,
        stats: serde_json::Value,
        origin: u64,
        step: u64,
        slots: BTreeSet<u32>,
        output: ReplayEquipmentChanges,
    }
    #[test]
    fn native_equipment_change_publication() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-equipment-publication-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                super::super::log_test_support::capture_log(|| c.output.coverage.log()),
                c.log,
                "log {i}"
            );
            let stat = |name: &str| c.stats[name].as_u64().unwrap() as usize;
            let raw = EquipmentChanges {
                records: c
                    .raw
                    .into_iter()
                    .map(|r| EquipmentChange {
                        chunk_number: None,
                        packet_index: None,
                        source: FilmPacket {
                            chunk_index: 1,
                            packet_type: 0,
                            byte_2: 0,
                            byte_3: 0,
                            payload_offset: 0,
                            payload_size: 0,
                            timestamp_us: r.time,
                        },
                        slot: r.slot,
                        counter: 0,
                        rank: u8::try_from(r.rank).ok(),
                        previous: u8::try_from(r.previous).ok(),
                        kind: r.kind,
                        recovered: r.recovered,
                        gap: r.gap,
                    })
                    .collect(),
                stats: EquipmentAssemblyStats {
                    lives: stat("Lives"),
                    missed_estimate: stat("MissedEstimate"),
                    counter_jumps: stat("CounterJumps"),
                    lives_first_off_spec: stat("LivesFirstOffSpec"),
                    repeats: stat("Repeats"),
                    recovered: stat("Recovered"),
                    ..Default::default()
                },
            };
            assert_eq!(
                build_replay_equipment_changes(&raw, c.origin, c.step, &c.slots),
                c.output,
                "case {i}"
            );
        }
    }
}
