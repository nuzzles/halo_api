# libfilmshell and a canonical Halo film parser

**Scope update, November 11, 2026:** the current target is broad libfilmshell
capability coverage and a stronger source-backed canonical structure. The
original 1:1 criteria and implementation sequence below remain historical
research guidance, not requirements for finishing the current target. The
current scope is recorded in [LIBFILMSHELL_ROBUSTNESS.md](LIBFILMSHELL_ROBUSTNESS.md).

Studied November 9, 2026. Upstream is pinned to
[`18067987ebc3628ad499c819c6aee814692df43b`](https://github.com/OpenSpartan/libfilmshell/tree/18067987ebc3628ad499c819c6aee814692df43b).
Local comparison uses `simbleau/theater-experiments` at `f2ff6fd` **plus the
existing working-tree changes**. The initial study and diagnostic changed no
production code. The implementation sequence below remains a roadmap; the
first control-reader implementation is recorded in the checkpoint below.

## Finding

libfilmshell is a valuable second implementation with substantial sequential
replication readers, additional component layouts, and useful game-data tables.
It can materially advance our parser. Its current implementation does **not**
establish a complete, field-for-field canonical film format.

Its public `Film` is a reconstructed gameplay model. Even `Film::raw()` exposes
selected projections over readers that can scan, infer types, recover past
stops, or supply zero bits beyond the source. We should keep our source-backed
`Film` architecture and port individually verified wire readers into it.
Gameplay reconstruction remains in `TheaterRuntime`.

The sampled replication results are strong: 12,973 of 13,048 type-0 packets
reached libfilmshell's control endpoint within the last payload byte. However,
this is endpoint evidence under its decoding assumptions, not field fidelity,
full-film coverage, or a measured improvement over our parser.

## What transfers

This comparison records the gaps at study time. The implementation checkpoint
below tracks the subsequent control port.

| Area | Upstream evidence | Local comparison and proposed use |
| --- | --- | --- |
| Control messages | [`components.rs`, `control_view`, `command_record`, `unit_block`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/components.rs#L821) reads kind 1 commands, kind 2 optional payloads, and the secondary unit block. | Our [`frame/controls.rs`](../parser/v41/chunks/replication/replication_stream/frame/controls.rs) explicitly stops on kinds 1/2 and the secondary block. This is a concrete extension target. Preserve each gate, baseline, count, delta-float prefix, value and reference as source-backed fields. |
| Additional components | [`components.rs` dispatch](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/components.rs#L313) contains vehicle/turret, radial-menu, managed-object, matchflow, supply-line and other readers. | Literal `*-component` names: upstream 241, local 210, with 38 upstream-only and 7 local-only. These are dispatch candidates, **not** 38 proven complete new decoders. Audit each conditional branch and registry parameter before porting. |
| Snapshot versus delta encoding | [`snapshot.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/snapshot.rs) and [`creation.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/creation.rs) distinguish snapshot creation/state readers from tick readers. Snapshot positions can carry raw floats; some references change representation. | Compare our keyframe/default readers by branch. `defaults::projectile` already describes a full-state form that is not wired into keyframe traversal. A single delta grammar applied to both contexts is insufficient. |
| Vehicle type state/physics | [`components.rs`, `VehicleTypeState` / `VehicleTypePhysics`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/components.rs#L1714) asserts that an explicitly listed delta component implies its runtime gate, while a snapshot does not. | Our [`m4b.rs`](../parser/v41/chunks/replication/components/m4b.rs) refuses the physics component for missing `vehicle+0x818`. Verify the delta inclusion rule against writer behavior; it could remove a refusal for explicit deltas. Snapshot gate selection remains unresolved. |
| Header/bootstrap | [`header.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/header.rs) locates an event-version table, build/branch strings, start time, variant GUIDs and level tag. | Our canonical registry retains the bootstrap suffix largely uninterpreted. These offsets provide useful anchors. Establish the enclosing versioned grammar and preserve full fixed strings and unknown fields, rather than promoting offset matches directly into canonical sections. |
| Registry parameters | [`registry.rs::versions`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/registry.rs#L66) supplies per-row parameters to readers. Several branches compare them to 1/2/3. | We already retain the raw u32 as `precision_level`. Audit component-specific uses: it is not universally a coordinate bit count. Do not globally substitute upstream's default `Params.version = 4`. |
| Map context | [`game/levels_data.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/game/levels_data.rs), [`levels.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/levels.rs) identify level tags, multiple BSP bounds, region-index widths and axis widths. | Extend and cross-check our existing map glossary. Widths needed to consume a field are parser context; world-coordinate conversion is runtime interpretation. Keep asset/build provenance explicit. |
| Variant/Bond data | [`variant.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/variant.rs) includes a Compact Binary v2 walker and paths for loadouts, respawns, rules and options. | Reuse schema/field-path research after locating the enclosing data structurally. A canonical Bond model must retain wire types, order, exact numeric bits, strings, byte arrays, unknown fields and collection entries. Upstream's selected callbacks and rounded numeric projections are not that model. |
| Events and game semantics | [`events.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/events.rs) reads kill/assist, weapon, projectile, action and scripted game events; `game/*_data.rs` names assets and options. | Compare bit layouts first, then add semantic names in runtime. Our current code-0 damage reader already has the corrected gate and broadly the same field sequence: do not treat older coverage prose as evidence that this remains a new upstream fix. |

The implementations are complementary. At this original study pin,
libfilmshell stopped on the second `unit-command-tick-component` form, while our
[`unit.rs`](../parser/v41/chunks/replication/components/unit.rs) read its extra
tick fields and tail. The current f050371 reader handles that form too. We also
had an explicit type-1 datum/mask table model; libfilmshell's public raw model
does not provide an equivalent typed table.
Replacing our parser wholesale would discard existing structural work.

## Boundaries that upstream has not solved canonically

1. **Highlights are still scanned.**
   [`highlights.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/highlights.rs)
   tries bit positions for `2d|25 c0`, an XUID in a chosen numerical range, and
   an end marker 15,406 bits later. Unknown type hints default to kills.
   This does not establish sequential type-9 record boundaries. Keep our
   summary count plus opaque stream until the record grammar is established.
   Its SPNKr attribution is a further research pointer, not evidence of a
   complete enclosing record grammar.

2. **Roster/player/bot record boundaries remain heuristic.**
   [`roster.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/roster.rs)
   scans all eight bit shifts for type-8 XUID patterns and searches a
   1,900–2,300-byte range for each next type-12 bot record. Bot takeover uses
   temporal proximity. These can inform runtime evidence, but do not complete
   canonical packet types 8 or 12.

3. **Replication can recover or infer.**
   [`payload.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/payload.rs)
   searches for the tick message after stops, defaults types by entity-index
   range, tries possible archetypes against plausible successors, and consults
   a later snapshot to resolve some ties. Its resolver also compares candidate
   packet endpoints. These techniques must not establish canonical record
   boundaries or silently supply an unrecorded type.

4. **Snapshots also have recovery and an unavailable gate.**
   `snapshot::entities` appends scan-discovered records after a stopped walk.
   `snapshot::walk` tries vehicle-state variants and selects one by the next
   plausible slot. A successful continuation does not prove that the chosen
   missing runtime value was serialized. Size words `n` and `m` are described
   as game structure sizes, so they cannot simply be used as wire lengths to
   skip unknown bodies.

5. **Script event 15 still assumes a layout.**
   [`events.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/events.rs#L528)
   reads `15 + 13` bits under a comment about the optional fields' usual state.
   Our parser's `MissingRuntimeGate15` is therefore not resolved by this
   implementation. Establish where the controlling state comes from.

6. **Bit reads are not bounded to the owning record.**
   [`bits.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/bits.rs)
   explicitly supplies zero bits outside the buffer. Many readers receive
   the whole chunk, so intermediate reads can also cross a packet boundary
   before a later endpoint check. A port must use our bounded reads and retain
   failed reads as unavailable, including each prefix of variable layouts.

7. **The raw API is selective.**
   [`format.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/format.rs)
   exposes event kind/handles/start, entity operations and snapshot entities,
   not every consumed field and extent. Original compressed chunks are not
   retained by its assembled container. Raw-frame iteration can fall back to
   marker framing when packet framing fails. This API is useful for inspection
   but cannot serve as our canonical AST or a field-level oracle unchanged.

8. **Version selection and unexplained packet kinds remain work.**
   Header admission uses a printable numeric build at a fixed offset. This
   is not a complete film-version/registry-revision compatibility matrix.
   Packet kinds such as 6 and 10 are identified at the envelope level without
   a full body model. Verify each supported build and all observed packet
   bodies rather than inheriting support from a recognizable header.

## What “1:1 canonical” should require

For a declared supported format/build/schema, parsing must follow the recorded
hierarchy and consume every section sequentially. Every recorded gate, count,
identity, enum code, mask, scalar, string, reference, extension and padding field
must have its exact bit representation and source range. An unknown enum value
can remain numeric when its wire width is known; an unknown record boundary
cannot be invented.

Recorded absence, a protocol-implied default, missing external context and a
truncated read must remain distinct. State required to decode a delta is private
parser context; it must not become a second recorded declaration in `Film`.
Any external game schema needed to parse the wire must be identified and
versioned. The unresolved vehicle gate shows why universal decoding from film
bytes alone is not yet established.

There are two separate completion tests:

- **Complete structural decoding:** no unexplained/unparsed bit ranges or
  heuristic continuations on the supported fixtures; independently checked
  fields, ordering and boundaries. Retaining an opaque buffer is lossless
  preservation, but does not finish decoding that buffer.
- **Exact re-encoding:** if 1:1 also means a reversible format implementation,
  encode the parsed fields back to identical *inflated* bytes, including reserved
  bits, floating-point bit patterns, string padding and wire encodings. Replaying
  the original stored byte buffer is not a parser/encoder round-trip test.
  Compressed transport identity additionally requires retaining the original
  compressed bytes; a valid recompression need not be byte-identical.

Neither repository currently establishes both tests for the entire format.
Keep unknown data and explicit stops throughout implementation, and report them
as incomplete coverage until their grammar is verified.

## First control-reader implementation (2026-11-09)

Kinds 1/2 and the secondary unit block now have bounded v41 readers. The existing
organization is retained: canonical models in `film`, wire decoding in
`parser/v41`, and indexing/interpretation in `runtime`. Commands and secondary
blocks are child modules of the existing control-view reader. They use the
shared component field reader and preserve gates, baseline values, raw float
bits, delta prefixes, handles, counts and source ranges. Kind 2 has no fabricated
player index. Kind 3 stops immediately after its kind bits because the baseline
layout is disputed. Missing position context and truncated reads remain explicit.

`ControlEntry` now has a `ControlPayload` enum; existing kind-0 projections live
in `ControlInput`. Consumers use `entry.index()` for an optional recorded player
index and match `entry.payload` for kind-specific data. Exact fields remain in
`DecodedFrameView::fields`; no runtime reconstruction becomes a canonical field.
This changes the Rust construction and serialized shape of `ControlEntry`.

The [17 wire fixtures](../fixtures/controls-v41/README.md) have an independent
read/skip trace exported from the pinned libfilmshell implementation. Eight tests
cover the primitive boundaries, every-bit truncation, source preservation,
relative command transitions, missing context, kind-3 refusal and integration
through `Film::parse` and runtime filters. Canonical traversal now requires an
entity terminator before controls; a rejected entity header cannot establish
that boundary. A negative case verifies that a control-shaped unread body
remains unparsed and produces no runtime controls. This completes the first control
slice of priorities 1/2, not the component-family audit or the full canonical
format. Real-film occurrence and semantic meanings of the newly supported
branches are not established by synthetic parity.

See [the validation checkpoint](VALIDATION.md#libfilmshell-control-reader-checkpoint-2026-11-09)
for corpus results and a pre-existing historical-oracle failure reproduced on
starting commit `f2ff6fd`. No old golden expectation was regenerated.

## Implementation sequence

| Priority | Work | Acceptance evidence |
| --- | --- | --- |
| 1 | Build a field/boundary differential harness around the pinned upstream readers. Trace reads and skips; record assumptions/recovery separately. Inventory stops in our current canonical production traversal. | Same payload, bit start, registry parameter and explicit context on both sides. Compare raw bits and endpoints; do not regenerate the existing LevelUp oracle. Missing corpus must fail the required coverage job. |
| 2 | Extend control kinds 1/2 and secondary blocks; resolve the kind-3 disagreement first. Audit and port the 38 candidate component names in small families. | Independently authored field goldens, each conditional branch, generation wrap, truncation at each field, and packet-local bounds. The original reader treated kind 3 as a continuation; upstream consumes a baseline then refuses it. The upgraded reader stops before that disputed baseline. Completing kind 3 still needs writer/fixture evidence. |
| 3 | Audit creation and full-state versus delta readers, parameter versions, component-inclusion masks, projectile snapshot forms and vehicle delta gates. | Sequential snapshots and frames with no header searches, no speculative type selection and no inferred runtime gates. Full fields and next boundaries must agree under verified context. |
| 4 | Parse bootstrap/event-version/build/variant sections structurally; connect versioned map/tag context where widths depend on assets. | Boundaries derived from preceding sections. Preserve full Bond representation and all unknown fields. Verify supported build/catalogue combinations and explicit missing-context failures. |
| 5 | Reverse-engineer the remaining summary, roster, bot and other packet bodies and event-15 runtime dependency. | Count-driven or terminator-driven record traversal from an established start, with exact end and trailing-bit accounting. Marker scans and plausible-successor searches cannot pass this gate. |
| 6 | Close the full-corpus coverage gaps and, if reversible encoding is part of the contract, implement the bit writer. | All supported films/branches accounted for; no silent skipped tests; independent field fixtures; exact inflated-byte round trips including unknown enum values and legal padding. |

Start by reusing the existing canonical models, field reader, source coverage and
version dispatch. Avoid importing upstream `assemble`, motion calibration, inferred
jumps, roster takeovers or clock fitting into `Film`. Those are separate runtime
features, even when they produce useful replay output.

The initial study did not vendor decoder code.

## Validation performed

The unmodified pinned checkout passed `cargo +stable test --offline --all-features`:
35 unit tests, 9 input tests, 15 corpus-named tests, and 4 doctests. **The 15 corpus
tests did no corpus work**, because the upstream `films/` directory is not
distributed. Some unit tests also return early without those films. A green
upstream test run is therefore not evidence of its claimed real-film coverage.

A separate diagnostic invoked the actual upstream readers on our retained v41
recordings. For each of five films it sampled the first, middle and last
replication chunk, with the original header, verified decompressed sizes and
SHA-256 hashes. It selected widths from upstream's header level tag/catalogue,
seeded each chunk's walker from its snapshot, then walked type-0 payloads and the
control view. The diagnostic did not invoke the high-level assembler or the
recursive ambiguous-type resolver/next-snapshot machinery.

| Film | Sampled replication indices | Frames | Control ends in last byte | Sequential snapshot entities, per sample |
| --- | --- | ---: | ---: | --- |
| Controller / stick circles | 1, 3, 5 | 3,018 | 3,018 | 207, 218, 218 |
| Bazaar idle | 1, 2, 3 | 2,919 | 2,919 | 265, 307, 306 |
| Octagon / AR kill | 1, 3, 5 | 3,035 | 3,035 | 209, 222, 222 |
| Hour-long raid | 1, 100, 198 | 1,470 | 1,470 | 306, 402, 395 |
| Ranked Oddball | 1, 31, 61 | 2,606 | 2,531 | 247, 317, 295 |
| **Total** | **15 chunks** | **13,048** | **12,973** | |

All 15 snapshot walks reported no stop and added no entities through the scan
fallback. Their reader still contains the vehicle-layout selection described
above; the diagnostic did not instrument whether that alternate branch ran.
Oddball chunk 31 stopped on `Component(35, 0)` in 74 frames and `BadHeader` in
one. One frame used tick-message resumption; the raw output separately counts
successful endpoints without that resumption. No sampled successful control
view yielded kind-1 command messages or secondary blocks. Kind 2 does not yield
a `ControlMessage`, so this diagnostic does not establish its occurrence.

These measurements establish reader behavior on selected inputs. They do not
prove field meanings, validate the new control branches, cover all 32 local
films, or establish relative coverage against halo_api. The existing halo_api
test suite and full corpus were not rerun for this documentation-only study.

## Reproducing the diagnostic

Retained artifacts: [probe.rs](research/libfilmshell/probe.rs),
[prepare.py](research/libfilmshell/prepare.py),
[inputs.json](research/libfilmshell/inputs.json), and
[results.txt](research/libfilmshell/results.txt). The full literal-name candidate
inventory is in [component-candidates.txt](research/libfilmshell/component-candidates.txt).

Clone/check out the pinned upstream commit into a disposable directory. Prepare
inputs using `prepare.py CORPUS OUTPUT`, where `CORPUS` contains the retained
`experiments/films` recordings. It verifies the retained hashes. Copy `probe.rs`
to upstream `src/study_probe.rs`, and append `#[cfg(test)] mod study_probe;` to
that disposable checkout's `src/lib.rs`. Then run, inside that checkout:

```sh
LIBFILMSHELL_STUDY_INPUTS=/absolute/path/to/prepared-inputs \
  cargo +stable test --offline --lib halo_api_study_probe -- --nocapture
```

Offline execution requires the dependencies to be cached. The study used the
installed stable compiler, Rust 1.98.1, instead of installing the checkout's pinned 1.97.1
toolchain. The probe is diagnostic glue for private upstream interfaces; it is
not part of halo_api's compiled library or a claim of canonical conformance.
