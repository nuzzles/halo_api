# Next phase: faithful Film, resolved replay, and playback

Current API migration (2026-09-27): the new two-layer goal is described in
[TWO_LAYER_API.md](TWO_LAYER_API.md). The public recording entry is now
`theater::film::Film::parse`; conversion produces `theater::resolved::ResolvedFilm`.
The historical parity closeout and requirements below remain a record of their
original scope and sequence.

Captured from the user's requirements on 2026-09-24.

## Sequence and implementation boundary

Finish the current pinned LevelUp v41 parity work first. This document records
the next phase; it does not redirect the current implementation or change its
acceptance criteria. Keep the current parity goal active until its existing
requirements are verified. Do not begin this architectural refactor yet.

After parity is complete, propose the concrete types, migration plan, explicit
fidelity contract, and validation matrix before starting the refactor. The names
below describe intended responsibilities; `ResolvedFilm` is tentative. Preserve
film-module ownership and account for current public APIs and portable exports
in the migration proposal.

## Layer 1: Film — the native recording

`Film` must faithfully represent the canonical recording hierarchy and order:
chunks, packets, views, records, component fields, and summary data.

Preserve:

- Native identities, timestamps, masks, quantized values, and record ordering.
- Source byte and bit ranges.
- Unknown fields, opaque payloads, and unparsed regions.
- The distinction between recorded data and data that cannot be decoded.

Define “1:1” explicitly as structural fidelity and preservation of all source
information. Explain how retained source information covers decoded and undecoded
regions. Byte-for-byte re-encoding is a separate capability: document its scope
and support status rather than implying that faithful decoding provides it.

Do not silently discard source information or substitute interpretations for
recorded values. Player timelines, inferred events, interpolated positions, and
convenient query indexes belong in the higher layers.

## Layer 2: resolved replay model

Build a separate higher-level structure, tentatively `ResolvedFilm`, by walking
`Film` chronologically. Resolve entity lifetimes, player identities, accumulated
component state, and relationships into storage suitable for querying and
introspection.

Expose an ordered event stream for every supported observable state change or
action, including human-readable events such as “Nuzzles jumped at 0:10” when
the evidence supports that interpretation. Each event must carry:

- Timestamp and stable ordering among events with the same timestamp.
- Event kind/category and relevant entities or players.
- Typed payload, with previous and new values where appropriate.
- Source references back to `Film`.
- Recorded-versus-derived provenance, identifying the derivation when derived.

Jump input and an inferred physical jump must be distinguishable event kinds.
Do not present inferred physical actions as directly recorded facts. Define
derivation rules and uncertainty explicitly.

Support filters by category, event kind, player/entity, and time range. Retain
continuous position and aim updates. Consumers may hide those updates from the
human-readable log without dropping them from the resolved model or playback.

## Layer 3: stateful playback and browser visualization

Provide a playback API that maintains the current resolved world. Access to the
already-materialized current snapshot must be O(1), preferably a borrowed view.
State the other costs separately:

- Advancing applies the intervening updates and has a cost tied to that work.
- Enumerating or copying the world scales with the world size.
- Arbitrary timestamp seeks require an explicit checkpoint/index strategy.
  Specify its time/memory tradeoffs; do not claim O(1) seeks without evidence.

The browser deliverable is a 3D replay of player positions and aim on a map, with
a synchronized, filterable event log underneath. Reconstruct everything the
recording supports while preserving gaps and uncertainty. Keep visual
interpolation separate from recorded state. Identify required external map
geometry and assets and their provenance.

## Validation requirements

Build a versioned golden-film suite with independently established expected
actions, actors, timestamps, and outcomes. Inventory every known action of
interest and cover positive cases and negative cases where an event must not
appear. Do not narrow that inventory to actions already implemented.

The proposal must map each requirement to fixtures, assertions, independent
evidence, and remaining coverage gaps across five complementary checks:

| Check | Required evidence |
| --- | --- |
| Native decoding | Compare ordered records and fields against the pinned reference parser; retain unknown data and verify source boundaries. |
| Semantic events | Compare resolved events with independently annotated golden films; define timestamp tolerances and distinguish exact wire timestamps from observed action times. |
| World state | Assert identities, lifetimes, positions, aim, and other supported state at selected timestamps. |
| Playback consistency | Seeking to a timestamp must produce the same resolved world state as sequential playback to that timestamp. |
| Browser integration | Verify synchronized rendering and event logging, filtering, pause/resume, and backward seeking. |

Golden expectations must not simply be generated by the implementation under
test. Reference-parser agreement is complementary evidence, not a substitute for
controlled recordings and independent observations. Record fixture versions,
annotation provenance, timing tolerances, negative cases, and uncertainty in the
validation design.

## Proposal deliverables after parity

Before refactoring, present:

1. Concrete types and ownership boundaries for the three layers, source links,
   event provenance/order, and playback snapshots/checkpoints.
2. A migration plan for the current Film API, resolved data, portable exports,
   existing consumers, and browser integration.
3. The fidelity contract, including unknown-data retention, timing and ordering,
   recorded/derived/interpolated distinctions, gaps, and re-encoding status.
4. A validation matrix covering the five checks above and the full agreed action
   inventory, with independent golden evidence and explicit missing coverage.

This is a queued requirements record, not the concrete design proposal or an
assertion that the current implementation already satisfies these contracts.
