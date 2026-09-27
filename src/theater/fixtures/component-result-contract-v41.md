# Native component result metadata

Reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: reference/halo_rust_component_result_contract_test.go.txt,
registered in reference/generate_oracles.py.

Ten independent native cases exercise generic delta and full-keyframe reads
under five policies: ordinary EMP byte, zero-width calibrated replacement,
unsupported name, zero-width stub recovery for that unsupported name, and a
present weapon with distinct high and low identifier words.
Expected component metadata, reader endpoints and complete profiles come from
the Go parser. Final native generation passed (0.386s). The first eight rows
are unchanged by the two positive-weapon additions.

The native calibrated replacement returns variant zero even with no field bits;
ordinary EMP, unsupported and stubbed reads return u32::MAX. The unsupported
component is included in the native full-keyframe trace with Ported=false.
These distinctions were not explicitly retained by Rust generic/full-keyframe
component metadata. The raw bytes alone cannot distinguish zero-bit calibration
from a native no-variant result without the reader context.

Rust now retains optional returned variants on generic component attempts and
optional returned variants/ported state on keyframe spans. KeyframeRecord.attempts
retains all attempted spans, including the unsupported final reader; components
continues to contain successful spans. Old exports have no attempt trace and
unknown new metadata rather than fabricated native values. Native policy is
sampled before component dispatch so later callback map mutations cannot change
the reported calibrated decision.

The regression compares all six CompResult fields, endpoints and JSON roundtrips,
including explicit old-export absence. It is also registered in the WASM harness.
This matrix covers nil payloads; the existing captured-payload and signed-rewind
oracles remain the evidence for six typed payload families. The positive weapon
cases require variant 0x89abcdef, distinct from high word 0x12345678, at both
generic and full-keyframe entry points.

Validation: 627 host tests pass, zero failed, 49 ignored (103.51s); Clippy passes
(22.75s); actual WASM runtime passes, including the ten cases. All 134,657
captured anchors match complete attempted component metadata, alongside existing
payload/source/JSON assertions (68.91s). Six complete documents match (104.91s).
Native fixture bytes, source hashes, formatting and diff checks pass. See
reference/component-result-validation.json. This validates the correction,
not complete parser parity.
