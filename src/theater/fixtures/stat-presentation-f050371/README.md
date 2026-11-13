# Stat presentation reference

`export_reference.py` checks three source blobs at libfilmshell
`f050371a6cda83b9887eeb83a1884336fce28e53` and executes the unchanged
`StatDisplay::from_game` and `StatDefinition::shown` implementations in a
disposable Rust harness. `reference.json` records seven display-code cases and
63 arithmetic cases, including negative and oversized precision values and
the i32 endpoints. Local tests compare exact f64 bit patterns and separately
check that missing or invalid Bond fields remain unresolved.

```sh
python3 src/theater/fixtures/stat-presentation-f050371/export_reference.py /path/to/libfilmshell > /tmp/stat-presentation-reference.json
cmp src/theater/fixtures/stat-presentation-f050371/reference.json /tmp/stat-presentation-reference.json
```

These are presentation conventions, not recorded values or proof that a
particular stat definition applies to an unresolved scoreboard slot.
