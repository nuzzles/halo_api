//! Reference weapon-family scans within recovered keyframe records.
//! These are catalog signature observations, not a sequentially decoded loadout.
use super::{DecodeError, recover_keyframe_anchors};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

/// Fold a 64-bit weapon ID to its high-word family. Native acceptance also
/// admits unknown families carrying the common weapon suffix.
pub fn canonical_weapon_id(id: u64) -> (u32, bool) {
    let family = (id >> 32) as u32;
    (
        family,
        v41_weapon_families().contains_key(&family) || id as u32 == 0x42c9679f,
    )
}
/// Native canonical family label; suffix-only accepted families may be unnamed.
pub fn canonical_weapon_name(family: u32) -> &'static str {
    v41_weapon_families()
        .get(&family)
        .map(String::as_str)
        .unwrap_or("")
}

/// Pinned reference catalog, derived from LevelUp's weapon enum and fusion table.
pub fn v41_weapon_families() -> &'static BTreeMap<u32, String> {
    static CATALOG: OnceLock<BTreeMap<u32, String>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("reference/weapon-families-v41.json"))
            .expect("validated embedded v41 weapon catalog")
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeLoadout {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub chunk: i32,
    pub packet_index: usize,
    pub slot: u32,
    /// Raw families in bit order, including repeated occurrences and aliases.
    pub families: Vec<u32>,
}

/// Scan using the pinned reference catalog. No alias folding or deduplication.
pub fn scan_keyframe_loadouts(
    chunks: &[FilmChunkData],
) -> Result<Vec<KeyframeLoadout>, DecodeError> {
    scan_keyframe_loadouts_with_families(chunks, &v41_weapon_families().keys().copied().collect())
}

/// A caller-supplied family set determines which signatures are accepted.
/// Empty sets produce no observations even when no film chunks are available.
pub fn scan_keyframe_loadouts_with_families(
    chunks: &[FilmChunkData],
    known: &BTreeSet<u32>,
) -> Result<Vec<KeyframeLoadout>, DecodeError> {
    if known.is_empty() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for chunk in super::fire_events::native_chunk_prefix(chunks)? {
        for (index, packet) in super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .enumerate()
        {
            if packet.packet_type != 2 {
                continue;
            }
            let payload =
                &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            for (slot, families) in families_by_record(payload, known, 35) {
                out.push(KeyframeLoadout {
                    timestamp_us: packet.timestamp_us,
                    chunk: chunk.metadata.index,
                    packet_index: index,
                    slot,
                    families,
                });
            }
        }
    }
    Ok(out)
}

pub(crate) fn families_by_record(
    payload: &[u8],
    known: &BTreeSet<u32>,
    archetype: u32,
) -> Vec<(u32, Vec<u32>)> {
    families_by_entity(payload, known, archetype)
        .into_iter()
        .map(|(id, families)| (id & 0x3fff_ffff, families))
        .collect()
}
/// Preserve the packed generation for ground-object lifetime attribution.
pub(crate) fn families_by_entity(
    payload: &[u8],
    known: &BTreeSet<u32>,
    archetype: u32,
) -> Vec<(u32, Vec<u32>)> {
    let mut anchors = recover_keyframe_anchors(payload);
    anchors.sort_by_key(|a| a.bit);
    let mut out: Vec<(u32, Vec<u32>)> = vec![];
    let mut record = None;
    let mut next = 0;
    let mut word = 0u32;
    let mut published = None;
    for bit in 0..payload.len() * 8 {
        word = (word << 1) | u32::from((payload[bit / 8] >> (7 - bit % 8)) & 1);
        if bit < 31 {
            continue;
        }
        let start = bit - 31;
        while next < anchors.len() && anchors[next].bit <= start {
            record = Some(next);
            next += 1;
        }
        let Some(index) = record else {
            continue;
        };
        if anchors[index].archetype != archetype || !known.contains(&word) {
            continue;
        }
        if published != Some(index) {
            out.push((anchors[index].id, vec![]));
            published = Some(index);
        }
        out.last_mut().unwrap().1.push(word);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn recovered_family_attribution_matches_native() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/keyframe-loadouts-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(rows.len(), 256);
        for row in rows {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let known: BTreeSet<u32> = serde_json::from_value(row["known"].clone()).unwrap();
            let actual: Vec<_> = families_by_record(&data, &known, 35)
                .into_iter()
                .map(|(slot, families)| serde_json::json!({"slot":slot,"families":families}))
                .collect();
            assert_eq!(serde_json::json!(actual), row["records"]);
        }
        assert!(
            scan_keyframe_loadouts_with_families(&[], &BTreeSet::new())
                .unwrap()
                .is_empty()
        );
        assert!(scan_keyframe_loadouts(&[]).is_err());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod source_tests {
    use super::*;
    use std::io::Read;
    fn cases() -> Vec<serde_json::Value> {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/loadout-sources-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        serde_json::from_slice(&raw).unwrap()
    }
    fn unhex(h: &str) -> Vec<u8> {
        (0..h.len())
            .step_by(2)
            .map(|p| u8::from_str_radix(&h[p..p + 2], 16).unwrap())
            .collect()
    }
    fn chunks(row: &serde_json::Value) -> Vec<FilmChunkData> {
        use crate::clients::hi::models::FilmChunk;
        row["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| FilmChunkData {
                metadata: FilmChunk {
                    index: c["index"].as_i64().unwrap() as i32,
                    chunk_type: c["kind"].as_i64().unwrap() as i32,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: 0,
                    file_relative_path: String::new(),
                },
                data: unhex(c["hex"].as_str().unwrap()),
            })
            .collect()
    }
    #[test]
    fn native_loadout_sources_and_directories() {
        let root =
            std::env::temp_dir().join(format!("halo-loadout-sources-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let rows = cases();
        assert_eq!(rows.len(), 128);
        for (i, row) in rows.into_iter().enumerate() {
            let dir = root.join(i.to_string());
            if !row["missing"].as_bool().unwrap() {
                std::fs::create_dir(&dir).unwrap();
            }
            let files: BTreeMap<String, String> =
                serde_json::from_value(row["files"].clone()).unwrap();
            for (name, h) in files {
                std::fs::write(dir.join(name), unhex(&h)).unwrap();
            }
            let known = serde_json::from_value(row["known"].clone()).unwrap();
            let actual = scan_directory_keyframe_loadouts(&dir, &known);
            let kind = match &actual {
                Ok(_) => "",
                Err(KeyframeLoadoutScanError::Source(_)) => "load",
                Err(KeyframeLoadoutScanError::NoReadableChunk) => "unreadable",
            };
            assert_eq!(kind, row["error"].as_str().unwrap(), "directory {i}");
            if let Ok(records) = actual {
                let expected: Vec<KeyframeLoadout> =
                    serde_json::from_value(row["records"].clone()).unwrap();
                assert_eq!(records, expected, "directory {i}");
            }
            let chunks = chunks(&row);
            let mut data = vec![vec![41, 0, 0, 0, 27, 0, 0, 0]];
            data.extend(chunks.iter().map(|c| c.data.clone()));
            let mut metadata = vec![super::super::FilmSourceMetadata {
                index: 0,
                chunk_type: 0,
                start_ms: 0,
            }];
            metadata.extend(chunks.iter().map(|c| super::super::FilmSourceMetadata {
                index: i64::from(c.metadata.index),
                chunk_type: i64::from(c.metadata.chunk_type),
                start_ms: 0,
            }));
            let source = super::super::FilmSource::load(&data, &metadata).unwrap();
            let actual = scan_source_keyframe_loadouts(&source, &known);
            assert_eq!(
                actual.is_err(),
                !row["loaded_error"].as_str().unwrap().is_empty(),
                "loaded {i}"
            );
            if let Ok(records) = actual {
                let expected: Vec<KeyframeLoadout> =
                    serde_json::from_value(row["loaded"].clone()).unwrap();
                assert_eq!(records, expected, "loaded {i}");
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "requires four local v41 films"]
    fn local_loadout_sources_match_native_corpus() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/gameplay-levelup-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 4);
        let known = v41_weapon_families().keys().map(|&k| (k, true)).collect();
        let mut total = 0;
        for row in rows {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(row["folder"].as_str().unwrap());
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join("film.json")).unwrap()).unwrap();
            let mut data = Vec::new();
            let mut metadata = Vec::new();
            for c in manifest["chunks"].as_array().unwrap() {
                data.push(std::fs::read(root.join(c["file"].as_str().unwrap())).unwrap());
                metadata.push(super::super::FilmSourceMetadata {
                    index: c["index"].as_i64().unwrap(),
                    chunk_type: c["chunk_type"].as_i64().unwrap(),
                    start_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                });
            }
            let source = super::super::FilmSource::load(&data, &metadata).unwrap();
            let expected: Vec<KeyframeLoadout> =
                serde_json::from_value(row["inputs"]["Loadouts"].clone()).unwrap();
            assert_eq!(
                scan_source_keyframe_loadouts(&source, &known).unwrap(),
                expected
            );
            assert_eq!(
                scan_keyframe_loadouts(&source.into_chunks().unwrap()).unwrap(),
                expected
            );
            total += expected.len();
        }
        assert_eq!(total, 501);
    }
    #[test]
    fn native_loadout_chunk_selection() {
        for (i, row) in cases().into_iter().enumerate() {
            let flags: BTreeMap<u32, bool> = serde_json::from_value(row["known"].clone()).unwrap();
            // Set-based compatibility API cannot represent false map entries.
            if !flags.is_empty() && flags.values().all(|b| !b) {
                continue;
            }
            let known = flags
                .into_iter()
                .filter_map(|(k, v)| v.then_some(k))
                .collect();
            let actual = scan_keyframe_loadouts_with_families(&chunks(&row), &known);
            assert_eq!(
                actual.is_err(),
                !row["loaded_error"].as_str().unwrap().is_empty(),
                "case {i}"
            );
            if let Ok(records) = actual {
                let expected: Vec<KeyframeLoadout> =
                    serde_json::from_value(row["loaded"].clone()).unwrap();
                assert_eq!(records, expected, "case {i}");
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyframeLoadoutScanError {
    #[error(transparent)]
    Source(#[from] super::FilmSourceError),
    #[error("no readable film chunk")]
    NoReadableChunk,
}

/// Native loaded-source scan. Empty catalogs bypass scanning; nonempty all-false
/// catalogs still require a readable data chunk. Families remain in bit order.
pub fn scan_source_keyframe_loadouts(
    source: &super::FilmSource,
    known: &BTreeMap<u32, bool>,
) -> Result<Vec<KeyframeLoadout>, KeyframeLoadoutScanError> {
    if known.is_empty() {
        return Ok(Vec::new());
    }
    let families = known.iter().filter_map(|(&k, &v)| v.then_some(k)).collect();
    let mut out = Vec::new();
    let mut read = false;
    for number in source.data_chunk_numbers() {
        let Some((data, packets)) = source.chunk_by_number(number) else {
            continue;
        };
        read = true;
        for (index, p) in packets.iter().enumerate() {
            if p.packet_type != 2 {
                continue;
            }
            for (slot, families) in families_by_record(
                &data[p.payload_offset..p.payload_offset + p.payload_size],
                &families,
                35,
            ) {
                out.push(KeyframeLoadout {
                    timestamp_us: p.timestamp_us,
                    chunk: number as i32,
                    packet_index: index,
                    slot,
                    families,
                });
            }
        }
    }
    if !read {
        return Err(KeyframeLoadoutScanError::NoReadableChunk);
    }
    Ok(out)
}

/// Native offline directory wrapper; empty catalogs do not touch the filesystem.
#[cfg(not(target_arch = "wasm32"))]
pub fn scan_directory_keyframe_loadouts(
    directory: impl AsRef<std::path::Path>,
    known: &BTreeMap<u32, bool>,
) -> Result<Vec<KeyframeLoadout>, KeyframeLoadoutScanError> {
    if known.is_empty() {
        return Ok(Vec::new());
    }
    scan_source_keyframe_loadouts(&super::FilmSource::load_directory(directory, &[])?, known)
}
