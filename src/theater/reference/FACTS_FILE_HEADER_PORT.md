# FilmFacts file header and freshness

Reference: header/coverage/Utilisable functions in `replay/filmfacts_fichier.go`,
pinned at `43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`facts_file_header.rs` carries every coverage field: four revision strings, Build,
optional registry Fingerprint/Status, Blocks and NamedSlots. Strings remain raw
bytes; native counts remain signed i64 on WASM. Nil registry and present zero
registry remain distinct, including partially decoded registry payloads.

The file prefix is `LEVELUPFILMFACTS\n`, codec 1, schema 3 and unsigned header
length. The header writer uses the current versions and actual length, as native
EncodeFilmFactsFile does; the fields on a read header are not writer overrides.
Coverage is followed by raw map module, three u64 axis widths and detection flag.

Header decoding preserves the native partial result on every returned error.
Its signed, wrapping body offset is retained explicitly. The native check is
`off + length > len(blob)` using native signed arithmetic; the reader then reads
against the FULL blob, not a slice limited to that length. Short lengths can
therefore succeed and point into already-read header bytes. Negative/wrapping
lengths can return successful headers with unusable section offsets. The future
full-file reader must refuse native out-of-bounds panics safely at that point,
without changing this observed header API behavior.

`usable` implements the independent freshness decision: reject codec/schema,
then compare the four current revision strings, then call the existing cooking
key check. Build and registry do not participate. Current revisions come from the
existing pinned coverage builder, avoiding a second constants table. Version,
revision, map and layout error distinctions are preserved. Revision diagnostics
use native `%s`, which can insert invalid UTF-8: message_bytes() retains those
bytes exactly, while Display is explicitly a lossy human-readable view.

Full-file decoding must not call usable automatically. Native full decoding checks
header versions and the cooking key but leaves revision freshness to the caller.

## Validation

- 6,201 coverage cases: 64 populated/empty source states, every prefix and all
  256 presence-byte values. Fields, writer bytes, errors and consumed offsets match.
- 8,492 header cases: every prefix, full-file trailing data, wrong codec values,
  zero/short/large/signed-wrapping lengths and partial headers. The body offset and
  every decoded field are compared, then native/Rust usability is compared even
  for partial headers.
- 448 explicit freshness decisions check each revision independently, changed
  Build/registry, wrong map and layout, including exact raw error bytes.
- Platform/static results: `facts-file-header-validation.json`.

This is part of the file codec, not completion of filmfacts_fichier.go. JSON
sections, section framing, full-file encode/decode and integration remain open.
Full v41 parity remains incomplete and the architecture phase stays deferred.
