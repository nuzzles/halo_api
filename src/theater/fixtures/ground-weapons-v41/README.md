# Ground-weapon reference sequences

`export_reference.py` runs the unchanged complete `src/inventory.rs` at
OpenSpartan/libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`, Git blob
`c5cc80c8f3cf7093da53b73cf00ca9a4937ffc79`.

Minimal payload/event adapters supply authored operations and pickup bodies. The
catalogue adapter returns known metadata for every input tag, isolating the
inventory algorithm from catalogue coverage. These are invocation fixtures, not
native packet decoding or independent proof of pickup identity.

Seven sequences cover pending/unpositioned objects, multiple positions, open and
deleted intervals, same-packet and other-packet pickups, duplicate same-tag objects,
and reuse after deletion. JSON retains the original final-coordinate/from/to and
`taken` results. Native local fixtures serialize these inputs as sequential
NEW/DELTA/DELETE and code-9 events with independently traced creation bodies.
Original frame indices map to explicit fixture packet timestamps.

Reproduce with a pinned checkout and rustc:

```sh
python3 src/theater/fixtures/ground-weapons-v41/export_reference.py /path/to/libfilmshell > /tmp/ground-reference.json
cmp src/theater/fixtures/ground-weapons-v41/reference.json /tmp/ground-reference.json
```

Local tests compare the original tag, last position and lifecycle endpoints;
history retains all earlier coordinates independently. Each upstream `taken`
has matching candidate evidence. The duplicate-tag case explicitly preserves the
disagreement: upstream consumes one pickup and marks only one deleted object,
whereas local analysis exposes that pickup as a candidate for both objects and
makes no `taken` claim. Unknown tags, initial NEW positions, incomplete reads and
gaps have separate local policy tests; their behavior is not established by this
small upstream invocation suite. See ../../docs/GROUND_WEAPONS.md.
