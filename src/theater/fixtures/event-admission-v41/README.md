# Recorded event admission

`reference.json` is exported from unchanged LevelUp functions at
`413971bcc485be53a2fa8cf229e170539a5146f8`. The exporter verifies six Git blobs,
extracts the native version/descriptor tables, version classification functions,
`placeDisponible`, `lireLaVueA`, `lireUnMessage`, `descripteurDuGenre` and
`chargeZoom`, and uses the original bit reader.

The 503 table cases cover lengths 0–125, three mismatches for each of 123 entries,
and recorded/missing major-version combinations. The 87 frame cases cover empty
lists, empty and zoom messages, unsupported codes, both configuration bits and
corruption policies, and partial headers. Prefix padding aligns each source end
without supplying artificial trailing bits. Present-reference callbacks panic
if reached; the message collection callback is empty. This oracle verifies
admission and boundaries, not semantic message collection or reference decoding.

Rust compares the native table and 371 applicable exact-length/major-41 cases,
every truncated recorded table, and all 87 frame cases. Four native shorter-table
cases decode under upstream's explicit numbering/closure hypothesis; canonical
Film deliberately refuses those cases. The fixed v41/27 bootstrap does not infer
variable table cardinality or detect an arbitrary extra word as a table extension.
Classification of other cardinalities/majors documents upstream behavior only.

Public Film tests preserve raw tables, retain the complete event header before a
stop, and refuse following views. Empty message lists remain readable without
version/configuration admission. A complete native table remains usable when
unrelated later header fields are missing; corruption policy still stops at its
own boundary. Replay tests verify runtime defaults cannot override missing or
mismatched versions or configuration zero. Their supplied empty roster is a test
prerequisite, not recorded identity evidence.

Reproduce from the repository root:

```sh
GO_BIN=/path/to/go GOCACHE=/tmp/halo-go-cache python3 src/theater/fixtures/event-admission-v41/export_source.py /path/to/pinned/grammar > /tmp/event-admission-reference.json
cmp src/theater/fixtures/event-admission-v41/reference.json /tmp/event-admission-reference.json
```

All 32 retained films use the native table and configuration one. Unsupported
table/configuration behavior therefore has authored source-fixture and public
integration evidence, not captured-film coverage. This does not establish global
reference ranges, alternate writer layouts or complete canonical film semantics.
