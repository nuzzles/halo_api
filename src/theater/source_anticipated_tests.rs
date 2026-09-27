use super::*;
use serde_json::{Value, json};
use std::io::Read;
#[test]
fn native_source_anticipated_domain() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/anticipated-domain-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 160);
    for (case, row) in rows.iter().enumerate() {
        let table = if row["loaded"] == true {
            let mut buffers = Vec::new();
            let mut meta = Vec::new();
            for c in row["inputs"].as_array().unwrap() {
                let h = c["Hex"].as_str().unwrap();
                buffers.push(
                    (0..h.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                        .collect::<Vec<_>>(),
                );
                meta.push(FilmSourceMetadata {
                    index: c["Index"].as_i64().unwrap(),
                    chunk_type: 0,
                    start_ms: 0,
                });
            }
            let source = FilmSource::load(&buffers, &meta).unwrap();
            build_source_anticipated_bindings(Some(&source))
        } else {
            let declarations: Vec<(u32, i64, u32)> =
                serde_json::from_value(row["declarations"].clone()).unwrap();
            AnticipatedBindings::from_declarations(declarations)
        };
        assert_eq!(json!(table), row["table"], "full table {case}");
        for q in row["queries"].as_array().unwrap() {
            let id = q["id"].as_u64().unwrap() as u32;
            let chunk = q["chunk"].as_i64().unwrap();
            let expected = (q["ok"] == true).then(|| AnticipatedDeclaration {
                chunk_index: q["declarant"].as_i64().unwrap(),
                archetype: q["archetype"].as_u64().unwrap() as u32,
            });
            assert_eq!(table.after(id, chunk), expected, "lookup {case}: {q}");
            let mut world = FilmWorld {
                current_chunk: chunk,
                anticipated: Some(table.clone()),
                ..Default::default()
            };
            assert_eq!(world.bind_anticipated(id), expected, "world {case}: {q}");
            assert_eq!(
                world.archetype(id & 0x3fffffff),
                expected.map(|d| d.archetype)
            );
        }
    }
    assert_eq!(
        build_source_anticipated_bindings(None),
        AnticipatedBindings::default()
    );
}
