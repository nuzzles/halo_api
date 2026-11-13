# Current libfilmshell mode-icon oracle

`reference.json` runs the unchanged `src/variant.rs::read` implementation
from [libfilmshell a13efb323b230c96b59685defbc376720043240f](https://github.com/OpenSpartan/libfilmshell/tree/a13efb323b230c96b59685defbc376720043240f)
on all 32 retained v41 registry chunks. The exporter verifies the original Git
blob `51c7626e939444345753f366b3eb1177980d9034`, compiles that source in
a disposable Rust harness with only the required `OptionValue` type, and reads
the exact retained headers. It neither patches the reader nor invokes our parser.
Each row includes manifest and original registry SHA-256 for source identity.

```sh
python3 src/theater/fixtures/variant-icon-a13efb/export_reference.py \
  /path/to/libfilmshell-at-a13efb experiments/films \
  > /tmp/variant-icon-reference.json
cmp /tmp/variant-icon-reference.json \
  src/theater/fixtures/variant-icon-a13efb/reference.json
cargo test --offline --all-features --lib \
  captured_mode_icons_match_latest_original_reader -- --ignored
```

The original reader returns icon 0 in 30 films, 8 in Bandit Evo, and 2 in
ranked Oddball. Our runtime view matches all 32. The recorded icon is the
selected variant header's Bond field `[0, 10]`; an omitted field uses the
current reference writer's default zero and remains distinguishable from a
recorded zero. Upstream finds the header by scanning backward from a string
path. Our canonical Bond parser already owns the enclosing header and field
range, so the runtime view uses that structure. Neither the icon index nor
this corpus establishes universal game-build applicability.
