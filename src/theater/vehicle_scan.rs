//! Native vehicle position scan over the census's exact ti=40 slot band.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use std::collections::BTreeSet;

/// Use the pinned v41 vehicle settings: all generations, dynamic i2/i3, and
/// production saturation/isolation/speed filters. The map supplies i0 precision.
pub fn scan_vehicle_positions(
    chunks: &[FilmChunkData],
    band: &BTreeSet<u32>,
    map: &FilmMapBounds,
) -> Result<BipedPositionStream, DecodeError> {
    scan_positions_for_band(
        chunks,
        band,
        map,
        BipedScanOptions {
            require_generation_one: false,
            ..Default::default()
        },
        Some(2),
        &[],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Record {
        time: u64,
        start: usize,
        position: usize,
        end: usize,
        slot: u32,
        generation: u8,
        mask: Vec<u8>,
        q: [u32; 3],
        world: [f32; 3],
        dirs: serde_json::Value,
        vitals: serde_json::Value,
    }
    #[derive(Deserialize)]
    struct Case {
        map: FilmMapBounds,
        data: Vec<u8>,
        band: BTreeSet<u32>,
        options: BipedScanOptions,
        records: Vec<Record>,
        accepted: Vec<usize>,
    }
    #[test]
    fn native_sparse_vehicle_band_scan() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/vehicle-scan-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let chunk = FilmChunkData {
                metadata: FilmChunk {
                    index: 1,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: c.data.len() as i64,
                    file_relative_path: String::new(),
                },
                data: c.data,
            };
            let out = scan_positions_for_band(&[chunk], &c.band, &c.map, c.options, Some(2), &[])
                .unwrap();
            assert_eq!(out.candidates.len(), c.records.len(), "count {i}");
            for (r, e) in out.candidates.iter().zip(c.records) {
                assert_eq!(r.source.timestamp_us, e.time, "time {i}");
                let r = &r.record;
                assert_eq!(
                    (r.start_bit, r.position_bit, r.end_bit, r.slot, r.generation),
                    (e.start, e.position, e.end, e.slot, e.generation),
                    "record {i}"
                );
                assert_eq!(r.component_indices, e.mask, "mask {i}");
                assert_eq!(r.quantized, e.q, "quanta {i}");
                assert_eq!(r.world, e.world, "world {i}");
                super::super::biped_capture::tests::assert_native(
                    &r.companions,
                    &e.dirs,
                    &e.vitals,
                );
            }
            assert_eq!(
                out.candidates
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| r.rejection.is_none())
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>(),
                c.accepted,
                "filter {i}"
            );
        }
    }
}
