//! Equipment-change assembly. Recovery candidates must come from a separately validated scan.
use super::{BipedAbilityEmission, FilmPacket};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentRecoveryOrigin {
    pub bit_offset: usize,
    pub head: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentEmission {
    /// Native file number; absent for legacy envelopes whose source.chunk_index is the file identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_number: Option<i64>,
    pub ability: BipedAbilityEmission,
    pub recovery: Option<EquipmentRecoveryOrigin>,
}
impl EquipmentEmission {
    pub fn native_chunk_number(&self) -> i64 {
        self.chunk_number
            .unwrap_or(i64::from(self.ability.source.chunk_index))
    }
    pub(super) fn film_order(&self) -> (u64, i64, usize) {
        (
            self.ability.source.timestamp_us,
            self.native_chunk_number(),
            self.ability
                .packet_index
                .unwrap_or(self.ability.source.payload_offset),
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentChangeKind {
    Taken,
    Spent,
    Spawned,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentChange {
    /// Native file number; absent for legacy envelopes whose source.chunk_index is the file identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_number: Option<i64>,
    /// Native ordinal among all packets in the source chunk; unknown without source bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub source: FilmPacket,
    pub slot: u32,
    pub counter: u8,
    pub rank: Option<u8>,
    /// A preceding rank can be unreliable when gap is nonzero.
    pub previous: Option<u8>,
    pub kind: EquipmentChangeKind,
    pub recovered: bool,
    pub gap: u8,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentAssemblyStats {
    pub lives: usize,
    pub repeats: usize,
    pub counter_jumps: usize,
    pub missed_estimate: usize,
    pub lives_first_off_spec: usize,
    pub spawned: usize,
    pub taken: usize,
    pub spent: usize,
    pub recovered: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentChanges {
    pub records: Vec<EquipmentChange>,
    pub stats: EquipmentAssemblyStats,
}
pub(crate) fn equipment_counter_step(from: u8, to: u8) -> u8 {
    to.wrapping_sub(from) & 7
}
fn violations(list: &[EquipmentEmission], head: bool) -> (usize, usize) {
    let mut prev = 4;
    let mut seen = head;
    let mut repeats = 0;
    let mut jumps = 0;
    for e in list {
        if seen {
            match equipment_counter_step(prev, e.ability.counter) {
                0 => repeats += 1,
                1 => {}
                _ => jumps += 1,
            }
        }
        prev = e.ability.counter;
        seen = true;
    }
    (repeats, jumps)
}
/// Merge strict and recovered emissions in film order, prune recovery that worsens
/// the counter chain, then classify changes using caller-supplied birth times.
/// This is the assembly stage only; it does not search for missing emissions.
pub fn assemble_equipment_changes(
    emissions: &[EquipmentEmission],
    births: &BTreeMap<u32, u64>,
) -> EquipmentChanges {
    let mut by_slot = BTreeMap::<u32, Vec<EquipmentEmission>>::new();
    for e in emissions {
        by_slot.entry(e.ability.slot).or_default().push(e.clone());
    }
    let mut out = EquipmentChanges::default();
    for (slot, mut list) in by_slot {
        let head = list
            .iter()
            .any(|e| e.recovery.as_ref().is_some_and(|r| r.head));
        list.sort_by_key(|e| {
            (
                e.ability.source.timestamp_us,
                e.native_chunk_number(),
                e.ability
                    .packet_index
                    .unwrap_or(e.ability.source.payload_offset),
                e.recovery.as_ref().map(|r| r.bit_offset),
            )
        });
        loop {
            let (repeats, jumps) = violations(&list, head);
            let remove = (0..list.len()).find(|&i| {
                if list[i].recovery.is_none() {
                    return false;
                }
                let mut without = list.clone();
                without.remove(i);
                let (r, j) = violations(&without, head);
                r < repeats || j < jumps
            });
            if let Some(i) = remove {
                list.remove(i);
            } else {
                break;
            }
        }
        let Some(first) = list.first() else { continue };
        out.stats.lives += 1;
        out.stats.lives_first_off_spec += usize::from(first.ability.counter != 5);
        let mut previous = None;
        for (i, e) in list.iter().enumerate() {
            let a = &e.ability;
            let mut gap = 0;
            if i > 0 {
                match equipment_counter_step(list[i - 1].ability.counter, a.counter) {
                    0 => out.stats.repeats += 1,
                    1 => {}
                    step => {
                        gap = step - 1;
                        out.stats.counter_jumps += 1;
                        out.stats.missed_estimate += usize::from(gap);
                    }
                }
            }
            let kind = if a.rank.is_none() {
                EquipmentChangeKind::Spent
            } else if i == 0
                && births
                    .get(&slot)
                    .is_some_and(|birth| a.source.timestamp_us <= birth.wrapping_add(1_000_000))
            {
                EquipmentChangeKind::Spawned
            } else {
                EquipmentChangeKind::Taken
            };
            match kind {
                EquipmentChangeKind::Taken => out.stats.taken += 1,
                EquipmentChangeKind::Spent => out.stats.spent += 1,
                EquipmentChangeKind::Spawned => out.stats.spawned += 1,
            }
            out.stats.recovered += usize::from(e.recovery.is_some());
            out.records.push(EquipmentChange {
                chunk_number: e.chunk_number,
                packet_index: a.packet_index,
                source: a.source,
                slot,
                counter: a.counter,
                rank: a.rank,
                previous,
                kind,
                recovered: e.recovery.is_some(),
                gap,
            });
            previous = a.rank;
        }
    }
    out.records.sort_by_key(|e| {
        (
            e.source.timestamp_us,
            e.chunk_number.unwrap_or(i64::from(e.source.chunk_index)),
            e.packet_index.unwrap_or(e.source.payload_offset),
            e.slot,
        )
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    #[test]
    fn assembly_pruning_classification_and_counter_stats_match_go() {
        let mut text = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut text)
        .unwrap();
        let oracle: Value = serde_json::from_str(&text).unwrap();
        let cases = oracle["equipment_cases"].as_array().unwrap();
        assert_eq!(cases.len(), 512);
        for case in cases {
            let births: BTreeMap<u32, u64> =
                serde_json::from_value(case["births"].clone()).unwrap();
            let input: Vec<_> = case["input"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let e = &v["emission"];
                    EquipmentEmission {
                        chunk_number: None,
                        ability: BipedAbilityEmission {
                            packet_index: None,
                            source: FilmPacket {
                                chunk_index: e["Chunk"].as_i64().unwrap() as i32,
                                packet_type: 0,
                                byte_2: 0,
                                byte_3: 0,
                                payload_offset: e["PacketIndex"].as_u64().unwrap() as usize,
                                payload_size: 0,
                                timestamp_us: e["TimestampUS"].as_u64().unwrap(),
                            },
                            slot: e["Slot"].as_u64().unwrap() as u32,
                            counter: e["Counter"].as_u64().unwrap() as u8,
                            rank: e["Rank"].as_u64().map(|v| v as u8),
                        },
                        recovery: v["recovered"].as_bool().unwrap().then(|| {
                            EquipmentRecoveryOrigin {
                                bit_offset: v["off"].as_u64().unwrap() as usize,
                                head: v["head"].as_bool().unwrap(),
                            }
                        }),
                    }
                })
                .collect();
            let out = assemble_equipment_changes(&input, &births);
            let expected = case["records"].as_array().unwrap();
            assert_eq!(out.records.len(), expected.len());
            // Go sorts the final output with an unstable sort and omits the recovery
            // offset from its key. Compare complete records within those undefined ties;
            // still require chronological ordering and every classification/provenance field.
            assert!(out.records.windows(2).all(|w| {
                let key = |a: &EquipmentChange| {
                    (
                        a.source.timestamp_us,
                        a.source.chunk_index,
                        a.source.payload_offset,
                        a.slot,
                    )
                };
                key(&w[0]) <= key(&w[1])
            }));
            let mut actual: Vec<_> = out.records.iter().map(|a|
                json!({"TimestampUS":a.source.timestamp_us,"Chunk":a.source.chunk_index,"PacketIndex":a.source.payload_offset,"Slot":a.slot,"Counter":a.counter,"Rank":a.rank.map_or(-1,i32::from),"Previous":a.previous.map_or(-1,i32::from),"Kind":a.kind,"Recovered":a.recovered,"Gap":a.gap}).to_string()
            ).collect();
            let mut expected: Vec<_> = expected.iter().map(Value::to_string).collect();
            actual.sort();
            expected.sort();
            assert_eq!(actual, expected);
            let mut expected = case["stats"].clone();
            expected.as_object_mut().unwrap().remove("Walk");
            let s = &out.stats;
            assert_eq!(
                json!({"Lives":s.lives,"Repeats":s.repeats,"CounterJumps":s.counter_jumps,"MissedEstimate":s.missed_estimate,"LivesFirstOffSpec":s.lives_first_off_spec,"Spawned":s.spawned,"Taken":s.taken,"Spent":s.spent,"Recovered":s.recovered}),
                expected
            );
            assert_eq!(
                serde_json::from_value::<EquipmentChanges>(json!(out)).unwrap(),
                out
            );
        }
    }
}
