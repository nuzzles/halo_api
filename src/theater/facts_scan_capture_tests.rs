use super::*;
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_scan_capture_boundary() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/scan-capture-v41.json.zlib")[..])
        .read_to_end(&mut bytes)
        .unwrap();
    let fixture: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let input = decode_film_facts_file(
        &unhex(fixture["input"].as_str().unwrap()),
        &FactsMapEntry {
            module: b"map".to_vec(),
            axis_widths: [13, 14, 15],
            bounds: [[-10., 100.], [-30., 300.], [-50., 500.]],
            ..Default::default()
        },
    )
    .unwrap();
    let rows = fixture["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 24);
    let (mut successful, mut detected, mut rejected_candidate) = (0, 0, 0);
    for (i, row) in rows.iter().enumerate() {
        let chunks = row["chunks"].as_array().unwrap();
        let buffers: Vec<_> = chunks
            .iter()
            .map(|c| unhex(c["hex"].as_str().unwrap()))
            .collect();
        let metadata: Vec<_> = chunks
            .iter()
            .map(|c| FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            })
            .collect();
        let source = (!buffers.is_empty()).then(|| FilmSource::load(&buffers, &metadata).unwrap());
        let imposed = (!row["imposed"].is_null()).then(|| I0Layout {
            gate_bits: row["imposed"]["GateBits"].as_i64().unwrap(),
            axis_widths: serde_json::from_value(row["imposed"]["AxisW"].clone()).unwrap(),
            region: row["imposed"]["Region"].as_u64().unwrap() as u32,
        });
        let context = NativeFilmContext::with_imposed_layout(source.as_ref(), imposed.as_ref());
        let map = row["map_present"]
            .as_bool()
            .unwrap()
            .then(|| FactsMapEntry {
                module: unhex(row["module_hex"].as_str().unwrap()),
                axis_widths: [13, 14, 15],
                ..Default::default()
            });
        let mut identity = FactsIdentityJsonReader::default()
            .read(&serde_json::to_vec(&row["identity"]).unwrap())
            .unwrap();
        let counter = FallbackCounter::default();
        counter.trigger_n("repli_manche_zero_decretee", row["hits"].as_i64().unwrap());
        let captured = capture_film_scan_facts(
            b"capture\0film",
            &context,
            map.as_ref(),
            &input.facts,
            &input.mode_guards,
            identity.as_ref(),
            row["counter_present"]
                .as_bool()
                .unwrap()
                .then_some(&counter),
        );
        if row["expected"].is_null() {
            assert!(captured.is_none(), "unexpected capture {i}");
            rejected_candidate += usize::from(context.i0_layout().layout.is_some());
            continue;
        }
        let captured = captured.unwrap();
        successful += 1;
        detected += usize::from(captured.facts.header.layout_detected);
        let expected = unhex(row["expected"].as_str().unwrap());
        assert_eq!(
            encode_film_facts_file(&captured).unwrap(),
            expected,
            "captured file {i}"
        );
        if let Some(id) = &mut identity {
            id.build = b"mutated".to_vec();
        }
        counter.trigger_n("repli_manche_zero_decretee", 99);
        assert_eq!(
            encode_film_facts_file(&captured).unwrap(),
            expected,
            "snapshot changed {i}"
        );
        assert_eq!(input.facts.header.axis_widths, [13, 14, 15]);
    }
    assert_eq!((successful, detected), (12, 4));
    assert!(rejected_candidate > 0);
}
