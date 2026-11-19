# Captured film goldens

`bazaar_idle` and `octagon_ar_kill` are small complete v41 film recordings from
local Bazaar and Octagon captures. Each `manifest.json` keeps only the
chunk kind, index, start time, and local filename. The `.bin` files contain
the captured inflated chunk bytes recompressed as zlib, matching the input
contract of `RawFilmChunk`.

These recordings exercise registry setup, multiple replication chunks, and
both empty and populated summary highlights. `Cargo.toml` excludes this
directory from the published crate. The integration tests compile only when
the directory exists in a development checkout.
