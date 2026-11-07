//! Desired carried ability from complete, owned biped records.
use super::*;

/// The desired ability slot; not a complete list of carried objects or active effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "rank", rename_all = "snake_case")]
pub enum EquipmentSelection {
    Empty,
    /// Match-palette rank, not a universal equipment ID. Zero is a real rank.
    Rank(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EquipmentSample {
    pub timestamp_us: u64,
    /// None is unknown; Empty is an explicitly cleared selection.
    pub selection: Option<EquipmentSelection>,
    /// Recorded carried object references, independent of the desired selection.
    pub objects: Option<Vec<HeldEquipment>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HeldEquipment {
    pub entity_id: u32,
    /// Immutable tag read from a prior complete equipment NEW since the last gap.
    pub object_tag: Option<u32>,
    pub identity_source: Option<PoseSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EquipmentChannel {
    Selection,
    Inventory,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquipmentObservation {
    pub timestamp_us: u64,
    pub entity_id: u32,
    pub source: PoseSource,
    /// Rotation/request counter, never interpreted as an equipment rank.
    pub counter: Option<u8>,
    pub channel: EquipmentChannel,
    pub selection: Option<EquipmentSelection>,
    pub objects: Option<Vec<HeldEquipment>>,
    pub fields: Vec<ComponentField>,
}

fn selection(fields: &[ComponentField]) -> Option<EquipmentSelection> {
    match value(fields, "rank.gate", 1)? {
        1 if fields.len() == 2 => Some(EquipmentSelection::Empty),
        0 if fields.len() == 3 => Some(EquipmentSelection::Rank(value(fields, "rank", 6)? as u8)),
        _ => None,
    }
}

pub(super) fn observe_selection(
    record: &EntityRecord,
    time: u64,
    source: PoseSource,
) -> Option<EquipmentObservation> {
    if record.stop != EntityViewStop::Complete {
        return None;
    }
    let mut matching = record.components.iter().filter(|c| {
        matches!(
            c.name.as_str(),
            "biped-desired-ability-set" | "biped-desired-ability-set-component"
        )
    });
    let c = matching.next()?;
    if matching.next().is_some() || c.status != ComponentReadStatus::Complete {
        return None;
    }
    let counter = value(&c.fields, "counter", 3).map(|v| v as u8);
    Some(EquipmentObservation {
        timestamp_us: time,
        entity_id: record.header.id?,
        source: PoseSource {
            start_bit: c.start_bit,
            end_bit: c.end_bit,
            ..source
        },
        counter,
        channel: EquipmentChannel::Selection,
        selection: counter.and_then(|_| selection(&c.fields)),
        objects: None,
        fields: c.fields.clone(),
    })
}

/// The category-0 reference base is 0x200; generation is the full two-bit value.
/// See LevelUp equipement_porte_lecteurs.go / FUN_140d10bb0.
fn inventory(fields: &[ComponentField], objects: &ObjectIdentities) -> Option<Vec<HeldEquipment>> {
    value(fields, "head", 3)?; // Preserve, but do not guess which slot it selects.
    let count = value(fields, "count", 3)? as usize;
    let mut out = Vec::new();
    let mut expected_fields = 2;
    for i in 0..count {
        let name = format!("equipment[{i}]");
        expected_fields += 1;
        if value(fields, &format!("{name}.present"), 1)? == 1 {
            let slot = value(fields, &format!("{name}.value"), 13)? as u32 + 512;
            let generation = value(fields, &format!("{name}.generation"), 2)? as u32;
            expected_fields += 2;
            let entity_id = slot | (generation << 30);
            let identity = objects.0.get(&entity_id);
            out.push(HeldEquipment {
                entity_id,
                object_tag: identity.map(|(tag, _)| *tag),
                identity_source: identity.map(|(_, source)| *source),
            });
        }
    }
    (fields.len() == expected_fields).then_some(out)
}

pub(super) fn observe_inventory(
    record: &EntityRecord,
    time: u64,
    source: PoseSource,
    objects: &ObjectIdentities,
) -> Option<EquipmentObservation> {
    if record.stop != EntityViewStop::Complete {
        return None;
    }
    let c = component(record, "unit-equipment-component")?;
    Some(EquipmentObservation {
        timestamp_us: time,
        entity_id: record.header.id?,
        source: PoseSource {
            start_bit: c.start_bit,
            end_bit: c.end_bit,
            ..source
        },
        channel: EquipmentChannel::Inventory,
        counter: None,
        selection: None,
        objects: inventory(&c.fields, objects),
        fields: c.fields.clone(),
    })
}

#[derive(Default)]
pub(super) struct ObjectIdentities(BTreeMap<u32, (u32, PoseSource)>);
impl ObjectIdentities {
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }
    pub(super) fn observe(&mut self, r: &EntityRecord, source: PoseSource) {
        let Some(id) = r.header.id else {
            return;
        };
        // A reused slot, including a wrapped generation, cannot inherit an identity.
        self.0.retain(|old, _| {
            old & 0x3fff_ffff != id & 0x3fff_ffff
                || (*old == id && r.header.kind == RecordKind::Delta)
        });
        if r.header.kind != RecordKind::New
            || r.archetype != Some(37)
            || r.stop != EntityViewStop::Complete
        {
            return;
        }
        let Some(d) = &r.default_state else {
            return;
        };
        if d.status != DefaultStateStatus::Complete {
            return;
        }
        if let Some(tag) = value(&d.fields, "mpp.object_tag", 32) {
            let f = d
                .fields
                .iter()
                .find(|f| f.name == "mpp.object_tag")
                .unwrap();
            self.0.insert(
                id,
                (
                    tag as u32,
                    PoseSource {
                        start_bit: f.bit as i64,
                        end_bit: (f.bit + f.width) as i64,
                        ..source
                    },
                ),
            );
        }
    }
}

pub(super) fn timeline(
    observations: &mut Vec<EquipmentObservation>,
    gaps: Vec<(u64, usize)>,
    lives: &[PlayerLife],
) -> Vec<EquipmentSample> {
    let within_life = |o: &EquipmentObservation| {
        lives.iter().any(|l| {
            l.entity_id == o.entity_id && l.start_us <= o.timestamp_us && o.timestamp_us < l.end_us
        })
    };
    let mut events = BTreeMap::<u64, Vec<(usize, Option<usize>)>>::new();
    for l in lives {
        for time in [l.start_us, l.end_us] {
            events.entry(time).or_default().push((0, None));
        }
    }
    // Index before filtering preserves gap order for coincident packet clocks.
    for (time, preceding) in gaps {
        events
            .entry(time)
            .or_default()
            .push((2 * preceding + 1, None));
    }
    for (i, o) in observations
        .iter()
        .enumerate()
        .filter(|(_, o)| within_life(o))
    {
        events
            .entry(o.timestamp_us)
            .or_default()
            .push((2 * i + 2, Some(i)));
    }
    let mut selection = None;
    let mut objects = None;
    let samples = events
        .into_iter()
        .map(|(timestamp_us, mut changes)| {
            changes.sort_by_key(|(order, _)| *order);
            for (_, index) in changes {
                match index.map(|i| &observations[i]) {
                    Some(o) => match o.channel {
                        EquipmentChannel::Selection => selection = o.selection,
                        EquipmentChannel::Inventory => objects = o.objects.clone(),
                    },
                    None => {
                        selection = None;
                        objects = None;
                    }
                }
            }
            EquipmentSample {
                timestamp_us,
                selection,
                objects: objects.clone(),
            }
        })
        .collect();
    observations.retain(within_life);
    samples
}

#[cfg(test)]
mod tests;
