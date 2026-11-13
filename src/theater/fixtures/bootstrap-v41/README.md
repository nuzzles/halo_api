# Fixed bootstrap header fixtures

The supported prefix is selected only after a v41/27 registry with 50 blocks,
the verified component fingerprint and endpoint `0xcb208`. Starting at that
endpoint it reads 123 little-endian event-version words, three 32-byte build
buffers, two little-endian words and the corruption-check bit. It consumes 4,769
bits. Strings retain every byte, including tails and non-ASCII bytes. Unknown
registry layouts have no canonical bootstrap header; a truncated supported
header retains complete fields and the first unavailable range.

Sources: pinned [libfilmshell header.rs](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/header.rs)
and [LevelUp vue_a_versions.go](https://github.com/JGtm/LevelUp/blob/413971bcc485be53a2fa8cf229e170539a5146f8/apps/go-api/internal/games/halo_infinite/film/internal/grammar/vue_a_versions.go).
The latter identifies `FUN_1428e1c64` reading `film+0xcb208+4*genre` under the
recorded film version, and the 123 native event kinds. No aligned build-string
search is used to select the canonical prefix.

`wire_cases.py` authors the fields independently, with both flag values, mixed
little-endian words, non-ASCII buffers and nonzero string tails. These are field
goldens, not an upstream exported read trace. Three tests cover raw fields,
serde, every-bit truncation without usable padding, unsupported-layout refusal,
and `Film::parse` integration with fire, zoom, entity-end and control-end records.
Only a complete supported header with fire version 3 enables the v3 reader;
other/missing versions refuse it. Default-box points need no map context;
indexed-box points retain their gate and stop before unknown index widths.

`registry-prefix.bin.zlib` contains only the original version words and component
registry from `controller/01-stick-circles/chunk-000-type-1.bin`. It excludes all
identity/player/variant data. The full inflated source SHA-256 is
`deaf9ff3a0cb31d66875c52fcb6b371ef27c6995456b535338d89a292668ac8b`;
the retained 832,008-byte prefix SHA-256 is
`669feb53b5104894b40252cc547dcf78d2492928f8fc733b48ff416cfb7aff82`.
The corpus audit in `docs/research/libfilmshell/bootstrap_inventory.py` verifies
all 32 current films share that prefix, declare 123 event versions at the
independently observed build boundary and record fire version 3.

The following identification is now retained in its own node with 11 fields:
two 256-byte buffers, timestamp, three words, three 4,096-byte blocks and two
16-byte map-variant IDs. Source widths are recorded against the game writers
`FUN_14299b198`/`FUN_14299b278` in pinned LevelUp
[film_identity.go](https://github.com/JGtm/LevelUp/blob/413971bcc485be53a2fa8cf229e170539a5146f8/apps/go-api/internal/games/halo_infinite/film/internal/grammar/film_identity.go).
All 12,848 bytes follow at the prior flag's one-bit shift. The complete section
ends at `0xce68c * 8 + 1`; its buffers/blocks have no fabricated internal schema.

`identification_cases.py` independently authors four shifted cases, including
zero timestamp/GUIDs and nonzero arbitrary unknown fields. `export_identification.py`
archives pinned libfilmshell and instruments only `header::bytes`, invoking the
original `header::read`. Its three selective read locations (timestamp and two
GUIDs) must match the authored fields. Upstream's suppression of all-zero values
is recorded separately; canonical raw fields retain them. The other fixed field
extents come from the writer map, not an upstream primitive trace.

Two identification tests cover all raw fields, source ranges, serde, following
body bits, every scalar/GUID truncation bit, and both ends/midpoints of larger
buffers. Two runtime tests compare the canonical projection with the original
build-search projection, record boundary provenance, and prove that an embedded
`HI_` string or truncated canonical section cannot select a fallback boundary.
This establishes the start of the game-options body, not the map-variant/Bond
or player-table grammar or all event-version admission rules.

```sh
python3 src/theater/fixtures/bootstrap-v41/wire_cases.py
python3 src/theater/docs/research/libfilmshell/bootstrap_inventory.py experiments/films
cargo test --offline --all-features --lib bootstrap
python3 src/theater/fixtures/bootstrap-v41/identification_cases.py
python3 src/theater/fixtures/bootstrap-v41/export_identification.py /path/to/libfilmshell > /tmp/identification-reference.json
cargo test --offline --all-features --lib canonical_identity
```
