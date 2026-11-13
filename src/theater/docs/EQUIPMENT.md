# Recorded equipment energy and delay

`runtime.equipment(EquipmentSchema::Reference8f0f8d0)` exposes source-linked
observations, object histories and exact-event state over existing canonical
equipment components. This view lives in `runtime/equipment`; the existing
`runtime/replay/equipment` selection/inventory organization is unchanged.

`observations()` retains each energy and delay component, including repetitions,
zero, partial reads and unsupported candidates. Each observation borrows its
original event and fields, retains the exact payload bit range, and validates
its bytes against the original packet before returning a value. Admission
requires registry v41/27, declared class 37, an unambiguous matching registry
row and the supported parameter range. Corruption-check trailers are validated.
Errors remain inspectable through `value()`; they never become zero.

Energy is an unsigned fourteen-bit word. `reference_energy_fraction()` follows
the pinned endpoint-aware f64 quantizer: 0 and 16383 are exactly 0 and 1; interior
codes are bin midpoints. `reference_energy_percent()` reproduces upstream's
rounded, lossy percentage. Delay retains every ten-bit word, including 1022 and
1023 if recorded. There is no elapsed/remaining conversion, reconstructed start,
inferred recharge rate or physical time-unit claim. Upstream suppresses some
countdown-like repeats; this view retains every source update.

`history(entity)` retains evidence changes, and `at(entity, timestamp)` includes
all same-clock records and any unread packet suffix. `at_event(entity, event)`
stops at that exact runtime-owned event; foreign events are rejected. Energy
and delay update independently. Missing fields preserve prior supported values;
NEW records, deletions, generation changes, reclassification, partial records,
source gaps, keyframe boundaries and opaque packets interrupt support. Readable
fields before a partial suffix remain available as observations even when the
post-record snapshot is unknown. Runtime generations and keyframe namespaces
are distinct `EquipmentEntity` variants; no baseline value crosses that boundary.
Absent creation components do not establish an implicit energy/delay default.

State queries require source-ordered, nondecreasing packet clocks. Leading
zero-clock bootstrap packets are accepted; a late zero or backwards clock
produces `UnresolvedClock` for the supplied recording's state/history queries.
The evidence points to the original envelope. Raw observations remain readable.
This is explicit refusal pending clock reconstruction, not a fabricated clock
or a claim that the original file is corrupt.

`ownership_at_event(entity, event)` separately exposes the current attachment
path and controller/roster evidence. A participant requires a traversable direct
class-35 parent, its current controller and a unique holder from independent
human/bot populations. Detached, missing, deleted, unmapped, conflicting and
stale-generation paths retain their evidence. Later same-time listings do not
backdate earlier observations. Opaque packets invalidate parent/controller
support until fresh observations establish it; unresolved clocks also prevent
participant claims. The raw path and controller remain inspectable on refusal.

These are on-demand queries that walk source prefixes. They do not establish a
complete equipment inventory, infer avatar lifetime, or prove continuous
recharge. Scalar, attachment,
controller and roster evidence are separate; a known scalar does not imply a
known participant. A later holder query must not relabel an earlier scalar's
source-time ownership.

## Contextual replay

The cached counterpart lives in `runtime/replay/equipment/energy.rs`, beside
the existing selection/inventory channel. `ReplayDiagnostics::equipment_energy`
retains every contextual scalar read, its exact source fields, value or error,
declaration, record completeness and optional relative packet clock. Snapshot
reads have the separate `EquipmentEntity::Keyframe` namespace and never seed
runtime object state. This contextual decoder's admission is distinct from the
canonical on-demand view; the contextual replay's explicit policies still apply.

`PlayerReplay::energy_observations` indexes readings attributed at their source
time. An observation's `ownership` retains candidate controller/holder evidence;
`participant` additionally requires the direct equipment-to-biped path and fresh
parent/controller support. `support_interrupted` identifies refusal due to stale
support after a gap. Candidate ownership alone does not establish attribution.
Later roster or parent updates never rewrite the original observation or add it
to a later holder's source-observation list.

`energy_samples` and `energy_at(timestamp)` expose all currently supported
equipment objects, with independent optional energy/delay indices into the
diagnostics. Each object keeps its current ownership and attachment path;
readings can predate that ownership. A later holder may receive supported object
state without acquiring historical source attribution. An empty sample means no
supported readings, not a complete assertion of empty equipment inventory.

NEW/DELETE, generation reuse, reclassification and partial records invalidate
cached values. Complete fields before an unread suffix remain diagnostic
observations. Opaque packets and unread control suffixes end this channel's
samples and require fresh parent/controller support. Missing source and general
replay gaps also apply; fresh evidence in the following chunk can restore
support. Samples are bounded by the existing biped ownership intervals and
requested replay endpoint. Untimed packets retain an absent clock and use the
preceding envelope's exclusive support bound; they do not become timed events.

Those packet/control boundaries now also interrupt the other replay channels
and current ownership. The energy diagnostics preserve the original gaps and
source readings; absent parent paths invalidated by a general gap also mark
`support_interrupted`. A fresh direct parent/controller chain can restore object
support without relabelling its earlier readings.

The five contextual integration tests reuse the same unchanged-reader scalar
oracle and add multi-object handoffs, population conflicts, generation/source
loss, timed/untimed opaque bodies and partial control suffixes. Authored tests
supply an explicit bootstrap policy to their small registry; they do not prove
public bootstrap admission. Captured audits verify source bytes and attribution
links without claiming complete equipment or continuous recharge coverage.

The [reference fixture](../fixtures/equipment-energy-v41/README.md) runs unchanged
pinned component and ammo readers. Authored native packet tests check field
bytes, quantization, independent updates, source order, human/bot conflicts,
opaque and missing-source gaps, partial suffixes, namespace separation and
Film/playhead immutability. Captured audits check all available scalar source
values; they do not certify complete ownership coverage. See [VALIDATION.md](VALIDATION.md).
