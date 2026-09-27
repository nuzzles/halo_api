use super::facts_creations_tests::{world, world_dump};
use super::facts_placement_stats_tests::{dump as dump_stats, source as stats};
use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn parse_world(v: &Value) -> FactsWorldSection {
    FactsWorldSection {
        placements: v["Placements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| FactsPlacement {
                start_us: p["T0US"].as_u64().unwrap(),
                end_us: p["T1US"].as_u64().unwrap(),
                life: serde_json::from_value(p["Life"].clone()).unwrap(),
                position: ["X", "Y", "Z"].map(|a| f32::from_bits(p[a].as_u64().unwrap() as u32)),
                global_id: p["GlobalID"].as_u64().unwrap() as u32,
                points: p["Points"].as_i64().unwrap(),
            })
            .collect(),
        placement_stats: stats(&v["PlacementStats"]),
        spawn_events: serde_json::from_value(v["SpawnEvents"].clone()).unwrap(),
        spawn_stats: serde_json::from_value(v["SpawnStats"].clone()).unwrap(),
        weapons: world(&v["Pads"]["Weapons"]),
        powerups: world(&v["Pads"]["Powerups"]),
    }
}
fn dump_world(g: &FactsWorldSection) -> Value {
    json!({"Placements":g.placements.iter().map(|p|json!({"T0US":p.start_us,"T1US":p.end_us,"Life":p.life,"X":p.position[0].to_bits(),"Y":p.position[1].to_bits(),"Z":p.position[2].to_bits(),"GlobalID":p.global_id,"Points":p.points})).collect::<Vec<_>>(),"PlacementStats":dump_stats(&g.placement_stats),"SpawnEvents":g.spawn_events,"SpawnStats":g.spawn_stats,"Pads":{"Weapons":world_dump(&g.weapons),"Powerups":world_dump(&g.powerups)}})
}
#[test]
fn native_facts_tail_codecs() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-tail-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let f: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(f["cases"].as_array().unwrap().len(), 28713);
    for (i, row) in f["cases"].as_array().unwrap().iter().enumerate() {
        let input = unhex(row["input"].as_str().unwrap());
        let mut r = NativeFactsReader::new(&input);
        let mut w = NativeFactsWriter::default();
        let v = &row["source"];
        let got = match row["kind"].as_str().unwrap() {
            "world" => {
                encode_facts_world_section(&mut w, &parse_world(v));
                dump_world(&decode_facts_world_section(&mut r))
            }
            "spawns" => {
                let events: Vec<FactsSpawnEvent> =
                    serde_json::from_value(v["SpawnEvents"].clone()).unwrap();
                encode_facts_spawns(
                    &mut w,
                    &events,
                    &serde_json::from_value(v["SpawnStats"].clone()).unwrap(),
                );
                let (events, stats) = decode_facts_spawns(&mut r);
                json!({"SpawnEvents":events,"SpawnStats":stats})
            }
            "queue" => {
                encode_facts_queue(&mut w, &serde_json::from_value(v.clone()).unwrap());
                json!(decode_facts_queue(&mut r))
            }
            "table" => {
                encode_facts_player_table(
                    &mut w,
                    &serde_json::from_value(v["FilmTable"].clone()).unwrap(),
                );
                json!({"FilmTable":decode_facts_player_table(&mut r)})
            }
            "teams" => {
                let teams: Option<std::collections::BTreeMap<i64, i64>> =
                    serde_json::from_value(v["PlayerTeams"].clone()).unwrap();
                encode_facts_player_teams(
                    &mut w,
                    teams.as_ref(),
                    &serde_json::from_value(v["TeamScan"].clone()).unwrap(),
                );
                let (teams, scan) = decode_facts_player_teams(&mut r);
                json!({"PlayerTeams":teams,"TeamScan":scan})
            }
            other => panic!("unexpected {other}"),
        };
        assert_eq!(
            w.bytes(),
            unhex(row["encoded"].as_str().unwrap()),
            "writer {i}"
        );
        assert_eq!(r.offset(), row["offset"], "cursor {i}");
        if row["panic"].as_bool().unwrap() {
            assert!(r.error().is_some(), "safe panic {i}");
            continue;
        }
        assert_eq!(r.error().unwrap_or(""), row["error"], "error {i}");
        assert_eq!(got, row["decoded"], "fields {i}");
    }
}
