use super::*;
use serde_json::{Value, json};
use std::{
    fs::{self, File, FileTimes},
    io::Read,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};
fn snapshot(index: &MapBackgroundIndex) -> Value {
    let queries = [
        "Alpha", "Beta", "Omega", "Linked", "Changed", "a", "b", "link", "map", "unknown", "Bytes",
        "\u{fffd}",
    ];
    let lookups: serde_json::Map<String, Value> = queries
        .into_iter()
        .map(|name| (name.to_owned(), json!(index.lookup_key(name))))
        .collect();
    let key_hex: serde_json::Map<String, Value> = queries
        .into_iter()
        .map(|name| {
            (
                name.to_owned(),
                json!(index.lookup_key(name).map(|k| {
                    k.as_bytes()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                })),
            )
        })
        .collect();
    json!({"key_hex":key_hex,"keys":index.keys(),"identities":index.identities(),"ambiguous":index.ambiguous(),"lookups":lookups})
}
#[test]
fn native_background_directory_sequence() {
    let mut raw = vec![];
    flate2::read::ZlibDecoder::new(
        include_bytes!("fixtures/background-directory-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 19);
    let root = std::env::temp_dir().join(format!(
        "halo-bg-dir-{}-{}",
        std::process::id(),
        UNIX_EPOCH.elapsed().unwrap().as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let mut previous = None;
    for (i, row) in rows.iter().enumerate() {
        let action = &row["action"];
        let name = if let Some(hex) = action["name_hex"].as_str() {
            use std::os::unix::ffi::OsStringExt;
            std::ffi::OsString::from_vec(
                (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect(),
            )
        } else {
            action["name"].as_str().unwrap_or("").into()
        };
        let path = root.join(name);
        match action["op"].as_str().unwrap() {
            "write" => {
                fs::write(&path, action["raw"].as_str().unwrap()).unwrap();
                let time = UNIX_EPOCH + Duration::from_nanos(action["time"].as_u64().unwrap());
                File::options()
                    .write(true)
                    .open(&path)
                    .unwrap()
                    .set_times(FileTimes::new().set_accessed(time).set_modified(time))
                    .unwrap();
            }
            "mkdir" => fs::create_dir(path).unwrap(),
            "remove" => fs::remove_file(path).unwrap(),
            "symlink" => {
                std::os::unix::fs::symlink(action["target"].as_str().unwrap(), path).unwrap()
            }
            "load" => {}
            op => panic!("unknown {op}"),
        }
        let cached = map_background_index_for(&root).unwrap();
        let reused = previous
            .as_ref()
            .is_some_and(|previous| Arc::ptr_eq(previous, &cached));
        assert_eq!(reused, row["reused"].as_bool().unwrap(), "reuse {i}");
        assert_eq!(snapshot(&cached), row["cached"], "cached {i}");
        assert_eq!(
            snapshot(&build_map_background_index(&root).unwrap()),
            row["fresh"],
            "fresh {i}"
        );
        previous = Some(cached);
    }
    assert!(build_map_background_index(root.join("nonexistent")).is_err());
    assert!(map_background_index_for(root.join("nonexistent")).is_err());
    assert!(build_map_background_index(root.join("bad.json")).is_err());
    assert!(map_background_index_for(root.join("bad.json")).is_err());
    let alias = map_background_index_for(root.join(".")).unwrap();
    assert!(!Arc::ptr_eq(previous.as_ref().unwrap(), &alias));
    assert!(Arc::ptr_eq(
        &alias,
        &map_background_index_for(root.join(".")).unwrap()
    ));
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let root = root.clone();
            std::thread::spawn(move || map_background_index_for(root).unwrap())
        })
        .collect();
    for worker in workers {
        assert!(Arc::ptr_eq(
            previous.as_ref().unwrap(),
            &worker.join().unwrap()
        ));
    }
    fs::remove_dir_all(root).unwrap();
}
