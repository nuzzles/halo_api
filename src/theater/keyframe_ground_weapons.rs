//! Ground-weapon family signatures within recovered keyframe entity records.
//! Families remain in bit order; aliases and duplicate signatures are retained.
use super::{
    DecodeError,
    fire_events::{native_chunk_packets, native_chunk_prefix},
};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeGroundWeapon {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub chunk: i32,
    pub packet_index: usize,
    pub slot: u32,
    #[serde(rename = "Gen")]
    pub generation: u32,
    pub families: Vec<u32>,
}
pub fn decode_keyframe_ground_weapons(
    payload: &[u8],
    known: &BTreeSet<u32>,
) -> Vec<KeyframeGroundWeapon> {
    super::keyframe_loadouts::families_by_entity(payload, known, 42)
        .into_iter()
        .map(|(id, families)| KeyframeGroundWeapon {
            timestamp_us: 0,
            chunk: 0,
            packet_index: 0,
            slot: id & 0x3fff_ffff,
            generation: id >> 30,
            families,
        })
        .collect()
}
pub fn scan_keyframe_ground_weapons(
    chunks: &[FilmChunkData],
) -> Result<Vec<KeyframeGroundWeapon>, DecodeError> {
    scan_keyframe_ground_weapons_with_families(
        chunks,
        &super::v41_weapon_families().keys().copied().collect(),
    )
}
pub fn scan_keyframe_ground_weapons_with_families(
    chunks: &[FilmChunkData],
    known: &BTreeSet<u32>,
) -> Result<Vec<KeyframeGroundWeapon>, DecodeError> {
    if known.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for c in native_chunk_prefix(chunks)? {
        for (index, p) in native_chunk_packets(c).into_iter().enumerate() {
            if p.packet_type != 2 {
                continue;
            }
            for mut weapon in decode_keyframe_ground_weapons(
                &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                known,
            ) {
                weapon.timestamp_us = p.timestamp_us;
                weapon.chunk = c.metadata.index;
                weapon.packet_index = index;
                out.push(weapon);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_ground_keyframe_families() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/ground-keyframes-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows["cases"].as_array().unwrap().iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let pay: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                .collect();
            let known = serde_json::from_value(row["known"].clone()).unwrap();
            let expected: Vec<KeyframeGroundWeapon> =
                serde_json::from_value(row["records"].clone()).unwrap();
            assert_eq!(
                decode_keyframe_ground_weapons(&pay, &known),
                expected,
                "case {i}"
            );
        }
        assert!(
            scan_keyframe_ground_weapons_with_families(&[], &BTreeSet::new())
                .unwrap()
                .is_empty()
        );
    }
}

/// Loading and scan failures remain distinct from a successful empty result.
#[derive(Debug, thiserror::Error)]
pub enum GroundWeaponScanError {
    #[error(transparent)]
    Source(#[from] super::FilmSourceError),
    #[error("no readable film chunk")]
    NoReadableChunk,
}

/// Scan an already loaded source with the native boolean family catalog.
/// A nonempty all-false catalog still performs the native readable-chunk check.
pub fn scan_source_keyframe_ground_weapons(
    source: &super::FilmSource,
    known: &std::collections::BTreeMap<u32, bool>,
) -> Result<Vec<KeyframeGroundWeapon>, GroundWeaponScanError> {
    if known.is_empty() {
        return Ok(Vec::new());
    }
    let families = known
        .iter()
        .filter_map(|(&k, &present)| present.then_some(k))
        .collect();
    let mut out = Vec::new();
    let mut read = false;
    for number in source.data_chunk_numbers() {
        let Some((bytes, packets)) = source.chunk_by_number(number) else {
            continue;
        };
        read = true;
        for (index, packet) in packets.iter().enumerate() {
            if packet.packet_type != 2 {
                continue;
            }
            let payload =
                &bytes[packet.payload_offset..packet.payload_offset + packet.payload_size];
            for mut weapon in decode_keyframe_ground_weapons(payload, &families) {
                weapon.timestamp_us = packet.timestamp_us;
                weapon.chunk = number as i32;
                weapon.packet_index = index;
                out.push(weapon);
            }
        }
    }
    if !read {
        return Err(GroundWeaponScanError::NoReadableChunk);
    }
    Ok(out)
}

/// Native offline directory wrapper. Empty catalogs bypass disk access;
/// loading errors propagate, even when every catalog flag is false.
#[cfg(not(target_arch = "wasm32"))]
pub fn scan_directory_keyframe_ground_weapons(
    directory: impl AsRef<std::path::Path>,
    known: &std::collections::BTreeMap<u32, bool>,
) -> Result<Vec<KeyframeGroundWeapon>, GroundWeaponScanError> {
    if known.is_empty() {
        return Ok(Vec::new());
    }
    scan_source_keyframe_ground_weapons(&super::FilmSource::load_directory(directory, &[])?, known)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod directory_tests {
    use super::*;
    use std::{collections::BTreeMap, io::Read};
    #[test]
    fn native_ground_directory_loading_and_results() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/ground-directory-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 128);
        let root =
            std::env::temp_dir().join(format!("halo-ground-directory-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let mut count = 0;
        for (i, row) in rows.into_iter().enumerate() {
            let dir = root.join(i.to_string());
            if !row["missing"].as_bool().unwrap() {
                std::fs::create_dir(&dir).unwrap();
            }
            let files: BTreeMap<String, String> =
                serde_json::from_value(row["files"].clone()).unwrap();
            for (name, h) in files {
                let bytes: Vec<u8> = (0..h.len())
                    .step_by(2)
                    .map(|p| u8::from_str_radix(&h[p..p + 2], 16).unwrap())
                    .collect();
                std::fs::write(dir.join(name), bytes).unwrap();
            }
            let known = serde_json::from_value(row["known"].clone()).unwrap();
            let actual = scan_directory_keyframe_ground_weapons(&dir, &known);
            let kind = match &actual {
                Ok(_) => "",
                Err(GroundWeaponScanError::Source(_)) => "load",
                Err(GroundWeaponScanError::NoReadableChunk) => "unreadable",
            };
            assert_eq!(kind, row["error"].as_str().unwrap(), "case {i}");
            if let Ok(records) = actual {
                let expected: Vec<KeyframeGroundWeapon> =
                    serde_json::from_value(row["records"].clone()).unwrap();
                count += records.len();
                assert_eq!(records, expected, "case {i}");
            }
        }
        std::fs::remove_dir_all(root).unwrap();
        assert!(count > 0, "positive loaded records required");
        eprintln!("{count} native directory records");
    }
}
