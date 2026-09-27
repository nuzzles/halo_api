use super::*;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::Read,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
#[test]
fn native_context_mpp_calibration() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/context-mpp-calibration-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 32);
    let mut total_anchors = 0;
    let mut conclusive = 0;
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
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let mut context = NativeFilmContext::new(Some(&source));
        let mut profile = NativeScanProfile {
            mpp: FilmMppWidths { lead: 11, index: 7 },
            ..Default::default()
        };
        profile.movement.world_object = NativePrecisionDescriptor {
            index_bits: 1,
            axis_bits: [16, 17, 18],
            region: 0,
        };
        context.set_scan_profile(profile).unwrap();
        let before = context.scan_profile().unwrap().mpp;
        let shared = Arc::new(AtomicUsize::new(0));
        let seen = shared.clone();
        context.observation().set_hook(
            NativeHookKind::Mpp,
            Some(Arc::new(move |_| {
                seen.fetch_add(1, Ordering::Relaxed);
            })),
        );
        let mut spans = EquipmentLifeSpans::new();
        for s in row["spans"].as_array().unwrap() {
            let key: EquipmentLifeKey = serde_json::from_value(s["key"].clone()).unwrap();
            let span: EquipmentLifeSpan = serde_json::from_value(s["span"].clone()).unwrap();
            spans.entry(key).or_default().push(span);
        }
        let band: BTreeSet<u32> = if row["empty_band"] == true {
            BTreeSet::new()
        } else {
            [512, 513, 515].into()
        };
        let map = FilmMapBounds {
            module: String::new(),
            min: [-100., -200., -300.],
            max: [300., 400., 500.],
            axis_widths: [16, 17, 18],
            region: 0,
            region_index_bits: 1,
        };
        let mut attempts = 0;
        let mut scores = std::collections::BTreeMap::new();
        let out = calibrate_context_equipment_mpp(
            &context,
            (row["no_bounds"] != true).then_some(&map),
            &band,
            &spans,
            |a| {
                assert_eq!(meta[a.source.chunk_index as usize].index, a.chunk);
                assert_eq!(a.read.default_state.start_bit, a.bit as i64 + 24);
                if a.matched_span.is_some() {
                    *scores
                        .entry((a.widths.lead, a.widths.index))
                        .or_insert(0usize) += 1;
                    assert!(a.read.creation.is_some());
                }
                attempts += 1;
            },
        )
        .unwrap();
        let expected: EquipmentMppCalibration =
            serde_json::from_value(row["calibration"].clone()).unwrap();
        assert_eq!(out.calibration, expected, "calibration {case}");
        assert_eq!(
            out.conclusive,
            row["conclusive"].as_bool().unwrap(),
            "verdict {case}"
        );
        assert_eq!(attempts, out.calibration.anchors * 63);
        for score in &out.calibration.by_widths {
            assert_eq!(
                scores.remove(&(score.widths.lead, score.widths.index)),
                Some(score.agree)
            );
        }
        assert!(scores.is_empty());
        assert_eq!(context.scan_profile().unwrap().mpp, before);
        assert_eq!(shared.load(Ordering::Relaxed), 0);
        assert_eq!(row["restored"], true);
        total_anchors += out.calibration.anchors;
        conclusive += usize::from(out.conclusive);
    }
    assert_eq!((total_anchors, conclusive), (549, 6));
}
