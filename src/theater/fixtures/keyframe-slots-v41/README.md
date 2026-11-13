# Native keyframe table fixtures

Pinned LevelUp: `413971bcc485be53a2fa8cf229e170539a5146f8`.
`export_source.py` checks the original Git blobs for `keyframe_entrees.go`,
`keyframe_fullstate_loop.go` and `source/bits.go`. It extracts unchanged table
classification/traversal and full-state/default-guard functions; only package
qualifiers change. The original bit reader is copied with a package rename.

The Go harness supplies context and a single empty registry row. Its default
and component callbacks panic if reached: this fixture tests native table
traversal and nonpositive signed body guards, not component bodies. Component
and creation readers have separate fixture suites. Recipe inputs author wire
bits independently in Go and Rust; expected traversal results come from Go.

The 24 cases cover 8,191-slot closure, all-vacant and mixed tables, live entries
after vacant entries, generation zero, nonmonotonic IDs, a raw all-ones live ID,
arbitrary vacant header words, invalid archetypes, incomplete guards, all byte
cuts across the last required header, nonzero alignment bits and extra data.
The native reader tests only the archetype when deciding whether a body exists.
Its traversal does not treat an all-ones ID as a table terminator.

On failure Go returns the failing slot start; Rust retains consumed prefix fields
and reports their end. Successful boundaries agree exactly. The fixture does not
prove runtime vacant-slot clearing or ownership semantics.

Reproduce:

```sh
GO_BIN=/path/to/go GOCACHE=/tmp/halo-go-cache python3 export_source.py /path/to/pinned/grammar > /tmp/reference.json
cmp reference.json /tmp/reference.json
```
