# Native reader profile and observer contract

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
This is a parity audit of the existing parser, not the queued Film architecture.

## Current contextual position-width behavior

Direct components, live frame records, contextual inference, chain trials,
repair, validated/raw resync and production views obtain consumed position
widths from the raw native profile. Unused zero/oversized position dimensions do
not reject these calls. NativeWidthRefusal diagnostics preserve reached-field
address-domain limits through failed records, inference output and speculative
results. Legacy encoding conversions remain explicitly checked. Signed component
continuation and generic signed header/frame entry now pass their native oracles.
Higher-level recovery projections and untested callback/capture mutations remain
open. See SIGNED_CURSOR_MIGRATION.md.
See [context-lazy-widths-v41.md](../fixtures/context-lazy-widths-v41.md) for native
comparisons and actual wasm32 validation. The eager-position-adapter statements
in older sections below are superseded by this section.

## Live profile and observer replacement

NativeFilmReader.profile copies the current profile with native calibration-map
aliasing. replace_profile and replace_observer independently replace their half
of the context and return the previous value/receiver. Neither operation resets
the bit cursor, capture slot or attached position accumulator. None detaches
observation without suppressing the underlying decode or world updates.

The existing reader-sequence oracle now runs through one live Rust reader as
well as the original per-attempt decoder. All 1,064 cases / 6,384 original reads
and 6,348 ordered callbacks are compared across 72 component names. The test
checks both receiver streams, previous-profile values, shared receiver identity,
raw decoded fields, rejection status, neutralization and restoration. The native
harness adds a detached-observer read to each case: another 1,064 reads, including
19 rejections, with no callback delivery. Every earlier fixture field was checked
unchanged before saving the extension. The final targeted test passed (18.16s).

The three existing position sequence tests also pass (0.73s). They alternate
whole-context replacement with the independent setters, preserving cursor,
capture slot and accumulator identity, and comparing all native world positions
and callbacks after every read. These fixtures cover 2,048 sequences / 32,768
position reads. No position expectations were regenerated.

This closes these live replacement operations and tested state-preservation
contracts. It does not close every callback-time mutation path, unsupported
signed live cursor or lazy width conversion. The older per-attempt-only API
descriptions below are historical; they do not describe the complete current API.

## Stateful context reader implementation

`NativeFilmContext.reader` now copies the scan profile and shares its lazily
created `NativeFilmObserver`. `scan_frame` copies the profile without inheriting
that observer. `NativeFilmReader` retains a padded bit cursor and capture slot,
and `replace_context` replaces profile and observer together. Hooks can be
replaced or cleared on a shared observer; each publication resolves its current
hook without holding the observer lock during callbacks. Histogram snapshots
clear only the absolute-index histogram.

Direct-reader callbacks are now delivered at component publication sites while
retaining their ordered diagnostic values. Every publication resolves the live
shared hook, so a mobility callback can enable or disable later movement and
reference callbacks in the same component. Absolute-index counts update at the
wire read, before position publication; clearing the histogram in a callback is
not undone by a later diagnostics merge. The native 256-case live-observer oracle
verifies these transitions, including 83 enabled reference callbacks, disabled
controls, and 143 position callbacks that see and clear nonempty histograms.
This does not exhaust all 30 hook families under mutation or frame-level counters.
NativeFilmReader.read_frame_records now connects the generic record loop to
frame settings, shared hooks, calibration maps and FilmWorld creation/deletion
commits. World position accumulation is not automatically attached, matching the
native generic loop. Inference/view-class/resync entry points remain separate.
Profile adaptation currently validates address-sized component widths before
dispatch; irrelevant oversized axis metadata can therefore refuse a read on WASM.
Mobility-extra metadata now retains i64 in both NativeScanProfile and the component
encoding. The actual mobility branch applies the policy: full-body, inactive,
unrelated-component and nonpositive settings never execute the skip. Active
positive skips use wrapping signed cursor arithmetic. Large skips beyond source
retain all actual source fields and compact their zero tail, tagged with
NativeWidthPurpose::MobilityExtra rather than a stub override. The original
128-case policy oracle and a 208-case large-width oracle check callbacks, cursor,
raw metadata and source preservation. All 208 outcomes now match, including four
signed-overflow negative endpoints. Static ComponentWidthOverrides route
calibrated/stub skips through the same wrapping signed arithmetic and
source-preserving zero-padding compaction as live profiles.
The 18-case native large-width oracle is exercised through both dispatch paths;
ordinary widths keep their scalar fields. Static widths outside signed i64 are
refused rather than interpreted as native negative widths. Other
input-dependent unused fields are not yet fully lazy.
Native delta-axis override zero now resolves to the traversal descriptor's
corresponding axis, matching deltaAxisW; the raw profile retains zero. The
3,528-case native regression covers nonuniform fallback widths, positive override
precedence, zero and 65–129-bit fields, short inputs, callbacks, world accumulation
and subsequent reads. A further 720 absolute cases check exact NaN/infinity bits.
Native padded scalar reads consume their full width and return the low 64 bits;
additional discarded-prefix fields retain all actual source bits. Bounded reads
still refuse widths above 64. Both live and deferred position capture match the
native callbacks. See absolute-width-boundaries-v41.md in fixtures for the
primary-field/prefix overlap contract and independently checked source retention.
See [delta-width-fallback-v41.md](../fixtures/delta-width-fallback-v41.md).
These remaining address/conversion limits remain explicit parity gaps. The historical per-attempt API description
below still applies to existing call sites; it is no longer an inventory of all
public reader APIs.

The new context-factory sequence oracle is documented in
[context-reader-sequence-v41.md](../fixtures/context-reader-sequence-v41.md).

## Optional direct-reader position accumulation

`NativeFilmReader.replace_position_accumulator` attaches/detaches a borrowed
`FilmWorld` for direct component reads. Absolute paths seed bound slots, deltas
add float32 values only to existing seeds, and baseline paths re-emit the saved
position. Unbound slots are not created. With no accumulator, deltas are relative
and baseline paths are silent. Removing a position hook suppresses callback
delivery without suppressing world updates. `replace_context` preserves the
accumulator, current slot and cursor. `set_bit_position` accepts an address-sized
position; `set_native_bit_position` accepts a signed native position. Both change
only the cursor, without undoing earlier observations or world mutations.

Evidence uses unchanged native oracles: position-hook-sequence (1,024 cases),
calibrated-position-sequence (512), and baseline-scope-sequence (512). The live
public reader now runs beside the existing internal reader for all 32,768 reads,
checking status, boundaries, callbacks and every world's float32 position bits.
The oracle includes all five position publication kinds and profile replacement.
The position-capture oracle adds 1,024 cases / 12,288 reads with explicit bind,
unbind and seed changes, checking candidates with no observer, world state and
detachment. Native expected values were not regenerated from Rust.

This is direct component-reader parity, not automatic replay reconstruction.
`read_frame_records` now uses a separately attached accumulator while committing
entity lifetimes to its traversal world. `read_frame_records_accumulating` instead
uses the supplied world for both purposes, passing a single mutable borrow between
component capture and record commits. It rejects an independently attached
accumulator before changing context/cursor/world, avoiding conflicting targets.
Shared-world accumulation applies for that call; the caller retains the updated
world for subsequent calls. Neither API enables accumulation by default.

The two actual-native frame oracles each cover 512 cases, with accumulation on
and off, ordered callbacks, frame record IDs/end bits, final cursor and complete
traversal/accumulator world snapshots. The separate-world oracle includes missing
slot cases. The shared-world harness seeds slots 0/50/51 before traversal, so those
missing-slot cases are intentionally no longer missing in that fixture. Further
cross-call lifecycle/accumulator coverage and inference/recovery accumulator
scope remain separate gates. Negative native bit positions are retained by the
signed cursor API. The next-phase architecture remains deferred.

## Ownership mapping

Native `Lecteur` owns a source bit cursor, a profile value, position-capture state
and an observer pointer. `PoserProfil` and `PoserObservation` return the previous
value/pointer; `PoserContexte` installs both. Temporary neutralization mutates the
observer's position, unit-reference and movement hooks, then restores them.
Other callbacks and inference counters remain live. In particular, the historical
MobilityActionHook is not disabled with movement-state capture.

Rust component attempts receive an immutable `PositionEncoding` and a
`NativeComponentCapture` value. They return a `DecodedComponent` containing raw
fields, source ranges, raw references and ordered callback observations. A caller
selects the context for its next attempt rather than swapping pointers in a shared
reader. Observer receiver identity and the native mutable callback API are not
exposed as Rust APIs. This ownership distinction must not be confused with
missing recorded data or claimed API-for-API compatibility.

- `movement_slot: None` suppresses movement publications before they are emitted.
- `unit_references: false` removes unit-reference callbacks from diagnostics,
  while the decoded raw reference fields remain available.
- Position capture has its own explicit context and suppression policies.
- `mobility_actions` remains independent of movement-state suppression.
- Native attempts return `(status, component)` even for unsupported bodies.
  Their callback observations and already-read fields survive failure.
- The convenience `decode_component`/`decode_component_with_encoding` APIs return
  `ComponentDecode::Unsupported` or `Truncated` without the partial component.
  They are lossy success-oriented wrappers; native all-data consumers use the
  attempt APIs. They are not proof of full Film-level fidelity.

## New sequence oracle

`halo_rust_reader_sequence_test.go.txt` executes the native mutable reader with
256 independently generated inputs and six operations per input:

1. Read with profile/observer A.
2. Read after neutralizing A's captures.
3. Swap to profile/observer B and read.
4. Restore the previous profile/observer and read while A remains neutralized.
5. Restore A's captures and read again.
6. Install B through `PoserContexte` and read.

The oracle snapshots both receivers after each read. Rust compares consumed bit
positions, success/rejection, every ordered callback and mobility action, and
inactive-receiver emptiness. It also checks that restored contexts reproduce
their earlier result, suppression does not change raw fields or bit consumption,
and component serialization preserves the result. There are 34 rejected reads
with 34 retained callbacks, making failed-read retention a positive assertion.

Scope is four component families: non-predicted ability, mobility action, crouch
and unit equipment. Profiles vary body policies, mobility trailing width and full
precision. This does not exhaust all profile fields, observer aliases, capture
state or all callback families. Existing hook/read/inference/frame fixtures cover
other contexts separately; their presence is not a substitute for auditing each
remaining caller's retention and suppression behavior. The 1,536 native reads
and Clippy check passed; no production parser changes were needed.

## Expanded component-family sequence coverage

The native sequence oracle now appends 256 cases to the original 256, preserving
all original rows exactly. Added families are biped-control-context-component,
biped-emp-timer-component, managed-object-property-component and
object-multiplayer-properties-component. The total is 512 cases / 3072 reads /
3408 ordered publications; 34 rejected reads retain 34 callbacks. The six-stage
swap/suppression/restoration protocol and both receiver snapshots are unchanged.
These cases expand callback routing and profile isolation evidence; they do not
create a shared mutable Rust observer API or establish every capture-state alias.
The first generation used an unregistered shorthand for the multiplayer field;
corrected it to the canonical component name before Rust validation. No unknown
component cases are represented as successful multiplayer-property coverage.

## Full direct-hook name sweep

The sequence oracle additionally appends eight cases per name from all eight
shared native hook-name groups (scalar, ability, object, managed, engine, player,
probe and equipment). The earlier 512 rows are preserved exactly. Total coverage
is 1064 cases / 6384 reads / 72 distinct component names / 6348 ordered
publications, spanning 25 typed callback variants plus mobility actions. There
are 38 rejected reads with 38 retained callbacks. Nonpublishing component names
remain in the sweep as negative callback controls.

This covers the direct dispatcher under the existing six-stage profile/observer
protocol. It does not claim MPP/default-creation hooks, position-accumulator state,
record-mask scan hooks, arbitrary shared observer aliases or a unified mutable
Rust reader API; those use different call paths and remain separately audited.

## Position quantum replacement with live capture state

The position-hook sequence oracle appends 512 cases to its original 512,
preserving every old row. New sequences alternate doubled/native delta quantum
through PoserProfil before each of 16 component reads. The native harness checks
that the entire private capture state is unchanged by each profile installation.
Rust updates the corresponding capture quantum while keeping the existing world,
slot and emitter context, then compares consumed bits, ordered observations and
all accumulated world positions after every read. 256 new sequences accumulate
positions; 256 observe without accumulation. New emitted position observations
include 20 DeltaAxis, 21 Delta8, 13 Absolute, 9 AbsoluteFallback and 232 Baseline.

This is 1024 sequences / 16384 reads across the existing six position-bearing
component families. It verifies this concrete profile change with live capture
state, not arbitrary profile changes or observer pointer aliases.


## Generic frame entry point

`NativeFilmReader.read_frame_records` installs `NativeFrameConfig.context`, skips
a positive preamble only at bit zero, and uses the existing native generic record
loop. Clean New/Delete records update the supplied FilmWorld immediately; failed
creations do not bind. Existing world slots, positions, generations and views are
preserved unless changed by those records. The reader keeps its end cursor and
last record capture slot, including after failure. End and failed attempts remain
in the returned DecodedEntityView.

Component calibration/stub maps remain shared and are consulted at each relevant
dispatch point, including after callbacks mutate them. Validated scalar settings
supply ID/MPP/default-state widths, corruption checks, movement encoding,
new-record tails and the simulation-completion gate. Unsupported scalar frame
widths return a typed NativeReaderProfileError. Invalid live-map widths refuse
the component attempt; their failure classification is not yet distinguished
from truncation. Arbitrary negative/oversized native metadata execution remains
outside this adapter's supported range.

The 128-case native oracle compares 584 records and 888 callbacks, live width
mutation, preamble/reused-reader semantics, generation rejection and final world
state. See [context-frame-v41.md](../fixtures/context-frame-v41.md). This does not
close class/inference/resync observer installation or constructor-wide Film
integration. Generic DecodeFrameRecords does not invoke RecordMaskHook; that
hook belongs to the biped scan publisher, so it is not fabricated here.


## Configured production view classes

`NativeFrameConfig.decode_production_views` connects the shared context to the
existing message/entity/control parser with view-table admission enabled. The
message start follows `packet_preamble_bits - 1` when the supplied lead equals a
positive configured preamble; other lead positions start directly at entities.
This includes empty/truncated sources and skips beyond their end. Live hooks,
calibration maps and the simulation-completion gate flow into record attempts.
Admission counters update the shared observer at the native decision point, while
per-pass diagnostics remain retained independently. Generic record and view-class
paths intentionally keep their different capture-slot and generation policies.

The native 128-context/two-pass oracle validates 906 records, 1,332 callbacks,
48 cumulative unknown-slot and 50 foreign-view rejections, final worlds, view
counts and cursors. See [context-views-v41.md](../fixtures/context-views-v41.md).
Disabled view classes/tables are explicitly refused by this entry point. The
alternative inference/resync context paths, positive anticipated-counter callback
timing, biped RecordMaskHook publication and whole-Film integration remain open.
The preexisting decode_production_frame APIs preserve their historical policy.


## Single-step inference scopes

NativeFilmObserver now exposes scoped capture neutralization with restoration on
drop or explicit restore. The three native scopes are available: all captures,
position plus movement, and movement alone. They remove actual shared hooks,
leaving mobility/other hooks and counters active. Callback changes inside a scope
are possible; scope exit restores the saved callback identities, including nil.

NativeFrameConfig.decode_inference_view routes single-step trials and successor
confirmation through live readers, shared maps/counters and native capture scopes.
It honors view-table admission and preserves the single-step rule that inferred
records do not create soft bindings. Diagnostic candidates remain retained while
actual installed hooks determine delivery. The no-context APIs keep their existing
filtered diagnostics. Native evidence covers 128 cases/61 inferences/1,217 actual
callbacks and post-decode restoration; nested/unwind behavior has a Rust regression.
See [context-inference-v41.md](../fixtures/context-inference-v41.md).

The live entry point refuses recursive chain inference until that separate
observer/counter path is connected. Resync's shallow observer-copy/map ownership,
biped RecordMaskHook and whole-Film integration remain open.


## Recursive chain context

The configured inference entry point now honors chain_inference. Recursive body
trials and NEW traversals carry the live reader context. The native chain scope
neutralizes position/movement while keeping unit-reference hooks active; it is
not the single-step all-captures scope. Chain outcomes increment the shared
observer once the resolution is decided, before later publications or scope
restoration. Shared alignment does not imply a unique archetype or a soft binding.
The old no-context inference and explicit repair/resync APIs retain their prior
behavior.

The live chain oracle compares 128 native cases, 107 inferences, 1,129 callbacks,
and counter visibility inside EMP callbacks: Immediate=81, Deep=26,
NoConfirmation=13. Positive live Ambiguous/BudgetExhausted and recursive NEW
callback cases remain uncovered by this fixture. Existing non-live tests remain
complementary evidence. See [context-chain-v41.md](../fixtures/context-chain-v41.md).
The earlier refusal of chain mode described above is superseded. Live repair and
resync ownership, biped RecordMaskHook and whole-Film integration remain open.

## Live validated resync

`NativeFrameConfig::validated_resync` now carries its reader context through all
candidate DELTAs and recursive confirmation trials. Its observer is shared;
position, reference and movement receivers are scoped off and restored, while
other callbacks may re-enable them during the scan. Candidates set their entity
slot; fresh recursive body readers start at zero. A successful search increments
the shared validated-resync count exactly once, and returns the landing for a
caller to re-read. It neither mutates World nor enables automatic resync.

The actual native context-resync oracle compares two scans for each of 128
contexts, including callback order, count timing and restoration. Existing native
resync fixtures also run through contextual search, testing repeated counts and
capture suppression. Raw resync's shallow observer copy remains a separate open
integration issue, as do live explicit repair and the other gates above.

## Live explicit component repair

`NativeFrameConfig::repair_component` now installs the live context in width
trials, recursive confirmations and the selected-width re-read. A present caller
stub, even zero, refuses repair. Each repair allocates a local empty stub map,
leaving the caller map intact and retaining shared calibration maps. Candidate
reads consult the local stub at dispatch; the tested entry is removed before
confirmation and after the selected-width re-read. The observer is shared with
scoped capture neutralization/restoration. New trial readers start at slot zero;
Delta readers set their entity slot. Success counts one repaired record and each
matching width; unsuccessful resolution records its chain outcome. The supplied
World remains unchanged and default frame traversal does not enable repair.

The actual native context-repair oracle covers 64 contexts with two calls each,
ordered callbacks/counter snapshots, preset refusal and caller map preservation.
Other remaining fidelity/integration gates are unchanged.

## Native shallow observer copy

`NativeFilmObserver::shallow_copy` reproduces Go observer assignment: callback
identities and scalar counters are copied, while already allocated component
width, anticipated-binding and absolute-index maps remain shared. Nil maps are
allocated independently on first write. `take_absolute_indices` replaces the
caller's map with a fresh allocated-empty map, preserving the old map in any
shallow copies. Ordinary Clone still aliases the full observer. Owned diagnostic
snapshots preserve the existing public counter representation.

The 16-case native observer-copy fixture validates map allocation/replacement,
scalar independence and copied callback identity. Raw-resync live scan integration
and its local position collector remain the next step; the primitive alone is
not evidence that the full raw-resync context path is complete.

## Configured raw target scan

`NativeFrameConfig::scan_for_target_delta` now installs the shallow observer copy
and a temporary first-position hook, carrying context through every candidate
Delta. Acceptance receives the slot, vector and position-presence flag; a false
flag can accompany a stale vector from a previous trial, matching native output.
Caller hook changes made by shared closures remain on the caller, while the copy
continues using its copied hook identities. Existing allocated maps are shared;
new maps created during scanning stay local. The World is not mutated and there
is no recursive successor confirmation. All read diagnostics remain available
independently of delivered callbacks.

The actual native context-raw-resync oracle covers 128 contexts and two scans
each. Whole DecodeFrameResync live context integration, suppression around scans
and accepted-record re-reading remain separate work.

## Configured full raw resync loop

`NativeFrameConfig::decode_resync_frame` starts at zero without a preamble and
carries live context through sequential reads. Clean New/Delete records update
World. Recovery removes position/movement hooks temporarily, performs the copied-
observer raw scan, restores callbacks before re-reading the accepted record, and
resets the next sequential reader's capture slot to zero. Delta reads set their
entity slot; New reads retain the sequential reader's current slot. The native
4096-iteration guard remains. Failed recovery returns the successfully collected
records and a typed stop. The accepted record is re-read even if callback mutation
makes it fail, matching the native control flow.

NativeResyncFrame preserves all read diagnostics independently of actual callback
delivery, plus records and resync source offsets. Live cumulative counters remain
on the caller observer. The legacy no-context API is unchanged. The native
128-case context-resync-frame fixture verifies returned records, callbacks,
acceptance and world bindings; its explicit remaining gaps are listed alongside
the fixture. It does not establish whole-Film constructor integration.

## Live biped RecordMaskHook

The observed payload scanner and observed explicit-band quantized/world source
scanners now dispatch RecordMaskHook at the native per-record point: after
companion decoding, only when CaptureDirs is enabled, and after saturation
rejection. Source isolation and speed filters run later, so callback publication
is not restricted to the final accepted positions. The hook borrows the complete
packet payload, ordered component indices and the actual after-i0 bit offset.
Current receiver lookup occurs for each publication. Generic frame readers do
not emit it. Existing no-observer APIs and retained mask reports remain available.

The existing pinned native record-mask fixture now verifies actual Rust callback
arguments for payload and source scans, including duplicate/readable-prefix
selection and temporal filters; see live-record-mask-v41.md for counts. This does
not assert whole-Film constructor/context integration or semantic-event coverage.

## Signed live width overrides and padded-tail retention

Component traversal now retains signed i64 coordinates and uses native wrapping
Skip semantics for calibrated/stub overrides, New default/tail skips and mobility
padding. Negative intermediate positions may continue through further skips;
real reads use native signed-reader behavior. All 24 single-component signed
width cases and the 96-case continuation gate match native outcomes.

Source addresses remain checked separately. NativeWidthAdjustment.end_bit is an
optional usize projection; native_end_bit returns the wrapping signed endpoint.
Large positive skips preserve available source fields and compact synthetic
padding. Derived padding counts saturate on narrow targets while native endpoints
remain exact. Native trace positions must not be used as unchecked slice offsets.

RecordHeader and generic frame entries now preserve signed positions and have
post-panic cursor evidence. Higher-level inference/resync source-address projections
and raw MPP width limits remain open. See SIGNED_CURSOR_MIGRATION.md.

## Positive live chain-budget evidence

The live-chain-budget fixture now exercises the native 200,000-trial limit
without lowering it: eight synthetic streams with calibrated successor widths
produce three BudgetExhausted and five NoConfirmation outcomes. The Rust public
inference-view API matches all live observer counters, stopping offsets, failed
Delta headers, frame diagnostics and JSON. No inferred bindings are fabricated.
The focused comparison passed in 3.32s. These cases do not establish every
callback mutation around budget exhaustion or complete recovery scope parity.

## Stateful direct-reader signed cursor integration

NativeFilmReader now owns NativeFilmBits. Its direct scalar, variable-width,
set/skip and remaining operations preserve the pinned signed 64-bit cursor,
including wrap and post-panic endpoints. NativeFilmBits::read_wide also removes
the prior u32 width boundary. The address-sized bit_position projection is
checked; native_bit_position is the lossless signed accessor. Direct component
entry points now retain signed starts and update the live reader cursor even on
host unwind. The 80-case signed-component-cursor fixture checks 28 panic and 52
nonpanicking cases, including reset/read recovery. WASM checks the 52 nonpanicking
cases because that target aborts on panic. Whole-frame entry/header address limits
and whole-frame post-panic copyback remain separate gaps.
See ../fixtures/reader-cursor-v41.md and
../fixtures/signed-component-cursor-v41.md for exact oracle scope.

## Raw resync recovery boundary coverage

The failed accepted-candidate re-read and positive 4096-iteration guard are now
checked separately by resync-mutation-v41.md and resync-guard-v41.md in fixtures.
Sixteen callback cases and 24,574 returned-record comparisons run on host and
WASM. Shared width-map deletion is exposed publicly without changing map identity.
This narrows the context-resync-frame fixture's listed gaps; native final-loop
cursor and arbitrary callback mutation are not claimed verified.

## Native final resync cursor verified

The final-loop cursor warning above is superseded for the 70 cases in
../fixtures/resync-cursor-v41.md. Hash-guarded native AST instrumentation preserves
all original outputs and exposes the cursor. It found and fixed missing baseline
selector consumption on an unbound Delta in native contextual lookup. Both
selector forms now match on host and WASM. Strict-generation readers are
unchanged; broader signed-address and callback contracts remain separate.

## NativeFilmReader frame panic copyback

Frame traversal now mirrors component cursor updates and the capture slot into
its stateful caller on unwind. Native skips of negative optional record prefixes
advance without source reads. The 80-case frame-panic-cursor oracle compares 72
panic outcomes and eight successful controls, checking final cursor/slot, EMP
callback order, prior deletion and identity retention, and reset/read recovery.
See ../fixtures/frame-panic-cursor-v41.md and frame-panic-cursor-validation.json.

This supersedes earlier statements that NativeFilmReader frame copyback is
unimplemented. It does not establish arbitrary hook/capture mutation parity,
signed/header domain completeness, or the separate NativeFilmBits adapter's
post-panic cursor behavior. The architectural phase remains deferred.

## Source-only frame adapter panic copyback

The separate decode_native_frame_records adapter now mirrors component cursor
updates through unwind and retains earlier binding changes. Its acceptance test
compares 24 applicable cases from the native frame-panic oracle: 16 panics and
eight successful controls, including reset/read recovery. The adapter has no live
observer and its static override maps are unsigned; this is not evidence for
negative configured widths or arbitrary callback behavior. Signed starting
positions and header/MPP domains remain open. See source-frame-panic-validation.json.

## Signed native record headers and generic frame entry

RecordHeader.start_bit/end_bit now retain i64 coordinates. The public
`decode_native_record_header` uses the native signed source reader and native
positive ID widths independently of pointer size; nonpositive widths omit the
low-ID read. Both generic frame entries preserve signed starts, wrapped End
positions and post-panic header cursors. NativeFilmReader.padded_bits returns zero
for negative coordinates and saturates on narrow targets, while native_bit_position
remains exact. Positive header JSON values retain their representation.

This supersedes the generic-header address refusal documented in older sections.
411 native header cases pass, plus 361 End/panic cases through both generic frame
entries. WASM executes the 275/225 nonpanicking subsets. High-level inference,
resync, march and recovery source-address projections remain to audit; this does
not establish full signed traversal or MPP width parity. See
signed-header-validation.json and ../fixtures/signed-header-v41.md.

## Native MPP width domain and retained field widths

Native MPP lead/index profile values are cast to u64 exactly as native uint reads.
They now use the signed native cursor without a pointer-size or i64-end refusal.
ComponentField.width is u64 on host and WASM; positive existing JSON numbers are
unchanged. Cursor positions remain i64. Source indexing and bounded-reader width
admission remain checked separately. Prefix bits discarded by wide numeric reads
are retained only over available source; synthetic padding is not expanded.

The 272-case direct native oracle checks MPP panic/cursor/publication/recovery
outcomes, while two actual native 2^32-bit frames check the public reader and
retained fields on host/WASM. Ordinary-start u64::MAX controls are analytical and
are not reported as executed native evidence. Raw position/profile widths still
use address-sized adapters and require their own audit. See mpp-domain-validation.json.

## Native position profile widths

World/traversal indices and axes, delta axis overrides, non-predicted ability
anchors, and predicted handle-tail indices now pass their raw u64 widths directly
to the native reader. No implicit usize maximum is imposed on these reads.
Explicit caller limits still fail only when the configured field is reached.
PositionEncoding remains a legacy address-sized convenience adapter; contextual
NativeFilmReader methods supply native raw widths independently of that adapter.

The position-domain oracle covers widths 65 and 2^32 in 18 direct reads and 18
generic frames, including exact callback float bits and source prefix retention.
It does not prove arbitrary u64 inputs or higher-level recovery/inference parity.
See position-domain-validation.json for the current checkpoint.

## Signed inference starts

NativeFrameConfig::decode_inference_view and decode_inference_frame accept a
checked TryInto<i64> start. Starts at or beyond source length return an empty
PayloadBoundary result with the unchanged cursor. Negative starts enter native
traversal: an optional 32-bit prefix skip may bring the cursor into readable
range, and an actual invalid read retains native panic behavior. Padded-bit
accounting is source-sized and saturating; end_bit remains signed and lossless.
The API has no caller-owned inference cursor to copy back on panic. See
signed-inference-entry-validation.json; signed chain/recovery support is separate.

## Signed inference results and confirmation

ChainInference.end_bit and InferenceEvidence::SingleStep.end_bit are i64.
Candidate endpoints are ordered and deduplicated as signed native integers.
The source-length upper bound on chain body trials remains; the incorrect lower
bound at zero is removed. Successor/prefix and terminal-tail reads use the native
signed reader, including native panics and wrapping arithmetic.

NativeFrameConfig::infer_unbound and ::infer_chain expose contextual inference
without mutating the supplied world. Results include speculative read diagnostics;
these methods retain the native scoped capture behavior. Negative results are
preserved in JSON and in full inference-frame continuation. See
signed-inference-trial-validation.json. This is not a claim that all resync and
harvest source-address APIs support the signed domain yet.

## Signed raw-resync scanning

Both contextual and standalone raw scanners accept signed starts and return i64
landings. Loop bounds and remaining-bit checks use native wrapping arithmetic;
invalid source reads retain native panic behavior. A successful acceptance at
landing -32 is distinct from the normal no-match result.

The sequential raw-resync caller applies its own next < 0 rejection after scan
callbacks. Its resync_bits retain nonnegative i64 positions, independently of pointer width. A
negative accepted scan does not become a sequentially recovered record. See
signed-resync-validation.json. This does not close harvesting or view-orchestration
source-address contracts.

## Signed and contextual target harvesting

NativeFrameConfig::confirm_harvest_successor and ::harvest_targets now preserve
signed successor positions and contextual width/observer behavior. The standalone
harvest path uses the same signed scan and confirmation logic. Prefix arithmetic
and remaining-bit checks wrap in the native signed domain; a successful successor
may end before bit zero. Trials and confirmation suppress position callbacks;
accepted rereads restore them. Callback mutations to shared widths affect the
reread, which is retained even when partial, and determine the next scan position.

Independent native fixtures cover 224 successor cases (60 panics, two negative
confirmations) and 32 contextual harvest cases. All focused host comparisons pass;
actual WASM passes 164 nonpanicking successors plus all contextual cases. Clippy,
format, diff checks and all 485 pinned source hashes pass. Full host/captured-film
validation is tracked in signed-harvest-validation.json; consult its status before
claiming this checkpoint finalized. Signed view orchestration, broader repair and
runtime contracts, and all-data corpus acceptance remain open. NEXT_PHASE.md stays
deferred.

Final signed/contextual harvest validation: 609 host tests passed, zero failures,
49 ignored (105.17s); Clippy passed (10.21s); actual WASM passed (10.27s build).
Four captured march films match (81.45s release), and six complete film documents
match (104.81s release). Both harvest fixtures regenerate byte-identically;
format/diff checks and 485 pinned hashes pass. See signed-harvest-validation.json.
This completes the harvest checkpoint, not full v41 parity. Signed view
orchestration and broader contracts/corpus acceptance remain open; NEXT_PHASE.md
remains deferred.

## Signed view orchestration

InferenceViewsOptions now retains native i64 view counts, lead skips and packet
preambles. Standalone and contextual inference orchestration retain signed cursors
between views; NativeFrameConfig::decode_inference_views exposes the configured
class/generic traversal. Production entry points accept checked signed starts and
preserve signed prefix/header/component continuation, including negative ends.

A prefix is skipped by native traversal. Its retained raw field therefore uses
available source bits without introducing an extra native read/panic. A completed
delta can end at -32, skip its next prefix to zero, reach End and continue into
control or subsequent generic views. Message/control guards use the native
wrapping position+width check; signed public reader variants retain the full int64
domain and native panic behavior. In particular an i64::MAX start can read a padded
terminator and finish at i64::MIN. This does not imply source bytes exist there.

Native fixtures cover 792 orchestration entries (94 panics), 120 direct readers
(48 panics), and 16 complete component-to-view continuations with calibrated/stub
widths including -83 and 2^32. Host comparisons, Clippy and actual WASM pass;
three oracle regenerations are byte-identical and 485 pinned hashes match.
Full host/captured validation is tracked in signed-views-validation.json.
March/keyframe research traversal projections and broader runtime/corpus
acceptance remain open. Full v41 parity is incomplete; NEXT_PHASE.md is deferred.

Final signed-view validation: 612 host tests pass, zero failures, 49 ignored
(109.25s); Clippy passes (17.94s); actual WASM passes (22.02s build), including
324 class-mode production entries in addition to both inference APIs and the
direct/continuation cases. Four captured march films pass (83.15s release), and
six complete documents match (105.94s release). Format/diff checks pass. See
signed-views-validation.json. This closes the signed-view checkpoint; the march,
queue diagnostics, remaining source/runtime contracts and full corpus acceptance
are still open. The architecture proposal/refactor remains deferred.

## Signed march and queue wrappers

March entry points now accept checked i64 starts and preserve signed remaining
arithmetic, headers, baseline reads and record continuation. The packet preamble
is applied at view entry, not whenever a record rewinds to zero.
NativeFrameConfig::march_records uses the shared contextual generic reader for
eight views, snapshots the supplied world slots, and restores them on normal
return. Native panics retain preceding mutations, as marchRecordsOf does.

KeyframeQueueVariant uses i64 ID widths and preambles. Queue measurements defer
ID width use until the native header consumes it, preserve negative preambles in
the reported variant, and apply a preamble only when positive. Signed endpoints
remain intact through headers, baseline reads and bodies.

Independent fixtures cover 84 march entries (14 panics), 210 End-only queue
measurements (42 panics), ten complete march continuations and four rollback
cases (two panics). The continuation covers end -32, end 0 followed by Delete,
and end beyond 2^32. The rollback fixture confirms that a preceding Delete is
restored normally but retained after a later native panic. All focused host
comparisons and Clippy pass. Full checks are tracked in
signed-wrappers-validation.json.

The legacy source-only ComponentWidthOverrides map is still unsigned and
pointer-sized; only its represented domain is compared for continuation. The
contextual march API covers every signed fixture row. Unused-ID tests establish
lazy admission, not arbitrary consumed-ID behavior. Native keyframe boundaries,
other nested/runtime contracts and full corpus acceptance remain open. The
architecture phase remains deferred.

Final signed-wrapper validation: 616 host tests pass, zero failures, 49 ignored
(110.05s); Clippy passes (17.78s); actual WASM passes (22.74s build). Four captured
march films pass (82.24s release), and six complete documents match (105.24s
release). All four native fixtures regenerate byte-identically; 485 source hashes,
format and diff checks pass. See signed-wrappers-validation.json. This completes
the march/queue checkpoint, not full v41 parity. The signed keyframe-chain
boundary mismatch and broader native table/layout/runtime/corpus gates remain
open; NEXT_PHASE.md remains deferred.

## Contextual keyframe records, chains and tables

Native keyframe-chain guards now use signed wrapping arithmetic and lazy native
header reads. Table traversal preserves signed record ends and its distinct
sentinel fallback: a negative header is a Header stop in a chain, but the table
then attempts a sentinel read and can panic. Prefix/body retention does not
project endpoints to pointer-sized addresses.

NativeFrameConfig now exposes read_keyframe_record, chain_keyframes and
read_keyframe_table with the shared profile maps and observer hooks. Full-state
entry reads the archetype at record+58, then skips its header to the body; retained
header fields do not introduce reads that native never performs. Signed starts
therefore preserve prefix-repaired negative entries and native panic boundaries.
The source-only APIs retain the same traversal fixes.

Independent native fixtures cover 56 chain entries, 24 direct full-state entries
(eight panics), and 30 contextual record/chain/table calls (two panics). A live EMP
hook changes the following calibrated/stub width, including a rewind to -32 and
a skip beyond 2^32. Events and width mutations survive table fallback panics.
All focused host tests pass; full validation is tracked in
keyframe-context-validation.json. Three fixtures regenerate byte-identically;
485 pinned hashes match.

This does not close the raw keyframe layout domain: NativeScanProfile still uses
the bounded KeyframeLayout (header >=64, size word <=32). A separate native layout
fixture is needed to migrate signed header skips and lazily consumed word widths.
Other runtime/source/corpus gates remain open; NEXT_PHASE.md stays deferred.

Final contextual keyframe validation: 619 host tests pass, zero failures, 49 ignored
(103.31s); Clippy passes (29.25s including build-lock wait); actual WASM passes
(17.54s build). Four captured march films pass (79.55s release), and six complete
film documents match (102.44s release). Three oracle fixtures regenerate
byte-identically; format/diff checks and 485 pinned hashes pass. See
keyframe-context-validation.json. The signed entry/chain/table context checkpoint
is complete. Raw keyframe layouts, remaining runtime/source contracts and full
corpus acceptance are still open; NEXT_PHASE.md remains deferred.

## Raw signed keyframe layouts

NativeScanProfile now retains signed header and size-word widths through
NativeKeyframeLayout. Contextual consumers skip headers with native signed
arithmetic and consume words lazily, preserving low-32 signed guard semantics
and available discarded source bits. Bounded legacy APIs retain their separate
layout contract. Short-header retention does not introduce native reads.

The native oracle supplies 90 chain/table cases (ten panics), generic End-frame
controls for the same settings, and 194 consumed-word cases (32 panics, 36
positive EMP publications). This supersedes the earlier raw-layout migration
pending statement. Full validation is tracked in keyframe-layout-validation.json.
Positive huge word reads are not executed by this oracle; negative-position
panic cases and generic bit-reader evidence have distinct coverage. Other
runtime/source contracts and complete v41 acceptance remain open. NEXT_PHASE.md
remains deferred.

Final raw-keyframe-layout validation: 621 host tests pass, zero failures,
49 ignored (116.08s); Clippy passes (50.00s); actual WASM runtime passes
(53.29s build). Four captured march films pass (81.78s release), and all six
complete film documents match (104.73s release). Both included keyframe fixtures
match native generator output byte-for-byte; all 485 pinned hashes, formatting
and diff checks pass. See keyframe-layout-validation.json. Full v41 parity is
still incomplete; signed validated resynchronization is the next concrete gate.

## Signed validated resynchronization

Validated resynchronization now accepts checked signed offsets and returns i64
landings in both the contextual NativeFrameConfig API and source-only chain
scanner. Native wrapping availability checks precede the optional 32-bit skip;
negative source reads retain native panic behavior. The former usize conversion
and unsigned empty-buffer scan bound are removed. Unsupported host values beyond
i64 remain outside the native int64 contract.

The expanded native oracle has 126 cases: 44 panics and six successful landings,
including a returned -32. Tests compare both APIs, signed JSON round trips,
world preservation and capture-hook restoration. Previous statements that this
migration is pending are superseded; final validation is tracked in
signed-validated-resync-validation.json. Other native runtime/source contracts
and complete all-data v41 acceptance remain open; NEXT_PHASE.md is deferred.

Final signed validated-resynchronization validation: 622 host tests pass, zero
failures, 49 ignored (106.02s); three focused tests pass (0.64s); Clippy passes
(20.06s including lock wait), and actual WASM execution passes (18.87s build).
All four captured march films pass (86.06s release) and six full documents match
(113.69s release). The native fixture is byte-identical to generator output;
485 source hashes and formatting/diff checks pass. An initial release build ran
out of disk space during stripping; regenerable crate build output was cleaned,
and a successful rebuild preceded both passing release runs. See
signed-validated-resync-validation.json. Full v41 parity remains incomplete;
the native isolation-gap option is the next concrete correction.

## Native isolation-gap preservation

BipedScanOptions now retains optional native_isolation_gap_ms as i64. When set,
it overrides the convenience microsecond value. Nonpositive native values disable
isolation; positive values convert with wrapping u64 multiplication, preserving
an active zero threshold. Both world-position and quantized-source filtering use
the resulting Option threshold. Legacy serialized options without the new field
retain their behavior, and the raw native value survives serialization.

The 64-case oracle compares world-stream filtering and raw-option conversion;
WASM runs the option/JSON cases. Existing source scans exercise the shared filter
integration. Validation is tracked in isolation-gap-validation.json; earlier
pending implementation statements are superseded, but full parity remains open.

Final native isolation-gap validation: 624 host tests pass, zero failures,
49 ignored (104.55s); three focused tests pass (1.71s); Clippy passes (1m09s
including lock wait), and actual WASM runtime passes (18.18s build), including
64 raw-option conversion/JSON cases. Four captured march films pass (82.76s
release), and all six complete film documents match (109.93s release). Native
fixture bytes, 485 source hashes, formatting and diff checks pass. See
isolation-gap-validation.json. This closes the signed millisecond and active
wrapped-zero threshold correction. Full v41 parity remains incomplete, and the
architecture in NEXT_PHASE.md remains deferred.

## Ordered payload projection across signed rewinds

A native 32-case parent oracle reproduced a payload-selection error: two
components starting at bit 172 could have different precision parameters, but
Rust selected the first observation for both. Component attempts now retain
ordered observation ranges; full keyframe and biped-probe spans additionally
retain ordered field ranges. Payload projections use those ranges instead of
source-bit overlap. Four native round-timer cases exercise partially overlapping
field ranges and distinct recorded quanta. All original source fields remain
retained; the correction changes their association with the requested component.

Range metadata is optional only for older serialized records. Missing keyframe
field ranges or parent observation provenance returns no typed payload rather
than guessing from source position; generic non-observation payloads still use
their existing ordered field ranges. This is current parser fidelity work, not
the deferred architecture. See overlapping-payload-validation.json for validation;
full v41 parity remains incomplete.

Final overlapping-payload validation: 625 host tests pass, zero failures,
49 ignored (108.71s); six focused payload tests pass (0.30s); Clippy passes
(36.37s including lock wait), and actual WASM runtime passes (28.04s build),
including all 36 overlap cases. Four captured march films pass (109.91s release),
and all six complete film documents match (156.87s release). The native probe
fixture comparison validates added Rust ranges separately from all native
fields; that final change was test-only. Native fixture bytes, 485 source hashes,
formatting and diff checks pass. See overlapping-payload-validation.json. Full
v41 parity remains incomplete; component-mask wrapping beyond index 63 is the
next confirmed correction. NEXT_PHASE.md remains deferred.

## Native component-mask wrapping

Native generic records, full keyframes and inference-body trials now visit the
entire registry component list. Presence tests use index & 63, matching native
traverseComponentLoopFrom; full keyframes consume all entries. The legacy probe
already wrapped its indices. Ordered ranges and full component indices remain
retained rather than renumbering components above 63.

The 36-case native oracle covers lengths 0, 1, 63, 64, 65, 66, 127, 128 and 129,
with full keyframes and three generic dense masks. It now also records 27 direct
body trials and recursive inference outcomes. Rust compares selected indices,
source starts, retained byte values, callback order, endpoints and inference
results. See component-mask-wrap-validation.json for validation; the earlier
pending implementation statement is superseded, not the broader parity gates.

Final component-mask-wrap validation: 626 host tests pass, zero failures,
49 ignored (113.96s); the focused 36-case matrix and 27 inference comparisons
pass (0.03s). Clippy passes (16.75s); actual WASM passes (15.59s build), including
all frame/keyframe cases and public chain inference. Four captured march films
pass (84.03s release), and six complete documents match (110.20s release).
Expanded raw validation also passes: all 134,657 recovered keyframe anchors
across 32 films (65.88s), including 189,416 typed payloads, 43,631 death states,
468 padded endpoints, ordered fields and source-bit checks; all 451 sequential
keyframe walks pass (3.50s). Native fixture bytes, 485 pinned hashes, formatting
and diff checks pass. See component-mask-wrap-validation.json. This closes the
native component-list cap correction; full v41 parity is still incomplete and
NEXT_PHASE.md remains deferred.
