//! Facts harvested from sequential entity records; no positional inference.
use super::{
    ComponentField, EntityRecord, EntityViewStop, FilmRegistry, ReplayVehicleDeathEvidence,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectDeadState {
    pub mort: bool,
    pub enum_a: i32,
    pub enum_b: i32,
    #[serde(rename = "Val0c")]
    pub val_0c: u8,
    #[serde(rename = "Val0e")]
    pub val_0e: u8,
    pub has_ref: bool,
    #[serde(rename = "GIDPresent")]
    pub gid_present: bool,
    #[serde(rename = "GlobalID")]
    pub global_id: u32,
    pub val14: u8,
    pub val18: i8,
    #[serde(rename = "SrcTag0")]
    pub src_tag0: u32,
    #[serde(rename = "SrcTag4c")]
    pub src_tag4c: u32,
}
impl ObjectDeadState {
    /// Project fields from the existing component reader. Engine sentinels are
    /// retained for absent enum, reference and source-handle branches.
    pub fn from_fields(fields: &[ComponentField]) -> Option<Self> {
        let raw = |name| fields.iter().find(|f| f.name == name).map(|f| f.raw);
        Some(Self {
            mort: raw("dead")? != 0,
            enum_a: raw("enum_a").map_or(-1, |v| v as i32),
            enum_b: raw("enum_b").map_or(-1, |v| v as i32),
            val_0c: raw("value_0c").unwrap_or(0) as u8,
            val_0e: raw("value_0e").unwrap_or(0) as u8,
            has_ref: raw("has_reference").unwrap_or(0) != 0,
            gid_present: raw("global_id.gate").unwrap_or(0) != 0,
            global_id: raw("global_id").map_or(u32::MAX, |v| v as u32),
            val14: raw("value_14").unwrap_or(0) as u8,
            val18: raw("value_18").map_or(-1, |v| v as i8),
            src_tag0: raw("source_tag").map_or(u32::MAX, |v| v as u32),
            src_tag4c: raw("source_tag_4c").map_or(u32::MAX, |v| v as u32),
        })
    }
}

impl EntityRecord {
    /// Native EntityTrace.Dead: the last decoded value, including Mort=false.
    /// A later calibrated skip has no value and does not erase an earlier read.
    pub fn captured_dead_state(&self) -> Option<ObjectDeadState> {
        self.attempts
            .iter()
            .rev()
            .filter(|a| a.span.name == "object-dead-state-component" && a.status == Some(true))
            .find_map(|a| {
                ObjectDeadState::from_fields(self.fields.get(a.field_start..a.field_end)?)
            })
    }
}

impl super::KeyframeRecord {
    /// Native EntityTrace.Dead projected from ordered attempts. Older exports
    /// without attempt provenance return None rather than guessing by bit range.
    pub fn captured_dead_state(&self) -> Option<ObjectDeadState> {
        self.attempts
            .iter()
            .rev()
            .filter(|a| a.name == "object-dead-state-component" && a.ported == Some(true))
            .find_map(|a| {
                let [start, end] = a.field_range?;
                ObjectDeadState::from_fields(self.fields.get(start..end)?)
            })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectDeath {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub slot: u32,
    pub r#gen: u32,
    pub type_index: u32,
    pub dead: ObjectDeadState,
    pub tail_desync: bool,
}
impl ObjectDeath {
    pub fn vehicle_evidence(&self) -> Option<ReplayVehicleDeathEvidence> {
        (self.type_index == 40).then_some(ReplayVehicleDeathEvidence {
            slot: self.slot,
            generation: self.r#gen,
            timestamp_us: self.timestamp_us,
            tail_desync: self.tail_desync,
        })
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VehicleOccupancy {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub slot: u32,
    pub r#gen: u32,
    pub attached: bool,
    pub parent_slot: u32,
    pub parent_gen: u32,
    pub has_seat: bool,
    pub seat: u32,
}
/// First captured parent component on a biped record. A later record failure
/// does not invalidate this transition. Free-parent readings are retained.
pub fn vehicle_occupancy_from_record(
    record: &EntityRecord,
    timestamp_us: u64,
) -> Option<VehicleOccupancy> {
    if record.archetype != Some(35) {
        return None;
    }
    let id = record.header.id?;
    record
        .attempts
        .iter()
        .filter(|c| c.span.name == "object-parent-state-component" && c.status == Some(true))
        .find_map(|attempt| {
            let fields = record.fields.get(attempt.field_start..attempt.field_end)?;
            let raw = |name| fields.iter().find(|f| f.name == name).map(|f| f.raw);
            let attached = raw("attached")? != 0;
            Some(VehicleOccupancy {
                timestamp_us,
                slot: id & 0x3fff_ffff,
                r#gen: id >> 30,
                attached,
                parent_slot: if attached {
                    (raw("parent.value")? as u32).wrapping_add(0x200)
                } else {
                    0
                },
                parent_gen: if attached {
                    raw("parent.generation")? as u32
                } else {
                    0
                },
                has_seat: raw("tail_sign")? != 0,
                seat: raw("tail_value").unwrap_or(0) as u32,
            })
        })
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarchRecordCoverage {
    pub records: BTreeMap<u32, u64>,
    pub clean_records: BTreeMap<u32, u64>,
    pub mask_declared: BTreeMap<u32, u64>,
    pub mask_declared_desync: BTreeMap<u32, u64>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarchRecordFacts {
    pub deaths: Vec<ObjectDeath>,
    pub occupancy: Vec<VehicleOccupancy>,
    pub coverage: MarchRecordCoverage,
}
impl MarchRecordFacts {
    /// Harvest native record facts and their coverage denominators. The registry
    /// supplies the dead-state index independently for each archetype.
    pub fn harvest(
        &mut self,
        records: &[EntityRecord],
        timestamp_us: u64,
        registry: &FilmRegistry,
    ) {
        for r in records {
            let ti = r.archetype.unwrap_or(0);
            let clean = r.stop == EntityViewStop::Complete;
            *self.coverage.records.entry(ti).or_default() += 1;
            if clean {
                *self.coverage.clean_records.entry(ti).or_default() += 1;
            }
            let di = registry.archetype(ti as usize).and_then(|a| {
                a.components
                    .iter()
                    .position(|c| c == "object-dead-state-component")
            });
            if di.is_some_and(|i| i < 64 && r.mask.unwrap_or(0) & (1 << i) != 0) {
                *self.coverage.mask_declared.entry(ti).or_default() += 1;
                if !clean {
                    *self.coverage.mask_declared_desync.entry(ti).or_default() += 1;
                }
            }
            if let Some(o) = vehicle_occupancy_from_record(r, timestamp_us) {
                self.occupancy.push(o);
            }
            let Some(dead) = r.captured_dead_state().filter(|d| d.mort) else {
                continue;
            };
            let desync = r
                .attempts
                .iter()
                .find(|a| a.status != Some(true))
                .map(|a| a.span.index)
                .or(match r.stop {
                    EntityViewStop::InvalidComponent { index }
                    | EntityViewStop::UnsupportedComponent { index, .. } => Some(index),
                    _ => None,
                });
            if !clean && !di.zip(desync).is_some_and(|(d, failed)| failed > d) {
                continue;
            }
            if let Some(id) = r.header.id {
                self.deaths.push(ObjectDeath {
                    timestamp_us,
                    slot: id & 0x3fff_ffff,
                    r#gen: id >> 30,
                    type_index: ti,
                    dead,
                    tail_desync: !clean,
                });
            }
        }
    }
    pub fn deduplicate(&mut self) {
        self.deaths
            .sort_by_key(|d| (d.timestamp_us, d.slot, d.r#gen, d.tail_desync));
        let mut seen = BTreeSet::new();
        self.deaths
            .retain(|d| seen.insert((d.timestamp_us, d.slot, d.r#gen)));
        self.occupancy
            .sort_by_key(|o| (o.timestamp_us, o.slot, o.parent_slot));
        let mut seen = BTreeSet::new();
        self.occupancy.retain(|o| seen.insert(o.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Input {
        payload: Vec<u8>,
        ti: u32,
        id: u32,
        time: u64,
    }
    #[derive(Deserialize)]
    struct Case {
        names: Vec<String>,
        level: u32,
        inputs: Vec<Input>,
        deaths: Vec<ObjectDeath>,
        occupancy: Vec<VehicleOccupancy>,
        records: BTreeMap<u32, u64>,
        clean: BTreeMap<u32, u64>,
        declared: BTreeMap<u32, u64>,
        desync: BTreeMap<u32, u64>,
    }
    #[test]
    fn native_record_harvest() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/march-facts-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let registry = FilmRegistry {
                archetypes: (0..50)
                    .map(|index| FilmArchetype {
                        index,
                        components: c.names.clone(),
                        levels: vec![c.level; 3],
                    })
                    .collect(),
                major_version: 41,
                format_version: 0,
                end_byte: 0,
                truncated: false,
            };
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
                mpp_widths: [8, 5],
                position: None,
                extra_fields: false,
                corruption_check: false,
            };
            let mut facts = MarchRecordFacts::default();
            for input in c.inputs {
                let mut bindings = EntityBindings::default();
                bindings.bind(input.id, input.ti);
                let record = crate::theater::components::decode_entity_record_at(
                    &input.payload,
                    0,
                    &registry,
                    &encoding,
                    &bindings,
                )
                .unwrap();
                facts.harvest(&[record.clone(), record], input.time, &registry);
            }
            facts.deduplicate();
            assert_eq!(facts.deaths, c.deaths, "deaths {i}");
            assert_eq!(facts.occupancy, c.occupancy, "occupancy {i}");
            assert_eq!(facts.coverage.records, c.records, "records {i}");
            assert_eq!(facts.coverage.clean_records, c.clean, "clean {i}");
            assert_eq!(facts.coverage.mask_declared, c.declared, "declared {i}");
            assert_eq!(facts.coverage.mask_declared_desync, c.desync, "desync {i}");
        }
    }
}
