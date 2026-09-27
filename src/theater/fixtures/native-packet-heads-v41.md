# Captured native head-reader oracle

Pinned reference: LevelUp `feat/v75`, commit
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_native_packet_heads_test.go.txt`.

The generator calls the pinned source packet walker and then the direct native
zoom, pickup and translocator decoders. It retains every selected attempt,
including refusals, before slot-band filtering, player resolution or timestamp
sorting. It supplies no map context to the translocator decoder.

The captured corpus contains 12,997 selected attempts:

| Reader | Selected | Accepted by native reader |
| --- | ---: | ---: |
| Zoom | 11,263 | 11,263 |
| Pickup | 1,646 | 1,646 |
| Translocator | 88 | 0 |

The first-byte family predicate does not prove an accepted head. In particular,
the 88 translocator-family selections must remain refusals; they do not establish
88 recorded teleports. Successful translocator position reads are separately
covered by the synthetic reference fixture `translocator-levelup-v41.json.zlib`.

The integrated corpus test matches every row by source-relative filename and
packet ordinal, checks payload offset/size and wire timestamp, then compares
native acceptance and the supported values. It must consume every reference row.
The existing zoom/pickup/translocator padding fixtures add 7,680 admission and
padding cases; admitted cases also run through the native entry. Additional
checks cover external map configuration and legacy export defaults.

To regenerate, copy the generator to
`internal/games/halo_infinite/film/internal/grammar/halo_rust_native_packet_heads_test.go`
under the pinned checkout's `apps/go-api`. From that directory:

```sh
GOCACHE=/private/tmp/halo-go-cache GOPATH=/private/tmp/halo-go-path \
  /private/tmp/halo-go-runtime/go/bin/go test \
  ./internal/games/halo_infinite/film/internal/grammar \
  -run '^TestHaloRustNativePacketHeads$' -count=1 -v
```

Compress `/private/tmp/halo-native-packet-heads.json` with zlib level 9 to produce
the adjacent fixture. The generator reads the retained film files under
`/Users/simbleau/git/halo_api/experiments/films` and never reads Rust output.

Validate from the Rust repository root:

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib native_data_ -- --include-ignored --nocapture
```

These are dedicated head reads, not complete event/list partitions. Their ends
must not replace the generic event-layout walker's terminator when continuing
into entity/control views. Map bounds supplied through options are external
quantization context, not recorded fields or inferred map geometry.
