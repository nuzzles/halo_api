# Fixed position domains and reference midpoints

This fixture separates fixed-domain constants from map-region bounds. It does not
claim that a runtime's unknown build matches the reference or that a parent-local
position is already in world space.

Sources:

- LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`: original
  `profile/loi_largeurs.go` and `profile/plages_quant.go`, plus the unchanged
  `bornePrecHaut` constant from `lecteur_position.go`. The first file cites
  HI_1_13_0's `DAT_143b8c6b8` / `FUN_140be9a14` for the default-world bounds;
  the position reader cites `DAT_143b8c6d0` / `FUN_141f85880` for local bounds.
- libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`: the unchanged
  `game/levels.rs` `LevelBox` and `LevelBox::engine` implementation.

The exporter verifies all four Git blobs, compiles the original Go bounds/width
code, then supplies those returned constants to the original Rust midpoint
function. The local-bound envelope is an adapter reproducing the three identical
axes passed by `lireF85880`; no original reader body or arithmetic is modified.
The reference JSON retains the exact bound bits, widths and f64 result bits.
It contains 75 vectors per domain: edge/center cases plus 64 seeded authored
vectors, for 150 results total. No captured positions generate expectations.

This proves reference midpoint arithmetic for the externally recovered constants.
It does not execute native film parsing or establish the engine's f32 rounding.
The native integration suite independently serializes every vector into NEW/DELTA
object-position components, with unchecked and both checked-trailer forms, and
compares the source-linked fixed-domain result. Other tests verify wrong-domain,
width, range and malformed-word refusals. Local results stay local. Indexed/raw
positions are refused by the fixed-domain method.

The API calls these values midpoints: it deliberately retains the existing f64
bin-center convention used by `PositionBounds::dequantize`. LevelUp's separate
`dequantWorldAxis` does f32 operations in a different order; these results must not
be advertised as bit-exact output of that function or of the engine. Neither
reference proves a world's unquantized original coordinate.

Reproduce (Go 1.23+ and Rust required):

```sh
python3 src/theater/fixtures/fixed-positions-v41/export_reference.py /path/to/pinned/grammar /path/to/pinned/libfilmshell /path/to/go > /tmp/fixed-reference.json
cmp /tmp/fixed-reference.json src/theater/fixtures/fixed-positions-v41/reference.json
```

The grammar directory must include the pinned `profile` files named above. The
exporter works offline once those sources are present.
