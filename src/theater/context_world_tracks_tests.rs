use super::*;
use std::{collections::BTreeSet, io::Read};
#[test]
fn native_context_world_tracks() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-world-tracks-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 112);
    let (mut records, mut tracks) = (0, 0);
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            let h = c["hex"].as_str().unwrap();
            buffers.push(
                (0..h.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect::<Vec<_>>(),
            );
            meta.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = (!buffers.is_empty()).then(|| FilmSource::load(&buffers, &meta).unwrap());
        let mut context = NativeFilmContext::new(source.as_ref());
        let axis_bits: [u64; 3] = serde_json::from_value(row["widths"].clone()).unwrap();
        context.set_world_precision(NativePrecisionDescriptor {
            index_bits: row["index_bits"].as_u64().unwrap(),
            axis_bits,
            region: row["region"].as_u64().unwrap() as u32,
        });
        // Deliberately different widths: the context, not these map defaults,
        // must govern the packet read.
        let map = FilmMapBounds {
            module: String::new(),
            min: [-100., -200., -300.],
            max: [300., 400., 500.],
            axis_widths: [6; 3],
            region: 0,
            region_index_bits: 1,
        };
        let band: BTreeSet<u32> = if row["empty_band"] == true {
            BTreeSet::new()
        } else {
            [512, 513, 515].into()
        };
        let out = scan_context_world_object_tracks_for_band(
            &context,
            (row["no_bounds"] != true).then_some(&map),
            &band,
        );
        if row["error"] != "" {
            assert_eq!(
                out.unwrap_err().to_string(),
                row["error"].as_str().unwrap(),
                "error {case}"
            );
            continue;
        }
        let out = out.unwrap();
        let expected: Vec<WorldObjectTrack> =
            serde_json::from_value(row["tracks"].clone()).unwrap();
        assert_eq!(out.tracks, expected, "tracks {case}");
        let expected = row["records"].as_array().unwrap();
        assert_eq!(out.records.len(), expected.len(), "records {case}");
        for (r, e) in out.records.iter().zip(expected) {
            let sample: WorldObjectSample = serde_json::from_value(e["sample"].clone()).unwrap();
            assert_eq!(r.record.sample, sample, "sample {case}");
            assert_eq!(r.record.slot as u64, e["slot"].as_u64().unwrap());
            assert_eq!(
                r.record.generation as u64,
                e["generation"].as_u64().unwrap()
            );
            assert_eq!(r.packet_index as u64, e["packet"].as_u64().unwrap());
            assert_eq!(r.source.payload_offset as u64, e["start"].as_u64().unwrap());
            assert_eq!(
                meta[r.source.chunk_index as usize].index,
                r.record.sample.chunk
            );
            assert!(r.record.bit < r.source.payload_size * 8);
        }
        records += out.records.len();
        tracks += out.tracks.len();
    }
    assert_eq!((records, tracks), (1722, 138));
}
