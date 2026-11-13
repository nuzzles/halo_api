# Zone event meaning oracle

`export_reference.py` verifies `src/zones.rs` blob
`4701db66d587260404006301be25b4fb2ae1931f` at libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03`, compiles its exact contiguous
constants/Mode/About/table/ZONED section, and queries every ID in 0..1024 for all
four modes. The 23 positive rows and queried range are retained in reference.json.
The Rust tests compare all 4,096 results, including unknown IDs.

The separately named Strongholds 0x322 broadcast is handled outside the upstream
table by `SH_CAPTURED`/`read()`. The local view retains that original message as
StrongholdCaptured, instead of using it to fill another capture message's target.
This one explicit extension is tested separately from table equality.

Reproduce with:

```sh
python3 src/theater/fixtures/zones-v41/export_reference.py /path/to/pinned/libfilmshell > /tmp/zone-reference.json
cmp src/theater/fixtures/zones-v41/reference.json /tmp/zone-reference.json
```

Authored native packet tests live in runtime/zones/tests.rs and use public Film
parsing with the existing recognized no-check bootstrap fixture. They exercise
message boundaries, same-time duplicates, raw field provenance, recipient bits,
mode collisions, repeated/wrong/absent keys, positions and truncation. Existing
independent events-v41 oracles establish the underlying code-82 body grammar;
the new table oracle is semantic evidence, not a new wire-format oracle.
