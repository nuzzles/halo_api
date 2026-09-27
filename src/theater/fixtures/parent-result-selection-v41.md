# First decoded parent result selection

Reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: reference/halo_rust_parent_result_selection_test.go.txt, registered
in reference/generate_oracles.py. Native generation passed (0.382s).

Sixteen native generic biped records contain parent, EMP, parent components.
The EMP hook changes the shared calibration map between parent attempts. Four
patterns per policy cover first attached/second free, skipped first/decoded
second, both skipped, and first free/second attached. Native occupancyFromRecord
selects the first CompResult with a parent payload, skipping nil payloads.
Each policy publishes one occupancy per case except both-skipped, which emits
none. The actual objectDeathHarvest output, component results and endpoints are
retained in parent-result-selection-v41.json.zlib.

The initial Rust comparison reproduced a missing free-parent transition: the
first successful named parent attempt was a calibrated skip with no parent
fields. vehicle_occupancy_from_record now continues until an attempt projects a
parent value. It still chooses the first decoded value, including an explicitly
free parent, rather than preferring a later attached parent. Both-skipped cases
remain absent. Field decoding and component ordering are unchanged.

All 16 focused cases pass (0.01s), including complete native occupancy fields,
cursor endpoints and a portable record roundtrip followed by the same projection.
The regression is registered in the WASM harness. The initial failure remains in
/private/tmp/halo-parent-result-before.log. Broader checks are tracked in
reference/parent-result-validation.json; full parity is not implied.

Final checks pass: 629 host tests, zero failed, 49 ignored (122.29s); Clippy
(18.02s); actual WASM runtime (18.48s build); four captured march films (83.94s)
and six complete documents (112.53s). Native fixture bytes, 485 source hashes,
formatting/diff and reference script syntax also pass.
