# Native observer shallow-copy ownership

Pinned reference: LevelUp feat/v75 `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Native harness: `../reference/halo_rust_observer_copy_test.go.txt`.

16 cases compare four snapshots of both original and copied observers (128
observer states), histogram reset output and callback replacement. Cases cover
nil, allocated-empty and populated absolute-index maps, nil/populated anticipated
binding maps, repair histograms, independent scalar repair/resync counters,
shared allocated maps, newly allocated local maps, and map replacement on reset.
Go generation succeeded before storing the compressed fixture. The generator
registers the harness and output mapping.

`Clone` aliases the entire Rust observer as before. `shallow_copy` creates an
independent observer containing copied callback identities and scalar counters,
while sharing allocated native maps. Histograms returned by `counters` are owned
snapshots. Taking absolute indices replaces this observer's map with a fresh,
allocated-empty map; an existing shallow copy retains the old map.

This establishes the observer ownership primitive needed by raw resync. It does
not yet connect the raw-resync scan to live context or validate its acceptance
callback and temporary position collector. It is a native state-machine fixture,
not an independently annotated film or semantic-action test.
