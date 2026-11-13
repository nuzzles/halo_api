# Player metadata source fixtures

Pinned LevelUp: `413971bcc485be53a2fa8cf229e170539a5146f8`.
`export_source.py` verifies Git blob hashes before extracting original shared
player readers, type-8/type-12 wrappers, constants and build/team profiles. The
original `source/bits.go` is copied with only its package name changed. Extracted
reader bodies are unchanged; type qualifications are adapted to harness structs.
The harness structs carry the fields used by those readers, without interpretation.

`reference.json` contains 30 authored packet cases with independent Go outputs.
Seven build profiles, pre/post-format-21 gates, unaligned successive entries,
full 16-unit names, unpaired surrogates, consumed NULs, distinct identity/join keys,
nonzero blobs/padding, maximum masks/lists, count limits, truncation, empty lists
and excessive tails are covered. Payloads are hex. Display strings in the oracle
are lossy, as upstream is; canonical Rust fields retain original units.

Reproduce using the pinned grammar checkout (including `source/bits.go`,
`profile/build_profile.go`, and `profile/equipe_bot.go`):

```sh
GO_BIN=/path/to/go GOCACHE=/tmp/halo-go-cache python3 export_source.py /path/to/pinned/grammar > /tmp/reference.json
cmp reference.json /tmp/reference.json
```

For independently checking retained captures, run the ignored test with
`HALO_ROSTER_EXPORT=/tmp/captured.json`, then:

```sh
GO_BIN=/path/to/go GOCACHE=/tmp/halo-go-cache python3 export_source.py /path/to/pinned/grammar /tmp/captured.json > /tmp/go-captured.json
python3 compare_captured.py /tmp/captured.json /tmp/go-captured.json
```

The source readers discard failed entries or packets; Rust retains partial fields
and explicit stops. Thus failed-prefix retention is checked against source spans,
not equated to Go's discarded output. For complete reads, identities, names,
short values, entry boundaries, populations and packet closure are compared.
Type-12 team bytes are cross-checked at each established build offset without
publishing those interpretations in canonical Film. These fixtures do not prove
ownership, build-independent structure, or the internals of opaque blobs.
