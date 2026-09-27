use super::*;
use serde_json::Value;
use std::io::Read;
fn fixture() -> Value {
    let mut raw = vec![];
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/background-index-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    serde_json::from_slice(&raw).unwrap()
}
#[test]
fn native_background_identity_normalization() {
    let oracle = fixture();
    let exceptions = oracle["normalizations"].as_object().unwrap();
    for c in (0..=0x10ffff).filter_map(char::from_u32) {
        let text = c.to_string();
        let expected = exceptions
            .get(&text)
            .map(|s| s.as_str().unwrap())
            .unwrap_or(&text);
        assert_eq!(
            normalize_map_identity(&text),
            expected,
            "U+{:04X}",
            c as u32
        );
    }
}
#[test]
fn native_background_index_lookup() {
    let oracle = fixture();
    let rows = oracle["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 257);
    for (i, row) in rows.iter().enumerate() {
        let backgrounds: Vec<_> = row["inputs"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| {
                (
                    key,
                    parse_map_background(&serde_json::to_vec(value).unwrap()).unwrap(),
                )
            })
            .collect();
        let actual = MapBackgroundIndex::from_backgrounds(
            backgrounds.iter().map(|(key, b)| (key.as_str(), b)),
        );
        assert_eq!(
            actual.keys(),
            row["keys"].as_u64().unwrap() as usize,
            "keys {i}"
        );
        assert_eq!(
            actual.identities(),
            row["identities"].as_u64().unwrap() as usize,
            "identities {i}"
        );
        assert_eq!(
            serde_json::to_value(actual.ambiguous()).unwrap(),
            row["ambiguous"],
            "ambiguities {i}"
        );
        for (name, key) in row["lookups"].as_object().unwrap() {
            assert_eq!(actual.lookup(name), key.as_str(), "lookup {i}: {name}");
        }
        let reversed = MapBackgroundIndex::from_backgrounds(
            backgrounds.iter().rev().map(|(key, b)| (key.as_str(), b)),
        );
        assert_eq!(actual, reversed, "order independence {i}");
    }
}

#[test]
fn native_background_index_raw_keys() {
    let oracle = fixture();
    let decode = |h: &str| {
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>()
    };
    let hex = |b: &[u8]| b.iter().map(|b| format!("{b:02x}")).collect::<String>();
    for row in oracle["raw_cases"].as_array().unwrap() {
        let keys: Vec<_> = row["keys"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| decode(v.as_str().unwrap()))
            .collect();
        let background = MapBackground {
            module: "Bytes".to_owned(),
            ..Default::default()
        };
        let index = MapBackgroundIndex::from_background_bytes(
            keys.iter().map(|key| (key.as_slice(), &background)),
        );
        assert_eq!(index.keys(), keys.len());
        assert_eq!(
            index.identities(),
            row["identities"].as_u64().unwrap() as usize
        );
        for (name, expected) in row["lookups"].as_object().unwrap() {
            assert_eq!(
                serde_json::json!(index.lookup_key(name).map(|k| hex(k.as_bytes()))),
                *expected
            );
        }
        let ambiguous: std::collections::BTreeMap<_, _> = index
            .ambiguous()
            .iter()
            .map(|(name, keys)| {
                (
                    name,
                    keys.iter().map(|k| hex(k.as_bytes())).collect::<Vec<_>>(),
                )
            })
            .collect();
        assert_eq!(
            serde_json::to_value(ambiguous).unwrap(),
            row["ambiguous_hex"]
        );
    }
}
