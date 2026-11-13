use super::*;
use serde_json::json;

#[test]
fn option_catalogue_matches_original_names_aliases_order_and_default_bits() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/game-catalogues-v41/reference.json"
    ))
    .unwrap();
    let catalog = Catalog;
    let names: Vec<_> = catalog
        .option_names()
        .map(|(hash, name)| {
            assert_eq!(catalog.option_name(hash), Some(name));
            assert_eq!(OptionHash::from_name(name), hash);
            assert_eq!(OptionHash::from_name(&name.to_ascii_uppercase()), hash);
            json!({"hash": hash.raw(), "name": name})
        })
        .collect();
    assert_eq!(names.len(), 98);
    assert_eq!(json!(names), reference["option_names"]);
    let modes: Vec<_> = catalog
        .mode_options()
        .iter()
        .map(|mode| {
            let options: Vec<_> = mode
                .options
                .iter()
                .map(|option| {
                    let default = option.default.map(|v| match v {
                        OptionValue::Number(number) => {
                            json!({"kind":"number", "bits":format!("{:016x}", number.to_bits())})
                        }
                        OptionValue::Bool(value) => json!({"kind":"bool", "value":value}),
                    });
                    json!({"name":option.name,"keys":option.keys,"default":default})
                })
                .collect();
            json!({"prefix":mode.prefix,"script":mode.script,"options":options})
        })
        .collect();
    assert_eq!(modes.len(), 12);
    assert_eq!(
        catalog
            .mode_options()
            .iter()
            .map(|m| m.options.len())
            .sum::<usize>(),
        247
    );
    assert_eq!(json!(modes), reference["modes"]);

    let rows: Vec<_> = catalog.option_names().collect();
    assert!(rows.windows(2).all(|w| w[0].0 < w[1].0));
    for raw in [0, u32::MAX, 0x42c9679f] {
        let hash = OptionHash(raw);
        assert_eq!(catalog.option_name(hash), None);
        assert_eq!(
            serde_json::from_value::<OptionHash>(json!(raw)).unwrap(),
            hash
        );
    }
    // Script aliases are not silently inserted into the original 98-entry table.
    let bastion = &catalog
        .mode_options()
        .iter()
        .find(|m| m.prefix == "BastionOptions")
        .unwrap()
        .options[0];
    assert_eq!(bastion.keys.len(), 2);
    assert_eq!(
        catalog.option_name(OptionHash::from_name(bastion.keys[0])),
        None
    );
    assert_eq!(
        catalog.option_name(OptionHash::from_name(bastion.keys[1])),
        Some(bastion.name)
    );
    let ctf = catalog
        .mode_options()
        .iter()
        .find(|m| m.script == "ctfoptions.lua")
        .unwrap();
    assert_eq!(
        ctf.options
            .iter()
            .find(|o| o.name == "CTFOptions_numberOfFlagsPerTeam")
            .unwrap()
            .default,
        None
    );
    assert!(
        ctf.options
            .iter()
            .any(|o| o.default == Some(OptionValue::Bool(false)))
    );
    assert!(
        ctf.options
            .iter()
            .any(|o| o.default == Some(OptionValue::Number(0.0)))
    );
}

#[test]
fn option_hash_matches_original_binary_vectors_without_implicit_normalization() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/game-catalogues-v41/reference.json"
    ))
    .unwrap();
    let cases = reference["murmur3"].as_array().unwrap();
    assert_eq!(cases.len(), 261);
    for (length, case) in cases.iter().enumerate() {
        let hex = case["hex"].as_str().unwrap();
        let bytes: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        assert_eq!(bytes.len(), length);
        assert_eq!(
            u64::from(murmur3(&bytes)),
            case["hash"].as_u64().unwrap(),
            "length {length}"
        );
    }
    assert_eq!(murmur3(b"default"), 0x42c9679f);
    assert_ne!(murmur3(b"DEFAULT"), murmur3(b"default"));
    assert_eq!(
        OptionHash::from_name("DEFAULT"),
        OptionHash(murmur3(b"default"))
    );
}

#[test]
fn f050371_option_catalogue_matches_unchanged_original_apis() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/options-f050371/reference.json"
    ))
    .unwrap();
    let catalog = Catalog;
    let names: Vec<_> = catalog
        .option_names()
        .map(|(hash, name)| {
            assert_eq!(catalog.option_name(hash), Some(name));
            assert_eq!(OptionHash::from_name(name), hash);
            json!({"hash": hash.raw(), "name": name})
        })
        .collect();
    assert_eq!(names.len(), 98);
    assert_eq!(json!(names), reference["option_names"]);
    let modes: Vec<_> = catalog
        .mode_options()
        .iter()
        .map(|mode| {
            let options: Vec<_> = mode
                .options
                .iter()
                .map(|option| {
                    let default = option.default.map(|v| match v {
                        OptionValue::Number(number) => {
                            json!({"kind":"number", "bits":format!("{:016x}", number.to_bits())})
                        }
                        OptionValue::Bool(value) => json!({"kind":"bool", "value":value}),
                    });
                    json!({"name":option.name,"keys":option.keys,"default":default})
                })
                .collect();
            json!({"prefix":mode.prefix,"script":mode.script,"options":options})
        })
        .collect();
    assert_eq!(modes.len(), 12);
    assert_eq!(
        modes
            .iter()
            .map(|m| m["options"].as_array().unwrap().len())
            .sum::<usize>(),
        247
    );
    assert_eq!(json!(modes), reference["modes"]);
}
