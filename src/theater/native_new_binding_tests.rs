use super::*;
use std::io::Read;
#[test]
fn native_new_binding_d61443e() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/new-binding-d61443e-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 15);
    let registry = FilmRegistry {
        archetypes: (0..4)
            .map(|i| FilmArchetype {
                index: i,
                components: if i == 3 {
                    vec!["high-frequency".into()]
                } else {
                    vec![]
                },
                levels: if i == 3 { vec![1] } else { vec![] },
            })
            .collect(),
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
    };
    for row in rows {
        let hex = row["hex"].as_str().unwrap();
        let payload: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        for inference in [false, true] {
            let mut world = FilmWorld::default();
            match row["mode"].as_u64().unwrap() {
                1 => world.bind_full((1 << 30) | 1, 2),
                2 => world.bind_full((1 << 30) | 1, 3),
                3 => world.bind_soft((1 << 30) | 1, 2),
                4 => {
                    world.bind_full((1 << 30) | 1, 2);
                    world.unbind(1);
                }
                _ => {}
            }
            let observer = NativeFilmObserver::default();
            let mut config = NativeFrameConfig {
                id_low_bits: 5,
                ..Default::default()
            };
            config.context.profile.grammar.generation_strict = true;
            config.context.observer = Some(observer.clone());
            let (views, end, records, diagnostics) = if inference {
                let frame = config
                    .decode_inference_views(&payload, 2, 3, &registry, &mut world)
                    .unwrap();
                let records: Vec<_> = frame
                    .entities
                    .iter()
                    .flat_map(|v| v.records.iter().filter_map(|r| r.decoded.clone()))
                    .collect();
                let mut d = FilmReadDiagnostics::default();
                for v in &frame.entities {
                    d.merge(&v.diagnostics);
                }
                (frame.views_completed, frame.end_bit, records, d)
            } else {
                let frame = config
                    .decode_production_views(&payload, 2, &registry, &mut world)
                    .unwrap();
                (
                    frame.views_completed,
                    frame.end_bit,
                    frame.records,
                    frame.header_diagnostics,
                )
            };
            assert_eq!(views, row["views"]);
            assert_eq!(end, row["end"]);
            assert_eq!(world.archetype(1).unwrap(), row["archetype"]);
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].header.kind, RecordKind::New);
            assert_eq!(records[0].archetype, Some(3));
            assert!(!records[0].fields.is_empty());
            assert_eq!(diagnostics.new_binding_refusals.len(), row["rejected"]);
            assert_eq!(
                observer.counters().new_binding_refusals,
                diagnostics.new_binding_refusals
            );
            if let Some(refusal) = diagnostics.new_binding_refusals.first() {
                assert_eq!(
                    (
                        refusal.record_bit,
                        refusal.slot,
                        refusal.existing_archetype,
                        refusal.proposed_archetype
                    ),
                    (2, 1, 2, 3)
                );
                assert_eq!(refusal.id, records[0].header.id.unwrap());
                assert!(!diagnostics.is_empty());
            }
            assert_eq!(
                serde_json::from_value::<FilmReadDiagnostics>(
                    serde_json::to_value(&diagnostics).unwrap()
                )
                .unwrap(),
                diagnostics
            );
        }
    }
}
