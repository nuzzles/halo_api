# Canonical v41 model and parser audit

This records the completed Phase 1/2 canonical-model/parser audit. Runtime redesign and private resolved
storage remain follow-on work. Existing resolved consumers migrate only to follow
canonical structural changes.

## Established changes

- Registry archetypes own component slots, their original name bytes, precision
  levels, and source/padding ranges. Duplicate block traces are removed.
- Type-0 packets own the first configuration bit and one sequential view traversal:
  message list, entity records, then controls. Message fields belong to messages.
- Incomplete message lists and conflicting code-0 message layouts refuse entity
  traversal without mutating grammar bindings. Overlapping continuation projections
  and their state policies are no longer public canonical structures.
- Frame header publication checks source bounds. A header refusal preserves the
  established preceding records and reports a truncated entity view.
- Source guards reject negative coordinates, negative widths, and overflowing
  endpoints. Layout skips retain available source fields and stop instead of
  moving beyond source or backwards over established fields.
- No-archetype keyframe entries require their full fixed header to fit; a short
  header reports `Truncated` rather than publishing an out-of-source extent.
- Component corruption-check gates belong to their component even when their
  value is truncated. Such components report `Truncated`.
- `PacketRead::Decoded` means a body decoder ran, with nested outcomes still
  explicit. It does not imply full-payload decoding.
- Summary candidate reads in resolution retain all 16 UTF-16 units, including
  terminator and padding. Canonical summaries retain the count and complete
  opaque record stream; no marker search establishes canonical record boundaries.
- Summary associations explicitly use `GuardedV41Layout` derivation and
  `DerivedSummary` event provenance. Raw bounded fields remain source-linked.
- Missing vehicle runtime state reports `RuntimeContextUnavailable` at the
  component's independently captured start boundary, without reading its body.
- Empty frame payloads do not invent a false configuration bit.
- Parser declarations retain a source-read NEW ID/archetype across a truncated
  body, and a recorded DELETE header clears the schema across truncated metadata.
  Records remain truncated; these grammar changes do not materialize lifetimes.
  Unsupported/invalid NEW bodies do not establish a parser declaration.

## Latest structural audit

- Reference/equipment/position callback projections are private parser observations.
  Entity/keyframe records no longer carry duplicate reference traces with synthetic
  absent values. Canonical action-mask reversal and absent-weapon sentinels are
  removed; their complete source-backed fields remain owned by their wire view.
- Partial nested events report actual field extents rather than parent return
  positions. Failed requested reads remain explicitly unavailable. Field publication
  stops at that failure rather than retrying smaller fields at the missing word.
- Packet payload access checks recorded header bytes, contiguous bounded ranges,
  payload length and decoded body type. Public construction/serde contracts are
  documented; this does not claim validation of arbitrary nested mutations.
- Packet coverage partitions all payload bits into fields, opaque, known padding,
  or unparsed regions. Coverage derives from the model without a second decode.
  Invalid/out-of-source/conflicting classifications refuse the coverage query.
- DELTA archetypes are no longer published from inherited schema bindings.
  NEW keeps the explicitly recorded archetype; decoding context remains private.
- NEW/keyframe default-state sections own ordered fields, range and structural
  status independently of record guards/masks and component updates. Resolution
  combines those source fields chronologically for its existing baseline storage.

- Datum headers and component masks now remain separate wire tables, with explicit
  source ranges for entries, masks, tail words, and padding. Masks preserve recorded
  MSB-first words; the former reversed index projection and synthetic Default
  generation are removed. An independent two-slot layout tests both tables.

Final checks pass after all ownership/layout changes and first-failure event
publication: 86 ordinary tests, 7 doctests, and the separately run 32-film corpus.
The latter checks 403,465 supplied reference contexts, one source-verified runtime
refusal, and 3,667 derived summary candidates. All captured packets have bounded,
non-conflicting complete source-coverage partitions. Clippy (all features/targets,
warnings denied), strict Rustdoc, WASM library checking, formatting, diff whitespace,
and all 14 versioned fixture sizes/SHA-256 hashes pass.
Unknown grammar remains explicitly preserved; this audit does not establish
complete field-for-field decoding of unsupported layouts or re-encoding support.

## Validation checkpoint

The first captured-corpus run before this goal's changes failed on a reference
endpoint at bit 34 of a 32-bit payload. The controller binding divergence is now
traced against commit `d61443ef59268ad734355db8e9974f68db5ca6d0` using the retained
Go baseline harness: NEW slot 256/archetype 21 at payload byte 492675 records its
declaration but the reference reads through bit 214 of a 208-bit payload before
binding. Rust previously dropped that declaration. DELETE metadata at byte
501401 also exceeds the source, causing a stale Rust schema. Source-only
declaration updates now make the controller clip pass its boundary checks.

The next mismatch is `natural-end/04-walk-forward/chunk-002-type-2.bin:509578`:
Rust has a slot-7200/archetype-27 declaration that the reference world lacks.
It consequently attempts a delta and stops at bit 184; the reference rejects
its header and completes views at bit 183. The Go trace identifies NEW slot 7200
at byte 501189: it reads beyond the 208-bit source through bit 254 and refuses
unsupported component 55, so it never binds. Rust stops at bit 190 while reading
the dense mask, but retains the directly read ID/archetype declaration. This is
an established difference between source-only declaration context and a reference
context influenced by synthetic tail bits. Earlier source truncation can produce
different subsequent contexts even when a later reference endpoint fits. These
differences were resolved for reader comparison by supplying independently
captured reference contexts per invocation, as described below. They remain
explicit differences in cumulative declaration context.

The endpoint-only golden is insufficient to isolate reader defects from these
context differences. Independent Go evidence now includes 1,241 reference-context
snapshots across all 32 films; all 403,465 endpoints were checked against the
unchanged original golden before retention. The corpus reader check restores the
same supplied reference context for each frame. Separate canonical assertions
check source bytes, frame/keyframe boundaries, and component fields bit for bit.

Under matching contexts, the initial reader difference in
`raids/01-hour-long-raid/chunk-001-type-2.bin:622938`: Rust stops at unsupported
`vehicle-type-physics-component` at bit 450, while Go ends at bit 739 (one completed
view in both). The pinned `composants_vue_b_m4b.go` explicitly assumes an unrecorded
vehicle+0x818 flag. The supplemental Go fixture now captures that component's
start at bit 450; Rust refuses there instead of making the same assumption.

The 32-film corpus subsequently passed all 403,465 supplied-context reader checks,
with one independently bounded runtime refusal, source-backed component-field
checks, and 3,667 summary projections. It also passed after moving summary searches
out of the canonical parser and marking those projections as derived. The original
endpoint golden remains unchanged. This proves bounded reader/source checks, not
complete field-for-field agreement for every unknown layout.

Reference projections, event extents, packet validation/coverage, and default-state
ownership are implemented and validated. Summary record boundaries remain
unknown rather than inferred.

## Phase 1/2 fidelity evidence matrix

| Requirement | Implementation and evidence | Limit |
| --- | --- | --- |
| Preserve supplied chunks and order | Registry-first dispatch; ordered `FilmDataChunk`; transport/source equality tests and 32-film checks | Unsupported versions are errors; unknown later kinds retain bytes |
| Registry mirrors recorded slots | Archetypes own ordered component names, precision, block and padding ranges; independently authored registry boundary cases | Bootstrap player searches belong to resolution |
| Preserve packet envelopes | `Packet<T>` retains all four recorded header fields and ranges; payload validation rejects inconsistent public envelopes | Public serde is not a complete nested-model validator |
| Preserve known hierarchy and fields | One sequential message/entity/control traversal; owned default states/components; separated datum headers and masks | Unsupported layouts stop rather than guess |
| Preserve exact source-backed values | `RawBits` comparisons across corpus components/defaults; independent datum-mask bit-order test | This is not a full reference oracle for every field |
| Distinguish inability and absence | Explicit component/event/list stops; unavailable event attempts; negative runtime-gate and truncation tests | Missing game runtime state cannot be reconstructed as recorded data |
| Preserve all payload bits | Original bytes plus source-coverage partitions for every captured packet; opaque/unparsed gaps stay distinct from padding | Coverage classification does not establish unknown grammar |
| Keep interpretation out of Film | Summary candidate searches and associations live in resolution with derived provenance; callback traces stay private | Summary sequential record grammar remains unknown |
| Preserve resolvability during migration | Independent resolved identity/lifetime/query/seek tests; default fields combined in source order | Future `TheaterRuntime` and private resolved storage are deferred |
| Reference comparison stays independent | Pinned Go fixtures, original endpoint golden unchanged, supplied reference contexts and source-verified runtime refusal | Padded Go and bounded Rust cumulative contexts intentionally differ |

All known-source fidelity claims are scoped to structural decoding and preservation.
They do not promise byte-for-byte re-encoding, every unknown field's semantics, or
independently annotated physical-action accuracy. Those are separate undertakings.
