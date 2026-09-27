use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn u(v: &Value, k: &str) -> u64 {
    v[k].as_u64().unwrap()
}
fn i(v: &Value, k: &str) -> i64 {
    v[k].as_i64().unwrap()
}
fn b(v: &Value, k: &str) -> bool {
    v[k].as_bool().unwrap()
}
fn array(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn nums(v: &Value) -> Option<Vec<u32>> {
    (!v.is_null()).then(|| {
        array(v)
            .iter()
            .map(|x| x.as_u64().unwrap() as u32)
            .collect()
    })
}
fn ammo(v: &Value) -> KeyframeSlotAmmo {
    KeyframeSlotAmmo {
        mag: v["Mag"].as_u64().map(|n| n as u32),
        res: v["Res"].as_u64().map(|n| n as u32),
        gauge: v["Gauge"].as_u64().map(f64::from_bits),
        gauge_quantum: None,
        overheat: u(v, "Overheat") as u32,
        flags: u(v, "Flags") as u32,
    }
}
fn dump_ammo(v: &KeyframeSlotAmmo) -> Value {
    assert!(v.gauge_quantum.is_none());
    json!({"Mag":v.mag,"Res":v.res,"Gauge":v.gauge.map(f64::to_bits),"Overheat":v.overheat,"Flags":v.flags})
}
fn events(v: &Value) -> FactsEvents {
    FactsEvents {
        fire: array(&v["Fire"])
            .iter()
            .map(|e| FactsFireEvent {
                timestamp_us: u(e, "TimestampUS"),
                film_index: i(e, "FilmIndex"),
                weapon_id: u(e, "WeaponID"),
                has_aim: b(e, "HasAim"),
                aim: std::array::from_fn(|a| f32::from_bits(e["Aim"][a].as_u64().unwrap() as u32)),
            })
            .collect(),
        loadouts: array(&v["Loadouts"])
            .iter()
            .map(|l| FactsLoadout {
                timestamp_us: u(l, "TimestampUS"),
                slot: u(l, "Slot") as u32,
                families: nums(&l["Families"]),
            })
            .collect(),
        grenades: array(&v["Grenades"])
            .iter()
            .map(|e| FactsGrenadeThrow {
                timestamp_us: u(e, "TimestampUS"),
                film_index: i(e, "FilmIndex"),
                type_id: u(e, "TypeID") as u32,
            })
            .collect(),
        projectiles: array(&v["Projectiles"])
            .iter()
            .map(|t| FactsProjectileTrack {
                slot: u(t, "Slot") as u32,
                generation: u(t, "Gen") as u32,
                points: (!t["Pts"].is_null()).then(|| {
                    array(&t["Pts"])
                        .iter()
                        .map(|p| FactsProjectileSample {
                            timestamp_us: u(p, "TimestampUS"),
                            chunk: i(p, "Chunk"),
                            position: ["X", "Y", "Z"].map(|a| f32::from_bits(u(p, a) as u32)),
                            at_rest: b(p, "AtRest"),
                        })
                        .collect()
                }),
            })
            .collect(),
    }
}
fn dump_events(g: &FactsEvents) -> Value {
    json!({
        "Fire":g.fire.iter().map(|e| json!({"TimestampUS":e.timestamp_us,"FilmIndex":e.film_index,"WeaponID":e.weapon_id,"HasAim":e.has_aim,"Aim":e.aim.map(f32::to_bits)})).collect::<Vec<_>>(),
        "Loadouts":g.loadouts.iter().map(|e| json!({"TimestampUS":e.timestamp_us,"Slot":e.slot,"Families":e.families})).collect::<Vec<_>>(),
        "Grenades":g.grenades.iter().map(|e| json!({"TimestampUS":e.timestamp_us,"FilmIndex":e.film_index,"TypeID":e.type_id})).collect::<Vec<_>>(),
        "Projectiles":g.projectiles.iter().map(|t| json!({"Slot":t.slot,"Gen":t.generation,"Pts":t.points.as_ref().map(|ps| ps.iter().map(|p| json!({"TimestampUS":p.timestamp_us,"Chunk":p.chunk,"X":p.position[0].to_bits(),"Y":p.position[1].to_bits(),"Z":p.position[2].to_bits(),"AtRest":p.at_rest})).collect::<Vec<_>>())})).collect::<Vec<_>>()
    })
}
fn inventory(v: &Value) -> FactsInventory {
    FactsInventory {
        inventory: array(&v["Inventory"])
            .iter()
            .map(|e| FactsKeyframeInventory {
                timestamp_us: u(e, "TimestampUS"),
                slot: u(e, "Slot") as u32,
                grenades_read: b(e, "GrenadesRead"),
                grenades: std::array::from_fn(|a| e["Grenades"][a].as_u64().unwrap() as u32),
                selected_grenade_rank: i(e, "SelectedGrenadeRank"),
                ability_rank: i(e, "AbilityRank"),
                drawn_slot: i(e, "DrawnSlot"),
                ammo_candidates: i(e, "AmmoCandidates"),
                ammo_read: b(e, "AmmoRead"),
                ammo: std::array::from_fn(|a| ammo(&e["Ammo"][a])),
            })
            .collect(),
        deltas: array(&v["InventoryDeltas"])
            .iter()
            .map(|e| FactsInventoryDelta {
                timestamp_us: u(e, "TimestampUS"),
                slot: u(e, "Slot") as u32,
                grenades: nums(&e["Grenades"]),
                selection_read: b(e, "SelRead"),
                selection: i(e, "Sel"),
                mask: u(e, "Mask") as u32,
            })
            .collect(),
    }
}
fn dump_inventory(g: &FactsInventory) -> Value {
    json!({
        "Inventory":g.inventory.iter().map(|e| json!({"TimestampUS":e.timestamp_us,"Slot":e.slot,"GrenadesRead":e.grenades_read,"Grenades":e.grenades,"SelectedGrenadeRank":e.selected_grenade_rank,"AbilityRank":e.ability_rank,"DrawnSlot":e.drawn_slot,"AmmoCandidates":e.ammo_candidates,"AmmoRead":e.ammo_read,"Ammo":e.ammo.iter().map(dump_ammo).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "InventoryDeltas":g.deltas.iter().map(|e| json!({"TimestampUS":e.timestamp_us,"Slot":e.slot,"Grenades":e.grenades,"SelRead":e.selection_read,"Sel":e.selection,"Mask":e.mask})).collect::<Vec<_>>()
    })
}
#[test]
fn native_facts_event_inventory_codecs() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-body-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(array(&fixture["cases"]).len(), 15928);
    for (n, row) in array(&fixture["cases"]).iter().enumerate() {
        let input = unhex(row["input"].as_str().unwrap());
        let mut r = NativeFactsReader::new(&input);
        let mut w = NativeFactsWriter::default();
        let got = if row["kind"] == "events" {
            encode_facts_events(&mut w, &events(&row["source"]));
            dump_events(&decode_facts_events(&mut r))
        } else {
            encode_facts_inventory(&mut w, &inventory(&row["source"]));
            dump_inventory(&decode_facts_inventory(&mut r))
        };
        assert_eq!(
            w.bytes(),
            unhex(row["encoded"].as_str().unwrap()),
            "writer {n}"
        );
        assert_eq!(r.offset(), row["offset"], "cursor {n}");
        if b(row, "panic") {
            assert!(r.error().is_some(), "safe panic refusal {n}");
            continue;
        }
        assert_eq!(r.error().unwrap_or(""), row["error"], "error {n}");
        assert_eq!(got, row["decoded"], "fields {n}");
    }
}
