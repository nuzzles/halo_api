# f050371 mode-classifier oracle

`cases.py` authors every presence combination of the eleven filename triggers
in `model/variant.rs`: 2,048 bounded Bond variant script blobs. The cases include
the new Attrition/Elimination, Extraction and Escalation triggers, competing
older triggers and the Firefight/KOTH pair.

`reference.json` comes from the original f050371 `scripts.rs` search and
`GameMode::from_scripts` classifier compiled unchanged by
`../script-evidence-v41/export_reference.py`. That exporter checks exact Git
blobs before compiling; the only adapter supplies the reader's three-bit shift
and two trailing bytes to expose the authored blob endpoint. The local test
parses every Bond fixture through Film and compares its source-name set and
precedence choice, while checking that all alternative matches retain source
occurrences. It does not establish actual script loading or a recorded engine
mode.

```sh
python3 src/theater/fixtures/script-modes-f050371/cases.py
python3 src/theater/fixtures/script-evidence-v41/export_reference.py \
  /path/to/libfilmshell-f050371 \
  src/theater/fixtures/script-modes-f050371/cases.json \
  > /tmp/script-modes-f050371-reference.json
cmp src/theater/fixtures/script-modes-f050371/reference.json \
  /tmp/script-modes-f050371-reference.json
cargo test --offline --lib f050371_modes_match_original_classifier_for_every_script_combination
```
