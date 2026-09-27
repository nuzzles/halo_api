# Cached equipment placement publication

build_facts_replay_equipment_placements accepts exact cache placement, calibration,
spawn, change, census, and position DTOs plus caller identity/catalog context.
It shares the recording publication loop, owner selection, origin classification,
and disappearance-bound calculation. Neither adapter constructs synthetic source
packets or overwrites retained cache payload.

Placement coverage lives, anchors, confirmed, and spawnLists retain native i64.
Calibration widths stay signed through the cache adapter: calibrated means both
are positive, with the native lead/index string when valid. Native publication
still accepts supplied placements when calibrated/scanned are false. Those flags
report evidence; they do not introduce a new publication gate. Empty placement
input or zero time step returns initial coverage before counting spawn inputs.

Positions keep native input ordering (callers supply timestamp-sorted positions).
Owner selection takes the nearest-in-time position per slot within 250ms, then the
nearest eligible owner within 3m with lower-slot tie breaking. Heading comes from
that same slot within 200ms and retains full cache u32 yaw in native conversion.
Placement coordinates remain unrounded, and no owner publication-track membership
is required. Frame filtering, clamping, and stable (time, ID) ordering are shared.

Only written spawn/death/taken evidence or the native manifest-piece fallback can
supply origin causes. Unknown change kinds do not become taken evidence. The
builder returns the fallback occurrence count for the existing outer document
counter; complete BuildFromFacts fallback restoration remains separate work.
Caller catalogs retain their existing String contract; raw native catalog string
domain reconciliation is part of the remaining broader API audit.

halo_rust_facts_placement_publication_test.go.txt supplies 1,024 native complete
placement-publication expectations and actual native fallback reports. It extends
the placement-evidence oracle with signed counter/calibration extrema, full u32
aim, empty input, caller family overrides, and unconfirmed calibration with
nonempty supplied placements. Check facts-placement-inputs-validation.json for
completed checks. Native agreement here covers this adapter; controlled film
semantic observations and complete document/capture parity are still required.
