# Latest libfilmshell motion-encoding labels

`reference.json` contains one label from the unchanged
[a13efb3 container reader](https://github.com/OpenSpartan/libfilmshell/blob/a13efb323b230c96b59685defbc376720043240f/src/container.rs)
for each of the 32 retained films, with its manifest SHA-256. The exporter
checks that source file's Git blob, invokes `detect_encoding_mode(film, None)`
through the upstream `research` feature, and does not call `halo_api`.

```sh
python3 src/theater/fixtures/encoding-a13efb/export_reference.py \
  /path/to/libfilmshell experiments/films > /tmp/encoding-reference.json
cmp /tmp/encoding-reference.json src/theater/fixtures/encoding-a13efb/reference.json
python3 src/theater/fixtures/encoding-a13efb/verify_source.py experiments/films
cargo test --offline --all-features --lib \
  captured_motion_encoding_labels_match_current_upstream -- --ignored --nocapture
```

These labels are motion-extraction hypotheses. The byte votes come from a
marker scan in the first three replay chunks; they are not recorded packet
discriminators or proof that a motion grammar decoded successfully.
