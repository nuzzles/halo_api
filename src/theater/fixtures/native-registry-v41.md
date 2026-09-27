# Native registry read trace

Pinned LevelUp commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_native_registry_test.go.txt`.

The fixture contains 122 cases: the 116 existing registry-result inputs and six
controlled cases for full blocks, nonzero string tails, invalid first names,
empty names with nonzero tails, a nonzero level under an empty name, and a final
nonzero padding byte. The generator calls pinned `entryName`, `entryLevel`,
`registryBlockTail` and `ParseRegistryChunk`; it does not use Rust outputs.
It retains 61 complete block attempts and 218 name attempts.

The Rust test compares ordered attempts, full 256-byte name fields, optional
level bytes, source boundaries, first nonzero tail positions and acceptance.
Legacy projections still match the existing native registry fixtures. Partial
trailing blocks are not read, following the reference's whole-block requirement.
The final rejected block is a boundary probe into the next section, not an
accepted archetype. String tails and unexamined source bytes survive in the
retained bootstrap. Zero-tail spans are validation reads, not component fields.

Old exports have `block_reads: None`; an empty trace from a new read is
`Some([])`. JSON tests verify both distinctions. The native-entry corpus checks
accepted-block counts, rejected-boundary positions, and source bytes for every
retained name attempt across all 32 films.

To reproduce the fixture, decompress `registry-result-v41.json.zlib` into
`/private/tmp/halo-native-registry-input.json`, copy the generator as an additional
`_test.go` file into the pinned reference's `internal/grammar` package, then run
`TestHaloRustNativeRegistry`. Compress its `/private/tmp/halo-native-registry.json`
output with zlib into this fixture. Never edit pinned production source.

Run the Rust checks:

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib registry:: -- --nocapture
CARGO_INCREMENTAL=0 cargo test --release --lib native_data_ -- --include-ignored --nocapture
```
