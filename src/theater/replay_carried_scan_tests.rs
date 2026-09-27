use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(h: &str) -> Vec<u8> {
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_carried_scan_phase() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/carried-scan-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 32);
    let mut counts = [0; 4];
    let mut log_count = 0;
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            buffers.push(unhex(c["hex"].as_str().unwrap()));
            meta.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = (!buffers.is_empty()).then(|| FilmSource::load(&buffers, &meta).unwrap());
        let layout = I0Layout {
            gate_bits: 5,
            axis_widths: [8, 9, 10],
            region: 0,
        };
        let context = NativeFilmContext::for_map(source.as_ref(), None, Some(&layout)).unwrap();
        let loadouts: Vec<KeyframeLoadout> = serde_json::from_value(
            row["loadouts"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into(),
        )
        .unwrap();
        let counter = FallbackCounter::default();
        counter.trigger_n(DEFAULT_GRENADE_CAP_FALLBACK, 4);
        let fb = (row["counter"] == true).then_some(&counter);
        let mut observations = Vec::new();
        let mut output = None;
        let logs = super::log_test_support::capture_logs(|| {
            output = Some(scan_replay_carried_inputs(
                b"carried",
                &context,
                &loadouts,
                fb,
                |o| {
                    let value=match &o {
                ReplayCarriedObservation::HeldWeapons(v)=>json!(v),ReplayCarriedObservation::HeldWeaponStats(v)=>json!(v),
                ReplayCarriedObservation::Pickups(v)=>json!(v.iter().map(|p|json!({"TimestampUS":p.timestamp_us,"Slot":p.slot,"CatalogID":p.catalog_id,"Class":p.class})).collect::<Vec<_>>()),
                ReplayCarriedObservation::PickupStats(v)=>json!(v),
                ReplayCarriedObservation::Inventory(v)=>json!(v.len()),ReplayCarriedObservation::InventoryStats(v)=>json!(v),
                ReplayCarriedObservation::Deltas(v)=>json!(v.iter().map(|r|{
                    let f=FactsInventoryDelta::from(r);let ammo:Vec<_>=r.ammo.iter().map(|a|json!({"WeaponSlot":a.weapon_slot,"Mag":a.magazine,"FracQ":a.fraction_quantum,"Res":a.reserve})).collect();
                    json!({"Slot":f.slot,"Chunk":r.chunk_number.unwrap(),"PacketIndex":r.packet_index.unwrap(),"TimestampUS":f.timestamp_us,"Grenades":f.grenades,"SelRead":f.selection_read,"Sel":f.selection,"Mask":f.mask,"Ammo":if ammo.is_empty(){Value::Null}else{json!(ammo)}})
                }).collect::<Vec<_>>()),ReplayCarriedObservation::DeltaStats(v)=>json!(v),
            };
                    observations.push(json!({"step":o.name(),"value":value,"fallback":fb.map_or(0,|c|c.count(DEFAULT_GRENADE_CAP_FALLBACK))}));
                },
            ));
        });
        assert_eq!(json!(logs), row["logs"], "logs {case}");
        log_count += logs.len();
        let mut expected = row["observations"].clone();
        for o in expected.as_array_mut().unwrap() {
            let step = o["step"].as_str().unwrap().to_owned();
            if step == "inventory" {
                o["value"] = json!(o["value"].as_array().map_or(0, Vec::len));
            } else if !step.ends_with(".stats") {
                if o["value"].is_null() {
                    o["value"] = json!([]);
                }
                if step == "pickups" {
                    for p in o["value"].as_array_mut().unwrap() {
                        p.as_object_mut().unwrap().remove("Chunk");
                    }
                }
            }
        }
        assert_eq!(json!(observations), expected, "observations {case}");
        let out = output.unwrap();
        counts[0] += out.published_weapons().len();
        counts[1] += out.published_pickups.len();
        counts[2] += out.published_inventory().len();
        counts[3] += out.published_deltas().len();
        let mut facts = NativeFilmFacts::default();
        facts.events.loadouts = loadouts
            .iter()
            .map(|l| FactsLoadout {
                timestamp_us: l.timestamp_us,
                slot: l.slot,
                families: Some(l.families.clone()),
            })
            .collect();
        out.apply_to_facts(&mut facts);
        let captured = capture_film_scan_facts(
            b"carried",
            &context,
            None,
            &facts,
            &FactsModeGuards::default(),
            None,
            fb,
        )
        .unwrap();
        assert_eq!(
            encode_film_facts_file(&captured).unwrap(),
            unhex(row["facts"].as_str().unwrap()),
            "complete facts file {case}"
        );
    }
    assert!(counts.iter().all(|&n| n > 0));
    assert_eq!(log_count, 128);
}
