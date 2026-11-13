# Recorded HUD splash text

`runtime.hud()` provides an on-demand view of canonical archetype-47, row-0
static splash components. It requires v41/27, a uniquely named
`managed-object-networked-splash-message-static-component` in row 0, and
component version 1. `schema_supported()` distinguishes missing/unsupported
schema from a supported stream with no observations.

```rust,ignore
for observation in runtime.hud().observations() {
    let message = observation.message()?;
    // Some(message): localization key and ordered arguments.
    // None: recorded absent-text bit. Err: partial or unsupported component.
}
```

Each observation borrows its canonical component fields and original runtime
event, preserving packet time, stable order, source record and entity ID.
Keyframe samples are observations, not inferred message appearance times.
DELTA admission uses the parser/runtime's established archetype binding; an
unbound DELTA cannot become a HUD observation. Every repeated component remains
separate, including equal text at the same timestamp. An omitted component
produces no observation. Queries neither move playback nor change Film.

`message()` returns the localization key and arguments in source order. Every
argument retains its three-bit kind plus exact value fields and source ranges:
empty, optional five-bit player index, signed integer word, 24-bit real code,
localization key, or unknown-kind word. Absent player differs from empty or
unknown kind. Localization keys are not indices into the variant string table.
Raw text-component suffixes stay accessible as `remaining_fields`; their
semantics are not inferred. The original component fields retain all gates.

`HudValue::reference_number()` exposes pinned libfilmshell's signed-i32 or
offset-binary real conversion, with exact f64 result agreement against its
unchanged `splash::Value::arg`. The real conversion is explicitly reference
behavior; it does not prove the game's original quantizer. Raw values remain
unchanged and unknown types never become numbers or defaults.

This view reports observations rather than continuous screen state. Unread
packets may contain further messages or clears. No last-message/final-state
query, distinct-viewer count, duration or roster/holder attribution is inferred.
Upstream groups identical frame/key/argument observations and calls their count
`seen_by`; that count alone does not prove distinct viewers.

The source layout remains in `film`, its reader in `parser/v41`, and this view
in `runtime/hud`. No wire grammar or admission was relaxed. Independent authored
and native-source evidence is documented in
[hud-v41/README.md](../fixtures/hud-v41/README.md).

Captured verification passes on 37 complete Octagon observations and 504
Recharge observations. All 541 agree with the unchanged native text reader;
every retained field matches the canonical packet bits. Octagon contains only
absent text. Recharge contains 488 present texts with 488 numeric arguments and
one distinct localization key. Other argument kinds, partial components and
schema failures have authored-fixture evidence, not positive captured coverage
in these two films. These counts describe observations, not distinct viewers.
