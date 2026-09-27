use super::*;
use serde_json::Value;
use std::io::Read;
#[test]
fn native_loaded_source_inventory() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/inventory-loaded-source-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 360);
    let (mut count, mut errors, mut positive) = (0, 0, 0);
    for (i, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut metadata = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            let h = c["hex"].as_str().unwrap();
            buffers.push(
                (0..h.len())
                    .step_by(2)
                    .map(|p| u8::from_str_radix(&h[p..p + 2], 16).unwrap())
                    .collect::<Vec<_>>(),
            );
            metadata.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: c["chunk_type"].as_i64().unwrap(),
                start_ms: 0,
            });
        }
        let source = (!buffers.is_empty()).then(|| FilmSource::load(&buffers, &metadata).unwrap());
        let mode = row["catalog_mode"].as_u64().unwrap();
        let known = if mode == 0 {
            Default::default()
        } else {
            [(0xabcedf13, mode == 1), (0xdeadbeef, mode == 1)].into()
        };
        let cap = row["cap"].as_u64().unwrap() as u32;
        let counter = FallbackCounter::default();
        counter.trigger_n(DEFAULT_GRENADE_CAP_FALLBACK, 4);
        let present = row["counter_present"].as_bool().unwrap();
        let (out, error) = scan_source_keyframe_inventory(
            source.as_ref(),
            &known,
            cap,
            present.then_some(&counter),
        );
        assert_eq!(
            error.is_some(),
            row["error"].as_bool().unwrap(),
            "error {i}"
        );
        assert_eq!(serde_json::json!(out.stats), row["stats"], "stats {i}");
        assert_eq!(
            counter.count(DEFAULT_GRENADE_CAP_FALLBACK) - 4,
            row["fallbacks"].as_i64().unwrap(),
            "fallback {i}"
        );
        assert_eq!(out.default_grenade_max, mode != 0 && cap == 0);
        assert_eq!(
            out.grenade_max,
            if mode == 0 {
                0
            } else if cap == 0 {
                2
            } else {
                cap
            }
        );
        let mut expected: Vec<KeyframeInventory> =
            serde_json::from_value(row["records"].clone()).unwrap();
        for (inv, bits) in expected
            .iter_mut()
            .zip(row["gauge_bits"].as_array().unwrap())
        {
            for (ammo, value) in inv.ammo.iter_mut().zip(bits.as_array().unwrap()) {
                ammo.gauge = value.as_u64().map(f64::from_bits);
                ammo.gauge_quantum = ammo.gauge.map(|g| (g * 4095.).round() as u16);
            }
        }
        assert_eq!(out.records, expected, "records {i}");
        for (inv, native) in out.records.iter().zip(&expected) {
            let (_, packets) = source.as_ref().unwrap().chunk_by_number(inv.chunk).unwrap();
            assert_eq!(packets[inv.packet_index].timestamp_us, inv.timestamp_us);
            assert_eq!(packets[inv.packet_index].packet_type, 2);
            assert_eq!(
                FactsKeyframeInventory::from(inv),
                FactsKeyframeInventory {
                    timestamp_us: native.timestamp_us,
                    slot: native.slot,
                    grenades_read: native.grenades_read,
                    grenades: native.grenades,
                    selected_grenade_rank: i64::from(native.selected_grenade_rank),
                    ability_rank: i64::from(native.ability_rank),
                    drawn_slot: i64::from(native.drawn_slot),
                    ammo_candidates: native.ammo_candidates as i64,
                    ammo_read: native.ammo_read,
                    ammo: native.ammo.clone(),
                }
            );
        }
        count += out.records.len();
        positive += usize::from(!out.records.is_empty());
        errors += usize::from(error.is_some());
    }
    assert_eq!((count, positive, errors), (288, 72, 120));
}

#[test]
fn native_context_pickups() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/pickup-padding-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    let expected_reads: std::collections::BTreeMap<_, _> = fixture["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["hex"].as_str().unwrap(), r))
        .collect();
    let rows = fixture["sources"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    let mut published = 0;
    for row in rows {
        let h = row["hex"].as_str().unwrap();
        let data: Vec<_> = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect();
        let source = FilmSource::load(
            &[data],
            &[FilmSourceMetadata {
                index: row["index"].as_i64().unwrap(),
                chunk_type: 2,
                start_ms: 0,
            }],
        )
        .unwrap();
        let context = NativeFilmContext::new(Some(&source));
        let out = scan_context_biped_pickups(&context);
        let (facts, stats) = facts_from_pickup_scan(&out);
        assert_eq!(serde_json::json!(stats), row["stats"]);
        assert_eq!(out.no_film_chunks, row["no_chunks"].as_bool().unwrap());
        let attempts = row["attempts"].as_array().unwrap();
        assert_eq!(out.attempts.len(), attempts.len());
        let mut expected_facts = Vec::new();
        for (a, e) in out.attempts.iter().zip(attempts) {
            assert_eq!(a.packet_index as u64, e["packet"].as_u64().unwrap());
            assert_eq!(a.source.payload_offset as u64, e["start"].as_u64().unwrap());
            assert_eq!(a.source.payload_size as u64, e["size"].as_u64().unwrap());
            assert_eq!(a.source.timestamp_us, e["time"].as_u64().unwrap());
            assert_eq!(a.published, e["published"].as_bool().unwrap());
            let payload = source.payload(&a.source).unwrap();
            let hex: String = payload.iter().map(|b| format!("{b:02x}")).collect();
            if a.published {
                let native = &expected_reads[hex.as_str()]["event"];
                expected_facts.push(FactsPickup {
                    timestamp_us: e["time"].as_u64().unwrap(),
                    slot: native["Slot"].as_u64().unwrap() as u32,
                    catalog_id: native["CatalogID"].as_u64().unwrap() as u32,
                    class: native["Class"].as_u64().unwrap() as u8,
                });
            }
        }
        assert_eq!(facts, expected_facts);
        published += facts.len();
    }
    assert!(published > 0);
    assert!(scan_context_biped_pickups(&NativeFilmContext::new(None)).no_film_chunks);
}
