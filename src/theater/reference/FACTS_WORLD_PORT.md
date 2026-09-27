# Cached creation records, statistics and world-object composition

Reference: replay/filmfacts_codec.go encode/decodeCreations, CreationStats and
WorldObjectScan at 43a01721e8a02c87c955e175936f0ccf8dd97a81.

FactsEquipmentCreation retains every native field: identity, timestamp, signed
chunk/packet/bit positions, reference and ability fields with their presence
flags, all four MPP values and presence flags, exact position float bits, signed
mask entries, both mask flags, default-state width, ammunition flag and values,
and final bit position. False presence flags do not erase the associated values
that the native cache writes unconditionally. Empty decoded masks remain None.

FactsCreationStats preserves all twelve native counters as i64, including on
WASM. Existing live-scan counters and offsets use narrower/unsigned Rust types;
these cache DTOs avoid narrowing the native signed domain. Decoding uses the
native count(20) guard and retains partial records through the first error. Mask
counts use the native signed conversion; Rust avoids eager allocation while
retaining the same read sequence and values for tested truncated inputs.

FactsWorldObjectScan composes Scanned, creations, all statistics, keyframes and
tracks in native order. It does not gate field transport on Scanned. Existing
keyframe count guards see the actual subsequent section bytes, so errors and
partial output are compared for the whole composed stream rather than assuming
standalone codecs can always roundtrip.

The native fixture has 18,945 complete/truncated streams from 64 source scans.
It populates all creation fields, nonzero values behind false presence flags,
signed mask extremes, negative/wide offsets, signed counters, keyframe lists and
tracks. Every encoded byte, returned field, f32 bit pattern, final error and
cursor boundary is compared. Raw native f32 bits are extracted directly, without
passing through f64 reflection conversion. See facts-world-validation.json.

The biped/vehicle position section and full facts-cache containers, version gates,
remaining channel sections and assembly integration are still unported. This is
derived-cache compatibility, not native film re-encoding or full v41 parity.
The requested architecture remains deferred.
