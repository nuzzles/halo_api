# Snapshot objects and stationary reference candidates

`runtime.objects(ObjectSchema::V41Registry27)` reads equipment (37), vehicle (40),
weapon (42) and device (43) keyframe records. It preserves the existing
film / parser/v41 / runtime organization and leaves canonical Film unchanged.

`snapshots()` retains every selected record, including missing/partial defaults,
unsupported positions, zero coordinates and incomplete tables. Each
`ObjectSnapshot` borrows its Event, KeyframeRecord and KeyframeTable.
`header()` checks the native slot identity, header source bits and uniqueness of
the full namespace ID. It does not turn namespace bits into a runtime generation.

`tag()` requires a complete current default section with contiguous source fields
and a unique 32-bit `mpp.object_tag`. `position()` requires a complete unique
parameter-1 position row in the v41/27 registry, contiguous extents, exact source
bits and a valid component-check suffix. Both underscore and dotted check names
use the shared structural validator; the unknown checksum algorithm is not
claimed to be verified.

Snapshot world positions are three little-endian float32 words after a zero
prefix: three bits for vehicle dynamic positions, one bit for the other selected
objects. Unsupported prefixes/attached forms retain canonical fields and refuse
world projection. Nonfinite coordinates refuse; signed zero is preserved.
`ObjectPosition` retains the raw word fields, finite field and engine coordinates.
Zero coordinates alone do not establish a holder or owner.

## Explicit reference policy

`reference_stationary(Catalog)` implements the
libfilmshell `8f0f8d0` snapshot comparison policy, with explicit source safeguards:

- Eligible observations require a complete native table, a complete record,
  flags 3, a current tag and a finite nonzero world position.
- Catalogue entries marked as vehicle parts are excluded. Unknown tags remain
  available; catalogue selection does not establish build or instance ownership.
- Runs match the entire snapshot namespace ID, tag and archetype. Every axis must
  stay within `0.05f32` engine units of the **first** position, not the previous one.
- `runs` retains singletons and every contributing sample. `confirmed()` requires
  at least two samples; `confirmed_at()` identifies the second Event. Its earlier
  anchor is retrospective evidence, not an earlier confirmed observation.
- `reference_xy()` uses the original anchor with the upstream `[-10*y, 10*x]`
  map-display convention. Original engine XYZ remains available.

These are comparisons of repeated snapshot addresses. They are not proven
physical-object lifetimes or evidence of continuous immobility between snapshots.
Even fully decoded tick packets can contain intermediate movement that this
snapshot presentation rule does not evaluate. No player/holder, spawn, pickup or
destruction attribution is implied.

## Boundaries and uncertainty

A complete subsequent table can establish that an address is absent. A present
address outside the selected object projection is `NotProjected`, not absent.
Position/tag/archetype changes and explicit exclusions close a run with the new
source observation. An incomplete table, missing chunk, unframed region or source
discontinuity closes comparison support without claiming absence. Completeness
requires native slot metadata, all 8,191 headers, a sub-byte tail and complete
retained records in addition to the table's End status.

Unread entity regions within framed tick packets cannot hide a separate snapshot
packet. Those boundaries are retained in each active run's `tick_gaps`, allowing
the snapshot-only comparison while explicitly withholding continuous-lifetime
claims. Incomplete snapshot tables break runs. Neither kind of gap invents a
timestamp; opaque chunks retain their original source boundary. Open runs have no
synthetic final end or destruction time.

`excluded` retains each rejected observation and reason. Raw snapshots remain
accessible independently, including readable prefixes of incomplete tables.
Queries do not move the runtime playhead or mutate Film serialization.

## Evidence and remaining work

[objects-v41](../fixtures/objects-v41/README.md) executes the unchanged original
world reader and raw-position helper on nine authored input sequences. Local
tests carry those same values through independently traced creation bodies and
authored native keyframe packets. Separate tests verify checked trailers, all
four archetypes, source mutations, duplicates, namespaces, incomplete tables,
tick/chunk gaps and reclassified addresses. The existing creation/component
suites retain their exhaustive truncation scope; these tests do not replace it.

Retained-film counts and final validation are in VALIDATION.md. This implements
snapshot object evidence and labelled stationary candidates. `weapons()` now adds
tick weapon-object histories and explicit same-packet/tag pickup candidates; see
[GROUND_WEAPONS.md](GROUND_WEAPONS.md). Resting-time interpretation, physical pickup
identity, broader position encodings and whole-format round-trip proof remain
open. Tag/packet coincidence cannot establish which object a player picked up.
