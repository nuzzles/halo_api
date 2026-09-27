//! Reference research-only world-object candidates. These scans are noisy and
//! must not be treated as confirmed ground-weapon positions or playback state.
use super::{FilmMapBounds, WorldObjectPrecision};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Native research sample, distinct from a validated mobile-lifetime sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResearchWorldObjectSample {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    #[serde(rename = "Gen")]
    pub generation: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Native nearest-sample query: scans all candidates, and retains the first
/// input on equal distance. No sorted-input assumption or interpolation.
pub fn nearest_research_world_object_sample(
    samples: &[ResearchWorldObjectSample],
    at_us: u64,
) -> Option<(&ResearchWorldObjectSample, u64)> {
    samples
        .iter()
        .map(|s| (s, s.timestamp_us.abs_diff(at_us)))
        .min_by_key(|(_, gap)| *gap)
}

/// Loaded equivalent of the native WorldObjectPositionsForBand research helper.
/// Keeps every candidate, grouped by slot (not lifetime), sorted by timestamp
/// with native tie ordering. Missing bounds, an empty band or unreadable prefix
/// returns no candidates. Precision is the caller's resolved world descriptor.
/// The native scanner's zero-tail behavior is retained; these are research
/// candidates, not independently established positions or physical actions.
pub fn scan_research_world_object_samples(
    chunks: &[FilmChunkData],
    map: Option<&FilmMapBounds>,
    band: &BTreeSet<u32>,
    precision: WorldObjectPrecision,
) -> BTreeMap<u32, Vec<ResearchWorldObjectSample>> {
    let mut out = BTreeMap::<u32, Vec<ResearchWorldObjectSample>>::new();
    let Some(map) = map else {
        return out;
    };
    if band.is_empty() {
        return out;
    }
    let Ok(chunks) = super::fire_events::native_chunk_prefix(chunks) else {
        return out;
    };
    for chunk in chunks {
        for packet in super::fire_events::native_chunk_packets(chunk) {
            if packet.packet_type != 0 {
                continue;
            }
            let payload =
                &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            for r in
                super::decode_world_object_positions_with_precision(payload, band, map, precision)
            {
                out.entry(r.slot)
                    .or_default()
                    .push(ResearchWorldObjectSample {
                        timestamp_us: packet.timestamp_us,
                        generation: r.generation,
                        x: r.sample.x,
                        y: r.sample.y,
                        z: r.sample.z,
                    });
            }
        }
    }
    for samples in out.values_mut() {
        super::native_sort::sort_by(samples, |a, b| a.timestamp_us.cmp(&b.timestamp_us));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::io::Read;
    #[test]
    fn native_research_samples_and_nearest() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/world-research-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let rows = rows.as_array().unwrap();
        assert_eq!(rows.len(), 256);
        let mut total = 0;
        for (index, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let h = c["hex"].as_str().unwrap();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 0,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: 0,
                            file_relative_path: String::new(),
                        },
                        data: (0..h.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                            .collect(),
                    }
                })
                .collect();
            let map = FilmMapBounds {
                module: String::new(),
                min: [-100., -200., -300.],
                max: [300., 400., 500.],
                axis_widths: serde_json::from_value(row["widths"].clone()).unwrap(),
                region: row["region"].as_u64().unwrap() as u32,
                region_index_bits: row["index_bits"].as_u64().unwrap() as usize,
            };
            let band = serde_json::from_value(row["band"].clone()).unwrap();
            let actual = scan_research_world_object_samples(
                &chunks,
                row["bounds_ok"].as_bool().unwrap().then_some(&map),
                &band,
                WorldObjectPrecision::from_map(&map),
            );
            let expected: BTreeMap<u32, Vec<ResearchWorldObjectSample>> =
                serde_json::from_value(row["samples"].clone()).unwrap();
            assert_eq!(actual, expected, "scan {index}");
            total += actual.values().map(Vec::len).sum::<usize>();
            let samples: Vec<ResearchWorldObjectSample> =
                serde_json::from_value(row["query_samples"].clone()).unwrap();
            let nearest =
                nearest_research_world_object_sample(&samples, row["at"].as_u64().unwrap());
            assert_eq!(nearest.is_some(), row["ok"].as_bool().unwrap());
            if let Some((sample, gap)) = nearest {
                let expected: ResearchWorldObjectSample =
                    serde_json::from_value(row["nearest"].clone()).unwrap();
                assert_eq!(sample, &expected, "nearest {index}");
                assert_eq!(gap, row["gap"].as_u64().unwrap());
            }
        }
        assert!(total > 1000, "oracle must exercise positive grouping");
        eprintln!("native research samples: {total}");
    }
}

/// Native offline world-object precision: use detected I0 widths when available,
/// otherwise retain the pinned profile invariant. Bounds do not select widths.
pub fn research_world_object_precision(chunks: &[FilmChunkData]) -> WorldObjectPrecision {
    let mut precision = WorldObjectPrecision {
        index_bits: 1,
        axis_bits: [13, 13, 14],
        region: 0,
    };
    let detection = super::detect_film_i0_layout(chunks);
    if detection.refusal.is_none()
        && let Some(layout) = detection.layout
        && layout.axis_widths.iter().all(|&width| width != 0)
    {
        precision.axis_bits = layout.axis_widths.map(|w| w as usize);
        if layout.gate_bits > 4 {
            precision.index_bits = (layout.gate_bits - 4) as usize;
        }
        precision.region = layout.region;
    }
    precision
}

// Preserve native numbered-prefix lookup while adapting to existing scanners.
// Out-of-range file numbers are never cast into valid data-chunk numbers.
#[cfg(not(target_arch = "wasm32"))]
fn research_directory_chunks(directory: &std::path::Path) -> Option<Vec<FilmChunkData>> {
    use crate::clients::hi::models::FilmChunk;
    let source = super::FilmSource::load_directory(directory, &[]).ok()?;
    let numbers = std::iter::once(0).chain(source.data_chunk_numbers());
    Some(
        numbers
            .filter_map(|number| {
                let (data, _) = source.chunk_by_number(number)?;
                Some(FilmChunkData {
                    metadata: FilmChunk {
                        index: i32::try_from(number).ok()?,
                        chunk_type: 0,
                        start_time_offset_ms: 0,
                        duration_ms: 0,
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: data.to_vec(),
                })
            })
            .collect(),
    )
}

/// Native offline research band: fill the ground-weapon slot range and exclude
/// slots observed with other archetypes. Load failures yield an empty band.
#[cfg(not(target_arch = "wasm32"))]
pub fn research_ground_weapon_directory_band(
    directory: impl AsRef<std::path::Path>,
) -> BTreeSet<u32> {
    research_directory_chunks(directory.as_ref()).map_or_else(BTreeSet::new, |chunks| {
        super::scan_world_object_keyframes(&chunks, 42).band
    })
}

/// Native offline research scan for an explicit band. Failed loads, absent
/// bounds, and empty bands yield no candidates. Map widths are deliberately
/// ignored: this wrapper installs detected recording widths or native defaults.
#[cfg(not(target_arch = "wasm32"))]
pub fn scan_directory_research_world_object_samples(
    directory: impl AsRef<std::path::Path>,
    map: Option<&FilmMapBounds>,
    band: &BTreeSet<u32>,
) -> BTreeMap<u32, Vec<ResearchWorldObjectSample>> {
    if map.is_none() || band.is_empty() {
        return BTreeMap::new();
    }
    let Some(chunks) = research_directory_chunks(directory.as_ref()) else {
        return BTreeMap::new();
    };
    let precision = research_world_object_precision(&chunks);
    scan_research_world_object_samples(&chunks, map, band, precision)
}

/// Native GroundWeaponPositions research wrapper. Candidates are not validated
/// ground-weapon locations and must not be published as playback state.
#[cfg(not(target_arch = "wasm32"))]
pub fn scan_directory_research_ground_weapon_samples(
    directory: impl AsRef<std::path::Path>,
    map: Option<&FilmMapBounds>,
) -> BTreeMap<u32, Vec<ResearchWorldObjectSample>> {
    if map.is_none() {
        return BTreeMap::new();
    }
    let directory = directory.as_ref();
    scan_directory_research_world_object_samples(
        directory,
        map,
        &research_ground_weapon_directory_band(directory),
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod directory_tests {
    use super::*;
    use std::io::Read;
    fn cases() -> Vec<serde_json::Value> {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/research-directory-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        serde_json::from_slice(&raw).unwrap()
    }
    fn check(row: &serde_json::Value, dir: &std::path::Path) {
        let flags: BTreeMap<u32, bool> = serde_json::from_value(row["band"].clone()).unwrap();
        let band = flags
            .into_iter()
            .filter_map(|(s, b)| b.then_some(s))
            .collect();
        let flags: BTreeMap<u32, bool> = serde_json::from_value(row["slots"].clone()).unwrap();
        let slots = flags
            .into_iter()
            .filter_map(|(s, b)| b.then_some(s))
            .collect();
        assert_eq!(research_ground_weapon_directory_band(dir), slots);
        let chunks = research_directory_chunks(dir).unwrap_or_default();
        let precision = research_world_object_precision(&chunks);
        assert_eq!(
            precision.index_bits as u64,
            row["precision"]["IndexW"].as_u64().unwrap()
        );
        assert_eq!(
            serde_json::to_value(precision.axis_bits).unwrap(),
            row["precision"]["AxisW"]
        );
        assert_eq!(
            u64::from(precision.region),
            row["precision"]["Region"].as_u64().unwrap()
        );
        // Deliberately unrelated map widths: only its coordinate bounds apply.
        let map = FilmMapBounds {
            module: String::new(),
            min: [-100., -200., -300.],
            max: [300., 400., 500.],
            axis_widths: [5, 6, 7],
            region: 2,
            region_index_bits: 3,
        };
        let bounds = row["bounds"].as_bool().unwrap().then_some(&map);
        let expected: BTreeMap<u32, Vec<ResearchWorldObjectSample>> =
            serde_json::from_value(row["samples"].clone()).unwrap();
        assert_eq!(
            scan_directory_research_world_object_samples(dir, bounds, &band),
            expected
        );
        let expected: BTreeMap<u32, Vec<ResearchWorldObjectSample>> =
            serde_json::from_value(row["ground"].clone()).unwrap();
        assert_eq!(
            scan_directory_research_ground_weapon_samples(dir, bounds),
            expected
        );
    }
    #[test]
    fn native_research_directory_defaults_and_failures() {
        let root = std::env::temp_dir().join(format!("halo-research-dir-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let rows = cases();
        assert_eq!(rows.len(), 34);
        for (i, row) in rows[..32].iter().enumerate() {
            let dir = root.join(i.to_string());
            if !row["missing"].as_bool().unwrap() {
                std::fs::create_dir(&dir).unwrap();
            }
            let files: BTreeMap<String, String> =
                serde_json::from_value(row["files"].clone()).unwrap();
            for (name, h) in files {
                let data: Vec<u8> = (0..h.len())
                    .step_by(2)
                    .map(|p| u8::from_str_radix(&h[p..p + 2], 16).unwrap())
                    .collect();
                std::fs::write(dir.join(name), data).unwrap();
            }
            check(row, &dir);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "requires local v41 film corpus"]
    fn local_research_directory_detected_and_fallback_precision() {
        let root =
            std::env::temp_dir().join(format!("halo-research-corpus-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        for (i, row) in cases()[32..].iter().enumerate() {
            let dir = root.join(i.to_string());
            std::fs::create_dir(&dir).unwrap();
            let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(row["folder"].as_str().unwrap());
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(source.join("film.json")).unwrap()).unwrap();
            for chunk in manifest["chunks"].as_array().unwrap() {
                let index = chunk["index"].as_i64().unwrap();
                if index > 7 {
                    continue;
                }
                std::fs::copy(
                    source.join(chunk["file"].as_str().unwrap()),
                    dir.join(format!("chunk_{index:02}.bin")),
                )
                .unwrap();
            }
            check(row, &dir);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
