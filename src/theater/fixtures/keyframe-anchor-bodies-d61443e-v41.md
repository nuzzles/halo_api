# Updated native keyframe body oracle, fixed independent inputs

Body reader: LevelUp d61443ef59268ad734355db8e9974f68db5ca6d0.
Input selection: the original 43a01721e8a02c87c955e175936f0ccf8dd97a81
`keyframe-anchor-bodies-v41.jsonl.zlib` fixture, retained unchanged.

`../reference/halo_rust_keyframe_anchor_bodies_d61443e_test.go.txt` reads only
the original file/packet-offset/anchor selection, then invokes the newer
`WalkKeyframeFullState` at every selected offset. No Rust output supplies inputs
or expectations. This isolates body grammar changes from newer heuristic anchor
selection: no selected anchor is thereby proven a canonical sequential record.

The generator asserts 32 films, 451 keyframes and all 134,657 original anchors,
with no unused or missing packet selections. Go results: 114,357 complete reads,
483 padded reads, 43,631 dead-state objects and 189,416 captured payloads.
Reproduce by copying the generator as `_test.go` into the newer pin's grammar
package, running `go test ./internal/games/halo_infinite/film/internal/grammar
-run '^TestHaloRustKeyframeAnchorBodiesD61443e$' -count=1` from apps/go-api, then
zlib-compressing `/private/tmp/halo-keyframe-anchor-bodies-d61443e.jsonl`.

The original Go selection remains a separate historical policy. Newer Go
WalkKeyframeWorld selects 136,276 anchors; no selection migration is claimed here.
