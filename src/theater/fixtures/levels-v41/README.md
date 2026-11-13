# Build-specific level and region fixtures

Current catalogue source: [OpenSpartan/libfilmshell](https://github.com/OpenSpartan/libfilmshell/tree/f050371a6cda83b9887eeb83a1884336fce28e53),
commit `f050371a6cda83b9887eeb83a1884336fce28e53`.

`catalogue.json` exports all 33 original levels / 65 structure BSP regions,
including exact `f32::to_bits()` bounds, tags, index widths, axis widths,
optional display names and all 33 Halo service level IDs. Region geometry is
identical to the earlier source snapshot.
`generate_catalogue.py` generates `film/catalog/maps/levels/data.rs` with `from_bits`
constants so decimal conversion cannot alter the calibration.

`points.json` contains 329 independently authored, MSB-first point bodies
against the older unchanged reader at `18067987ebc3628ad499c819c6aee814692df43b`:
195 indexed event points (65 regions at precision 16/15/12), 130 indexed
component positions (dynamic/object), three default event points and one default
object point. Each has a following `0xd3` marker. `reference.json` contains 324
traces from the unchanged original readers, including their following marker.
Instrumentation only observes primitive reads/skips; parameters come from the
original `levels::level` and `Params::set_level` implementations.

The five omitted original-reader comparisons are `mp_02` region 4. Its catalogue
has five regions and a three-bit index, but original event/component parameters
hold only four region slots and fall back to the primary widths for index 4.
Local tests use the exported region-4 widths and authored source fields directly.
They do not claim upstream reader agreement for this unsupported slot.

Tests check raw codes, field extents, every-bit truncation, following components
in actual DELTA records, following detonation/zoom events, default-box context
independence, runtime region selection, and unknown-region refusal before any
axis. A verified table takes precedence over historical opaque aggregate widths;
missing regions cannot be repaired with a stub. Missing layout is Unsupported,
whereas missing bytes remain Truncated.

## Build identity

Catalogue label: `269225.26.04.08.1618 HI_1_13_0`.
The film's `BuildVersion` (`6.10026.19225.0` in the retained corpus) is a product
version in a different namespace. Binding uses the recorded little-endian
`BuildId` **269225**, `BuildBranch` **HI_1_13_0**, and the sequential game-options
`LevelTag`. The identification of `BuildId` at `0xcb454` follows pinned LevelUp
`film_identity.go` (`DAT_144e4ef50`), not a string search or neighboring-version
fallback. The catalogue's date/time suffix is retained as source metadata; it is
not a separately recorded film field. All 32 retained films match this engine
build identity and a catalogue tag. Altered build IDs/branches/tags and incomplete
header/identification/options sections refuse binding.

Runtime uses additional catalogue regions only when its caller's primary bounds
and region-index width agree exactly. Default-box source values do not become
coordinates in the primary region. Bounds are in engine units; rendering/world
unit conversions remain outside canonical Film data.

## Reproduce

```sh
python3 src/theater/fixtures/levels-v41/export_reference.py /path/to/libfilmshell > /tmp/catalogue.json
cmp /tmp/catalogue.json src/theater/fixtures/levels-v41/catalogue.json
python3 src/theater/fixtures/levels-v41/generate_catalogue.py
python3 src/theater/fixtures/levels-v41/generate_points.py
python3 src/theater/fixtures/levels-v41/export_points.py /path/to/libfilmshell > /tmp/points-reference.json
cmp /tmp/points-reference.json src/theater/fixtures/levels-v41/reference.json
```

The exporters use disposable archives of their recorded commits and offline Cargo.
No captured-film oracle is generated or rewritten by this workflow.
