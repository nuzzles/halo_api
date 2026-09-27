# Native view-class context and cumulative observers

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_context_views_test.go.txt`, registered in
`generate_oracles.py`. Expected output comes from native
`DecodeFrameViewsCurseur`, with view classes and view tables enabled.

128 contexts each decode the same payload twice while sharing the observer,
calibration maps and world: 256 passes, 906 records and 1,332 callbacks. Cumulative
native counters report 48 unknown-slot rejections and 50 foreign-view rejections.
Thirty passes complete zero views. Cases include nonempty message selectors,
truncated sources, variable preambles, direct entity-view starts, optional record
prefixes, corruption gates and both simulation-completion settings.

EMP callbacks modify the calibration width for the following rounds component.
That component must skip the new width and must not publish rounds. Crouch
callbacks expose the native capture-slot inheritance: New does not automatically
replace the inherited slot on this path. Simulation-state cases expose the
context's completion policy. View admission terminates a rejected entity view
before its body; generation-strict settings do not replace this admission rule.

Rust compares final cursors, completed views, ordered callbacks, calibration-map
values, record IDs/status/masks, component starts/names/status, world slots and
serialized ProductionFrame retention. Observer counters are compared after each
pass and their increment against that pass's retained admission diagnostics.
World position values are compared as f32 rather than JSON numeric spelling.

This fixture does not cover the alternative view-tables-disabled inference path,
resync, record-mask biped publication or optional position accumulation. The new
configured production entry point explicitly refuses disabled view classes or
view tables instead of silently using a different traversal policy. Anticipated
binding counters are routed but do not have a positive case in this fixture;
earlier native production-admission tests cover their underlying admission rule.
