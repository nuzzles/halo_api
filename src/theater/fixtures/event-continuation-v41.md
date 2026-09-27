# Native event-list continuation oracle

Pinned LevelUp commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_event_continuation_test.go.txt`, installed as
an additional test in `internal/facts/killsource` in the pinned checkout.

The 2,560 cases compose the actual reference `evStep` with
`grammar.DecodeFrameViewsCurseur`. No Rust output supplies expected values.
The inputs are the encoded frames and `extra`/`check` flags from the existing
`frames-levelup-v41.json.zlib` reference fixture. The generator prepends manually
encoded event lists and removes the original empty-message preamble.

Coverage: empty lists, 65-event stub lists, zoom layouts, fixed opaque bodies,
both code-15 runtime settings (including a 70-bit payload), kill layouts,
unsupported code-36 bodies and code-14 presence providers. Each combination has
five byte cuts, including empty, head truncation, body truncation and full input.
For nonempty lists that reach their terminator, the fixture contains following
view endpoints, view counts, ordered entity records/component attempts, and a
second traversal using the same native grammar world.

640 cases reach a nonempty list terminator; the remaining 1,920 must not use the
continuation path. Empty lists belong to the original generic frame path.
The oracle is a composition of supported native readers, not evidence that the
reference already connects these readers in its production entry or supports
every event body. Reaching a list terminator does not establish that the following
views decode completely. Synthetic tail reads remain subject to the existing
frame reader's padding diagnostics.

Reproduction from the repository root:

```sh
python3 - <<'PY'
import json, zlib
from pathlib import Path
rows = json.loads(zlib.decompress(Path('src/theater/fixtures/frames-levelup-v41.json.zlib').read_bytes()))
Path('/private/tmp/halo-native-continuation-input.json').write_text(json.dumps([
    {key: row[key] for key in ('hex', 'extra', 'check')} for row in rows
]))
PY
```

Copy the generator to `internal/facts/killsource/halo_rust_event_continuation_test.go`
inside the reference film root. From its `apps/go-api` directory:

```sh
GOCACHE=/private/tmp/halo-go-cache GOPATH=/private/tmp/halo-go-path \
  /private/tmp/halo-go-runtime/go/bin/go test \
  ./internal/games/halo_infinite/film/internal/facts/killsource \
  -run '^TestHaloRustEventContinuation$' -count=1 -v
```

Compress `/private/tmp/halo-native-continuation.json` with zlib level 9 to create
the adjacent fixture. Validate with:

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib native_data_event_continuation
```
