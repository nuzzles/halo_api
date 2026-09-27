# Signed traversal implementation and remaining gates

This is current v41 parity work, separate from the deferred architecture in
NEXT_PHASE.md. The 96-case signed-continuation gate now passes without an ignore.

Implemented:

- ComponentCursor carries signed i64 native positions; native reads use
  NativeFilmBits, while bounded reads retain checked source access.
- Calibrated/stub, New default/tail and mobility skips use wrapping addition,
  preserving negative intermediate/final positions and continuation without reads.
- Native component, record, view and observation trace coordinates use i64.
  Positive JSON numbers retain their representation; negative native positions
  are preserved as signed numbers rather than fabricated source offsets.
- Source indexing uses checked projections. NativeWidthAdjustment.end_bit remains
  an Option<usize> address projection; native_end_bit preserves wrapping i64.
- Large positive skips retain available source fields and compact synthetic tails.
  Derived padded-bit counts saturate at usize::MAX on narrow targets; native
  endpoints remain exact.
- Stateful direct components mirror their cursor through successful reads and
  host unwinding. Diagnostic prefix rereads do not overwrite the live cursor.

Evidence: 96 record continuation cases, 80 direct-component cursor cases (28
panics and 52 nonpanicking outcomes), and four public inference-frame cases.
The final host suite passed 596 tests, zero failed, 49 ignored (108.76s), and
Clippy and actual WASM execution passed. Four captured march comparisons and six
complete film document comparisons passed in release mode (84.28s and 108.00s).
See signed-cursor-migration-validation.json and signed-continuation-validation.json.

Remaining parity gates:

- Higher-level inference, march, resync and recovery paths still contain checked
  source-address projections and require their own signed-domain audit. Native
  RecordHeader now retains i64 positions; direct header decoding and both generic
  frame entries use the native signed reader, including panic cursor copyback.
- NativeFilmReader frame cursor and capture-slot copyback now match 80 native
  frame cases, including 72 panics from reads or hooks and prior deletions. The
  separate NativeFilmBits decode_native_frame_records adapter now passes 24
  applicable native cases, including 16 panics, cursor copyback and earlier binding
  changes. Arbitrary observer/capture mutations remain unverified.
- Raw position/profile widths and some recovery/inference source-address
  projections still restrict the native domain and need further differential cases.
  MPP reads now use u64 widths and native cursor wrapping/panic behavior.
- Full payload/callback comparison, remaining dynamic type contracts and captured
  all-data acceptance remain open. These tests do not prove complete v41 parity.

No byte-for-byte re-encoding or deferred Film/ResolvedFilm/playback implementation
is claimed by this correction.

The subsequent frame-recovery correction passed 597 host tests, zero failures,
49 ignored (88.11s), Clippy (18.00s), and actual WASM runtime checks. Its native
oracle covers 80 cases including 72 panic outcomes. See
frame-panic-cursor-validation.json. The captured comparisons above predate this
follow-up; they are not claimed as rerun evidence for it.

Source-only frame-adapter follow-up: 24 applicable native cases pass, including
16 panics and earlier binding changes. Final suite: 598 passed, zero failed,
49 ignored (86.35s). Clippy and actual WASM runtime pass; see
source-frame-panic-validation.json. Signed header entry and raw width/configuration
limits are still not established as complete.

Signed header correction: the 411-case native header oracle passes (136 panics,
275 successful headers). Both generic frame adapters also pass 361 applicable
End/panic cases; WASM passes 275 direct and 225 frame cases. RecordHeader and
creation-origin coordinates now use i64. One active 2^32-bit native ID read is
included; infeasible i64::MAX active reads are analytical controls, not executed
oracle evidence. See signed-header-validation.json. Higher-level recovery/source
address checks and raw MPP widths remain separate gates.

Signed-header checkpoint final validation: 600 host tests passed, zero failures,
49 ignored (105.04s); Clippy passed (10.09s); actual WASM passed. Four captured
march comparisons passed in 82.31s release, and six complete film documents in
104.44s release. This does not close the remaining MPP/source-recovery gates.

MPP width correction: ComponentField.width now retains u64 counts on every target.
Native MPP lead/index reads bypass pointer-size and signed-end admission checks;
explicit caller width limits remain separate. 272 direct native cases (212 panics)
and two actual 2^32-bit whole-frame native reads pass. Available discarded source
prefix bits remain retained, and synthetic padding remains compact. See
mpp-domain-validation.json for final validation and remaining raw position limits.

MPP checkpoint final validation: 602 host tests passed, zero failures, 49 ignored
(86.75s); Clippy passed (16.06s); actual WASM passed the two wide MPP frames.
Captured march (four films, 79.74s release) and complete documents (six films,
101.75s release) passed. Raw position/profile and higher-level recovery gates
remain open.

## Position-width follow-up

Raw world/traversal index and axis reads, delta widths, ability anchors, and
predicted handle tails now consume u64 widths without pointer-size admission.
The native oracle covers 18 configurations through both direct components and
generic frames. Host/WASM validation is tracked in position-domain-validation.json.
The legacy PositionEncoding adapter remains address-sized; raw contextual reads
bypass its widths. This supersedes the raw-position restriction described above
for these paths, without claiming arbitrary signed recovery support.

Next signed-domain audit (implementation remains separate):

- `NativeFrameConfig::decode_inference_view` and `decode_inference_frame_contextual`
  accept usize starts and reject starts beyond source length. Native decodeInferLoop
  accepts signed reader positions and skips its loop when already beyond frameLen.
- Inference header continuation projects `header_bit` into the legacy Cursor;
  `read_bound_record_contextual` also accepts a usize header position. Compare
  negative starts with/without the optional Skip(32), padded starts, and wrapping
  component skips before changing these signatures.
- Chain trials currently admit only endpoints between zero and the actual source
  length. Determine which bounds match native inference candidate acceptance;
  source-array bounds must not be removed merely because they use usize.
- Raw recovery `scanForTargetDelta` iterates from its signed `from` argument.
  Audit negative starts, failed rereads, callback mutation, and resulting cursor
  separately from the successful generic-frame tests.

Position-width checkpoint final captured validation: all four march films pass
(81.48s release), and all six complete film documents match (104.30s release).
See position-domain-validation.json. Full v41 parity remains incomplete.

## Signed inference entry correction

Inference entries now accept checked i64 coordinates, including starts beyond
the source. The inference loop uses a signed native header reader and wrapping
optional-prefix addition; bound-record rereads accept signed coordinates too.
90 independent native cases cover empty/short/zero payloads and prefix-adjusted
negative starts (21 native panics, 69 successes). Validation is tracked in
signed-inference-entry-validation.json. This supersedes the entry-point restriction
noted above; broader chain/recovery restrictions remain.

The next native probe, halo_rust_signed_inference_trial_test.go.txt, executes 72
calibrated/stub-width trials: 40 negative endpoints are accepted by deltaBodyTrial,
and 16 single-step inference attempts panic while reading their successors.
Native bounds reject only endpoints beyond frameLen, not negative endpoints.
Rust chain_delta_body_trial_contextual currently rejects negative endpoints and
returns usize. This is a reproduced remaining mismatch, not a passed parity gate.
Output: /private/tmp/halo-signed-inference-trial.json. The generator is retained;
the fixture is registered as signed-inference-trial-v41.json.zlib, but no Rust
test or chain-domain correction is claimed yet. Two native controls confirm an
archetype at endpoint -32: bodyStart 32, calibrated/stub width -74, with the
optional prefix reaching a clean bound successor at bit zero. This is a valid
negative inference result currently lost by Rust, in addition to panic mismatches.

Signed-inference-entry checkpoint final captured validation: four march films
pass (82.10s release), and six complete film documents match (105.57s release).
See signed-inference-entry-validation.json. The signed-trial oracle remains an
open correction; full v41 parity is not established.

## Signed trial and successor correction

The negative-trial mismatch above is now corrected for single-step and chain
inference. Native trial endpoints, chain candidate maps, confirmation recursion,
and InferenceEvidence/ChainInference endpoints retain i64. Successor headers and
terminal reads use the signed native reader; remaining-bit arithmetic and prefix
skips wrap as native int64. Chain repair candidate maps also preserve signed ends;
raw resync/harvest entry and repair-domain acceptance still need separate audits.

NativeFrameConfig now exposes contextual infer_unbound/infer_chain methods with
shared diagnostics. The 72-case native fixture compares direct trials, both
inference modes, and four complete inference frames. It includes 40 negative
trial endpoints, 16 single-step panics, 32 chain panics, and confirmed endpoint
-32 in each mode. The four frames retain the inferred endpoint, ordered successor
records, final cursor, bindings, and JSON. See signed-inference-trial-validation.json
for checkpoint status. Full parity and the deferred architecture are unchanged.

Next raw-resync gate: signed-resync-start-v41.json.zlib contains 36 native scan
cases, including 14 panics and a successful accepted landing at -32 (from -32,
optional prefix, callback receives slot 0 with no position). Rust's scanner still
accepts/returns usize. Native scanForTargetDelta returns that signed landing;
DecodeFrameResync separately rejects any next < 0 in frame_harvest.go. Preserve
this caller distinction when migrating, rather than globally treating only -1
as failure. The generator is registered; no Rust comparison or correction is
claimed for this new fixture yet.

Signed-inference-trial checkpoint final captured validation: all four march films
pass (82.27s release), and all six complete film documents match (105.64s release).
See signed-inference-trial-validation.json. Raw resync and broader acceptance
remain open; full v41 parity is incomplete.

## Signed raw-resync correction

Both raw scanner APIs now accept checked i64 starts and return signed landings.
Their loop bounds, remaining-bit checks, and prefix skips use native signed
arithmetic, including short/empty payload behavior. Both sequential raw-resync
callers now read signed headers/continuations and explicitly reject negative
recovery landings after the scanner has delivered acceptance callbacks. Their
successful resync_bits retain i64 positions; admitting only nonnegative landings
does not make a bit coordinate pointer-sized.

The expanded native scan fixture has 108 cases (38 panics); the contextual and
standalone APIs both pass. Eight full-frame native cases check calibrated/stub
rewinds, positive recovery controls and negative landing rejection. The native
cursor-return instrumentation is checked against original records and callbacks.
See signed-resync-validation.json for full checkpoint status. This supersedes
the raw-resync-start gap above; harvesting and view orchestration remain open.

Next harvesting gate: signed-harvest-successor-v41.json.zlib preserves 112 native
harvestNextBoundClean cases (30 panics). It includes empty payloads, hard/soft
bindings, optional prefixes, and signed extremes. A hard-bound successor at -32
is confirmed when the prefix reaches a substantive Delta at bit zero. Rust's
next_bound_clean still takes usize, and target harvesting still projects signed
record ends to source addresses. Native FrameConfig/observer-context harvesting
also needs coverage; current scan_frame_targets APIs take the legacy profile.
The generator and fixture are registered, but no Rust comparison or harvesting
correction is claimed yet. Avoid conflating flush-tail acceptance in this helper
with chain confirmation's explicit End-marker rule.

Final signed raw-resync captured validation, including the i64 resync_bits change:
four march films pass (81.56s release), and six complete film documents match
(104.86s release). See signed-resync-validation.json. Harvesting, view orchestration,
and broader all-data acceptance remain open; full v41 parity is incomplete.

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

Next signed-view gate: signed-views-v41.json.zlib records 792 direct native
DecodeFrameViewsCurseur calls (94 panics, 698 normal returns, 76 negative final
cursors). It varies class/generic mode, signed starts, negative/default preambles,
view count, prefixes and source size; records, cursor, completed views and current
world view are retained. Native generation passes; Rust comparison is pending.
Audit InferenceViewsOptions, NativeFrameConfig::decode_production_views,
production-frame header/component continuations, and message/control source
checks together. In particular, native placeDisponible checks signed wrapping
position+n <= frameLen; converting a negative cursor to usize before that check
cannot preserve this contract. This is a parity gate, not the deferred refactor.

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

Next march gate: signed-march-entry-v41.json.zlib has 84 native marchRecordsOf
entries, including 14 panics and 16 negative starts that return normally. The
wrapper's signed Remaining >= 8 check and DecodeFrameRecords preamble behavior
must be preserved. Rust walk_march_with_policy still takes an unsigned start,
uses saturating unsigned remaining arithmetic and projects record endpoints to
usize. Native fixture generation passes; Rust comparison/correction is pending.
Also audit keyframe queue measurements and native keyframe table continuation;
bounded replication-tail projections already have explicit in-source guards and
should not be migrated merely because a text search finds native_address there.

The queue-diagnostic gate now has an independent native fixture:
queue-unused-widths-v41.json.zlib contains 210 direct WalkPacketRecords calls,
168 full measurements and 42 wrapped-header panics. Unused negative/large ID
widths survive End-only traversal; negative preambles remain in Variant without
being applied. Rust KeyframeQueueVariant is i32 and walk_keyframe_queue_records
rejects ID widths before reading the marker. Native execution passes; Rust
comparison/correction is pending. Do not infer consumed-ID behavior from this
End-only fixture.

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

Next native keyframe-chain gate: signed-keyframe-chain-v41.json.zlib has 56
native boundary cases. For from=i64::MAX-63, want=i64::MAX, native wrapping
pos+64 admits the header check and returns Header; Rust's saturating addition
currently returns End. Exact-target short circuit and negative-start header
rejection are independently represented. No bodies are entered. Native generation
passes; Rust comparison/correction is pending. Native table fallback and raw
keyframe layout widths still need separate audits.

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

Next raw keyframe-layout gate: keyframe-layout-domain-v41.json.zlib records 90
native chain/table calls with a no-archetype entry, nine signed header skips and
five size-word widths. Ten table cases panic on negative fallback. Size words
are unused and must not trigger eager validation. Native generation passes;
Rust comparison is pending. Migrate raw NativeScanProfile keyframe scalars to an
int64 representation and adapt the existing bounded layout only at consumers;
keep the separately bounded replication API explicit. Consumed size-word widths
need their own oracle rather than inferring them from this unused-word matrix.

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
