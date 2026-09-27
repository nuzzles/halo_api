# Identity and fallback JSON sections

Pinned file sections 2 and 3 in `replay/filmfacts_fichier.go` use native
`profile.FilmIdentity` and `[]fallback.Declenchement`.

`facts_identity_json.rs` retains all fourteen identity fields and both fallback
fields. Native strings are raw byte vectors until Go-compatible JSON encoding
normalizes invalid UTF-8; JSON decode produces the native normalized Unicode.
Signed native ints remain i64 on WASM. Named fallback strings preserve the native
`fallback.Nom` type in mismatch diagnostics. All fields are written in native
declaration order. Nil identity pointers and nil/empty fallback/type-version
slices remain distinct.

The shared JSON parser now retains decoded string-node values; it still validates
the entire syntax before schema errors. `Shape::String` supports named string
types. `Shape::Pointer` and `JsonPointer<T>` are distinct from the established
Option<Vec/Map> nil-collection projection, so old object-death schemas are unchanged.

`FactsJsonState<T>` retains backing cells across successful section updates.
`FactsIdentityJsonReader` and `FactsFallbacksJsonReader` expose this behavior for
repeated sections. Pointer null resets the identity; an object updates an existing
identity. Shorter arrays/slices retain backing cells, and later null elements or
partial objects reuse native prior values. Empty arrays and null slices clear
that backing. Errors terminate the file decode: these readers are deliberately
invalidated after any error and must be discarded. They do not promise to expose
Go's partially mutated destination after a failed Unmarshal.

## Validation

- 512 native writer cases, with all single-byte values in source strings,
  arbitrary signed metadata, nil/present pointers and nil/empty/populated slices.
- 1,457 native decode sequences, including every prefix of targeted charges,
  all single-byte string inputs, invalid numeric types/ranges, surrogates,
  duplicate/folded fields and explicit multi-section backing-state sequences.
- All previous shared facts JSON tests run as regressions for the parser changes.
  Platform results are in `facts-identity-json-validation.json`.

`facts-file-json-field-inventory.json` is an independent reflection inventory of
all four file JSON payloads (64 reachable types), including declaration order,
JSON tags, named primitive types and unexported fields. Generate it with
`halo_rust_facts_file_json_inventory_test.go.txt` / TestHaloRustFactsFileJSONInventory
in the native replay package. There are no custom JSON/text marshalers, no JSON
tags and only float32 fields. The only unexported field is killsource.Kill.paquet,
which native JSON intentionally omits. Statborg and killsource remain pending.

This does not complete the file codec or full v41 parser goal. Framing, remaining
JSON payloads, conversions/BuildFromFacts, integration and final reconciliation
remain open. The architectural phase remains deferred.
