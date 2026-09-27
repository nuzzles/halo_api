//! Held-weapon identity emissions and native spawn-loadout classification.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeldWeaponChangeKind {
    Taken,
    Dropped,
    Swapped,
    Restated,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HeldWeaponChange {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub chunk: i64,
    pub slot: u32,
    /// Registry component index, not a zero-based inventory slot.
    pub slot_index: u8,
    pub family: u32,
    pub low: u32,
    pub previous: u32,
    pub kind: HeldWeaponChangeKind,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HeldWeaponChangeStats {
    pub records: usize,
    pub with_component: usize,
    pub emissions: usize,
    pub repeats: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeldWeaponChangeStream {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_components: Vec<BipedChannelRejection>,
    /// Every attempted component in scan order, including intermediate fields and callbacks.
    /// Absence in old exports means this trace was unavailable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_attempts: Vec<super::BipedComponentAttempt>,
    pub records: Vec<HeldWeaponChange>,
    pub stats: HeldWeaponChangeStats,
}

pub(super) fn classify(
    change: &HeldWeaponChange,
    had_previous: bool,
    loadouts: &[&KeyframeLoadout],
) -> HeldWeaponChangeKind {
    if change.family == u32::MAX {
        return HeldWeaponChangeKind::Dropped;
    }
    if had_previous {
        return if change.previous == u32::MAX {
            HeldWeaponChangeKind::Taken
        } else {
            HeldWeaponChangeKind::Swapped
        };
    }
    // Native spawnSetFrom uses the first available sample even before its date;
    // thereafter the last input-order sample at/before the change wins.
    if let Some(mut pick) = loadouts.first().copied() {
        for &sample in loadouts {
            if sample.timestamp_us <= change.timestamp_us {
                pick = sample;
            }
        }
        if pick.families.contains(&change.family) {
            return HeldWeaponChangeKind::Restated;
        }
    }
    HeldWeaponChangeKind::Taken
}

pub fn scan_held_weapon_changes(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    anchors: &BipedPositionStream,
    encoding: &PositionEncoding,
    loadouts: &[KeyframeLoadout],
) -> Result<HeldWeaponChangeStream, DecodeError> {
    scan_held_weapon_changes_impl(
        chunks,
        registry,
        anchors,
        super::biped_channels::BipedComponentReader::Legacy(encoding),
        loadouts,
    )
}
/// Held-weapon publication under the film's complete direct-reader context.
pub fn scan_held_weapon_changes_with_context(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    anchors: &BipedPositionStream,
    context: &FrameEncoding,
    loadouts: &[KeyframeLoadout],
) -> Result<HeldWeaponChangeStream, DecodeError> {
    scan_held_weapon_changes_impl(
        chunks,
        registry,
        anchors,
        super::biped_channels::BipedComponentReader::Context(context),
        loadouts,
    )
}
fn scan_held_weapon_changes_impl(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    anchors: &BipedPositionStream,
    reader: super::biped_channels::BipedComponentReader<'_>,
    loadouts: &[KeyframeLoadout],
) -> Result<HeldWeaponChangeStream, DecodeError> {
    if chunks.is_empty() {
        return Err(DecodeError::Missing("film chunks"));
    }
    if anchors.slot_band.is_none() {
        return Err(DecodeError::Missing("biped slot band"));
    }
    let arch = registry
        .archetype(35)
        .ok_or(DecodeError::Missing("biped archetype"))?;
    let indices: BTreeSet<_> = arch
        .components
        .iter()
        .enumerate()
        .filter(|(_, name)| *name == "weapon-state-type-info")
        .map(|(i, _)| i as u8)
        .collect();
    let last = *indices
        .last()
        .ok_or(DecodeError::Missing("biped weapon identity component"))?;
    let packet_indices = super::fire_events::native_packet_indices(chunks);
    let bytes = super::fire_events::native_chunk_data(chunks);
    let mut by_slot = BTreeMap::<u32, Vec<&KeyframeLoadout>>::new();
    for sample in loadouts {
        by_slot.entry(sample.slot).or_default().push(sample);
    }
    let mut previous = BTreeMap::new();
    let mut out = HeldWeaponChangeStream::default();
    for candidate in &anchors.candidates {
        let source = candidate.source;
        // Match native delta walking: missing chunks have no record callbacks.
        let Some(chunk) = bytes.get(&source.chunk_index) else {
            continue;
        };
        out.stats.records += 1;
        let record = &candidate.record;
        if !record.component_indices.iter().any(|i| indices.contains(i)) {
            continue;
        }
        out.stats.with_component += 1;
        let data = source
            .payload_offset
            .checked_add(source.payload_size)
            .and_then(|end| chunk.get(source.payload_offset..end))
            .ok_or(DecodeError::Truncated {
                chunk: source.chunk_index,
                offset: source.payload_offset,
            })?;
        for (index, component) in super::biped_channels::walk_biped_components_retaining_rejections(
            data,
            record,
            arch,
            reader,
            last,
            (
                source,
                packet_indices
                    .get(&(source.chunk_index, source.payload_offset))
                    .copied(),
            ),
            (&mut out.rejected_components, &mut out.component_attempts),
        ) {
            if !indices.contains(&index) {
                continue;
            }
            let field = |name: &str| {
                component
                    .fields
                    .iter()
                    .find(|f| f.name == name)
                    .map(|f| f.raw as u32)
            };
            let family = field("id_high").unwrap_or(u32::MAX);
            let low = field("variant").unwrap_or(u32::MAX);
            let prev = previous.insert((record.slot, index), family);
            out.stats.emissions += 1;
            out.stats.repeats += usize::from(prev == Some(family));
            let mut change = HeldWeaponChange {
                timestamp_us: source.timestamp_us,
                chunk: i64::from(source.chunk_index),
                slot: record.slot,
                slot_index: index,
                family,
                low,
                previous: prev.unwrap_or(u32::MAX),
                kind: HeldWeaponChangeKind::Taken,
            };
            change.kind = classify(
                &change,
                prev.is_some(),
                by_slot
                    .get(&record.slot)
                    .map(Vec::as_slice)
                    .unwrap_or_default(),
            );
            out.records.push(change);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn change_classification_matches_native() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/weapon-classification-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(rows.len(), 4096);
        for row in rows {
            let change = HeldWeaponChange {
                timestamp_us: 20,
                chunk: 0,
                slot: 512,
                slot_index: 43,
                family: row["family"].as_u64().unwrap() as u32,
                low: 0,
                previous: row["previous"].as_u64().unwrap() as u32,
                kind: HeldWeaponChangeKind::Taken,
            };
            let loadout = KeyframeLoadout {
                timestamp_us: 40,
                chunk: 0,
                packet_index: 0,
                slot: 512,
                families: row["spawn"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(|k| k.parse().unwrap())
                    .collect(),
            };
            let entries = if row["available"] == true {
                vec![&loadout]
            } else {
                vec![]
            };
            assert_eq!(
                serde_json::json!(classify(
                    &change,
                    row["had_previous"].as_bool().unwrap(),
                    &entries
                )),
                row["kind"]
            );
        }
    }
}
