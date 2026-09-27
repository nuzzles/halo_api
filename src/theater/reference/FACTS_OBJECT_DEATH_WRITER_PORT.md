# Object-death facts cache writer

Reference: `replay/filmfacts_mortsdobjet.go` at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

The writer carries all ObjectDeath/DeadState fields, all thirteen death-statistic
fields, and all six FrameConfig data fields with the full nested scan profile.
Only the observation callback is excluded by native design. Cache DTO defaults
are zero values, not production scan defaults. Counters and widths retain native
signed/unsigned 64-bit domains on WASM. Raw grammar map keys retain bytes, including
invalid UTF-8; integer map keys sort lexically as decimal strings. Nil and empty
lists/maps have different JSON encodings. Field order follows Go declarations.

Native writer failure is also observable: a nonfinite float causes a prefixed
JSON error, overwrites any prior writer error, and appends a zero-length charge.
Successful writes retain prior writer errors. A 2,304-case independent Go oracle
covers complete charges, populated profiles, map allocation states, malformed
string keys, prior errors, and every float-bearing field's boundary values.

## Native float formatting

The oracle runtime is Go 1.26.5. Its shortest float formatter uses Dragonbox.
Rust Display and serde_json are both insufficient to reproduce its bytes:

- 45691.3125f32: Display writes 45691.313; Go writes 45691.312.
- 0.000244140625f32: serde_json writes 0.00024414062; Go writes 0.00024414063.

Both alternatives round-trip to the same binary float, but byte-for-byte cache
writer parity requires the native decimal spelling. `facts_json_float.rs` ports
the complete finite float32 Dragonbox path and uses the required native power
range. `facts_json_write.rs` applies encoding/json's float32 exponent cutoffs and
string escaping. The port retains the Go license in GO_LICENSE.txt; runtime
source hashes and scope are in facts-json-float-runtime.json.

The independent float suite has 68,608 cases: 65,536 seeded random bit patterns,
plus signed boundary mantissas for all 256 exponents. It checks the final charge
through the public writer, including nonfinite errors and zero-length payloads.
This runtime-specific spelling is not a claim that native Film re-encoding or
compiler-independent derived NaN payloads are supported.

## Decoder and remaining integration

Object-death JSON decoding is now implemented and separately validated. See
FACTS_OBJECT_DEATH_DECODER_PORT.md and facts-object-deaths-decoder-validation.json
for native null/duplicate-field, slice backing, array, numeric-width and syntax
behavior. Both halves of filmfacts_mortsdobjet.go are now ported.

Conversions between cache DTOs and scan-result types remain part of integration.
Remaining cache channels, framing and BuildFromFacts still need implementation.
Full v41 parity remains incomplete, and the architecture phase stays deferred.
