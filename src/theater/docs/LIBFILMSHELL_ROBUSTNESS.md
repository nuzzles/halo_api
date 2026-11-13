# libfilmshell robustness work ledger

Current objective (revised November 11, 2026): broad coverage of the latest
libfilmshell's relevant film and runtime capabilities, with a stronger
source-backed canonical structure and the existing film/parser/v41/runtime
organization. Success means the major supported sections and useful runtime
views are available, with exact source data and explicit uncertainty wherever
decoding or interpretation is incomplete. It does not require every upstream
branch, feature-for-feature parity, a complete 1:1 film grammar, or a byte-exact
encoder. Older checkpoints below describe the former, stricter objective as
history; their technical gaps remain recorded but are not completion gates.
Historical `CatalogRevision` references below describe the former API; current
lookups use `film::catalog::Catalog` and one bundled dataset.
Active reference: OpenSpartan/libfilmshell `a13efb323b230c96b59685defbc376720043240f`.
The revised broad-coverage objective is met for the supported v41/27 scope:
the parser retains the major container, registry, frame, snapshot, event and
component structures with source locations, raw values and explicit stops;
runtime provides source-linked identity, roster, controller, replay, vehicle,
scoreboard, event and other projections. The main component suite covers
284/286 upstream aliases, with separate source traces for the remaining two;
creation traces cover all 23 upstream nonempty reader types. The 32-film source
preservation audit and the latest 606-passing library suite support that breadth.
This is a scope-level conclusion, not a claim that every upstream branch or
captured record decodes completely. The original study remains in
LIBFILMSHELL_STUDY.md, and the detailed gaps below remain useful future work.

Latest implementation: type-9 managed-player creation admits only the recorded
0–31 participant domain in typed complete-record accessors and replay, backed by
1,024 original-source oracle cases, following raw-position finite closure and
corrected secondary-control reference/position/target branches with a 336-case
oracle and 72 captured differential views (47 complete endpoints match corrected upstream), following
configurable chunk/film byte and packet-count budgets
with explicit admission errors and retained packet suffixes, following
nine event-body readers/corrections from the new f050371
source revision (see LIBFILMSHELL_UPDATE_F050371.md), following
source-time kill/game-event holder joins, following
control-list resource-boundary closure, following
projectile source-history retention and explicit
vehicle fixed-domain positions and nonempty
end-packet refusal, following general opaque-packet/control-suffix continuity in
contextual replay, following exact reference weapon charge/heat projections,
cached contextual equipment energy/delay replay, the canonical equipment source
view, reserve-ammunition and movement-input
channels, source-time scoreboard/engine holder joins and canonical zero-bit
position axes. Validation results are recorded in the final checkpoint below.
See the final checkpoint below and VALIDATION.md. Earlier work includes contextual
replay ride integration, following
source-linked ride histories, controller/holder integration and source-continuity
work in
replay, scoreboard, attachments and controller history. The active reference
is a13efb above; earlier oracles retain their historical pins.
Ride histories now join direct vehicle links and raw seats to ancestor changes
and current ownership. Contextual replay now exposes its own source-linked
ride diagnostics and per-player samples. Replay supports recorded engine-ID reuse and bot
identities, and distinguishes ownership interval starts from observed spawns.
Verification is recorded in the
checkpoints below and VALIDATION.md. Earlier catalogue, variant, component and
container checkpoints retain their original evidence scope.
The main component suite has 12,011 cases /284 aliases;
source traces cover all remaining upstream aliases.
The organization remains film / parser/v41 / runtime. Alias coverage is not
complete branch, format, build, packet or runtime capability coverage.

Exact-site DELTA schema context now allows a caller with independent evidence
to decode one otherwise unbound DELTA without fabricating a recorded archetype
or carrying the schema into later records. The public test covers the unbound
default, four mismatched source-site keys, a valid empty-mask record, a second
unprovided DELTA, and an invalid registry archetype. This does not resolve the
corpus of unbound DELTA stops without independent schema evidence.
The ordinary all-feature library suite passed 606 tests with 21 ignored;
strict all-target Clippy, formatting and whitespace checks passed.

Contextual replay now also tests a unique, successor-corroborated schema when
a DELTA uses a different full generation ID from an occupied slot. This can
recover an archetype missed behind an unread packet without changing canonical
Film traversal. If no candidate passes, the prior slot traversal remains
available with its ordinary source-gap handling. The focused synthetic
recovery and existing generation-reuse views pass. The ignored long-raid
regression still reaches the source-linked type-42 creation in contextual
replay (327.71 seconds in debug). A targeted release probe confirms that
contextual replay then reads the chunk-141 slot-1333 DELTA with the next index
at upstream's reported bit 2450 and closes the frame. The new recovery hook
was not needed for that case; its full-corpus effect remains unverified.

An explicit Film parse context now admits either writer branch of event 85's
conditional 68-bit tail. The tail-present 32/32/4 fields, every tail-bit
truncation, contradictory recorded variant, and following frame boundary have
synthetic source-layout tests. The external selection is retained as parser
provenance on the event list. The 32 retained films do not record their runtime
playback switches, so this does not turn their default parse into a complete
canonical event-85 read or prove tail-present captured-film behavior.
On the retained long raid, supplying a film-wide absent-tail context through
the public API reaches the source-linked chunk-140 type-42 NEW and chunk-141
type-42 DELTA, then closes the 29-record frame at bit 4306. This confirms the
new entry point's real-film continuation under that external condition; it
does not establish the unrecorded writer switches for the film.

| Requirement | Current evidence / remaining work |
| --- | --- |
| Native snapshot table traversal | Corrected the old ID-sentinel walk to 8,191 physical entries, retaining vacant headers. Original-Go fixtures and the 32-film corpus pass: 243/451 tables close, 208 retain explicit body stops; 35,634 live headers are reached after vacant slots. No header scan or vehicle-branch probing enters canonical Film. |
| Control kinds 1/2 and secondary unit blocks | Implemented with 17 upstream-traced cases, every-bit truncation, integration and rejection-boundary tests. Kind 3 remains a shared unknown, not upstream support. |
| All component dispatch aliases, not just names ending in `-component` | All 57 previously absent aliases have readers. The main suite covers 284/286 upstream names; timers and alliances use source traces because upstream splits the original mask primitive. Separate source traces cover disputed predicted ability, orientation, music, splash and TLV layouts. Conditional snapshot vehicle state still refuses unavailable context. All aliases now have some fixture evidence; complete branch/build/runtime coverage remains unproven. |
| Shared component branches and row versions | Corrected legacy filter interleaving/prefix retention; added present effect/crew/respawn/cutscene branches, dynamic/object snapshot positions, and delta vehicle physics. Registry-version and following-component boundaries tested. Biped anchor/mobility/posture/point readers and manager/player/stat/text branches have independent traces. Four-field timer order is corrected. TLV overflow/resource stops are explicit and cannot be repaired with stubs. Source conflicts remain distinct from upstream parity. Wider versions and validity, less-common branches, and complete runtime effects still need audit. |
| Snapshot/full-state versus delta and creation bodies | Enclosing keyframe/frame now supplies explicit snapshot mode. Low-frequency/navpoint/dynamic/object snapshot positions preserve raw words. Projectile defaults now dispatch in keyframes; vehicle snapshot positions and invalid common counts are corrected. 436 independent creation traces cover all 23 upstream nonempty reader types and exercise NEW/keyframe boundaries. Snapshot vehicle enable context remains unavailable; other shared readers/defaults remain under audit. |
| Map/build/registry context and position bounds | Imported all 33 levels / 65 regions, bound to recorded engine build 269225 + branch + tag. All regions decode with their own widths, with unknown/default-box safeguards. 329 authored bodies, 324 original-reader traces; runtime conversions remain separate. Other builds remain unsupported by this catalogue. |
| Bootstrap, header, event versions and variants/Bond | Added bounded canonical fixed header for the verified v41/27 registry: all 123 event-version words, build buffers/words and corruption flag. The following 11 identification fields and sequential game-options envelope retain both generic Bond v2 messages and establish the player boundary. Runtime identity/player boundaries have explicit provenance; incomplete canonical sections do not trigger searches. Recorded fire version 3 admits map-independent/default-box bodies. Selected engine/playback variant semantics now guard events 85/116, with explicit runtime assumptions. Engine-1 teleport now uses the source-verified 30/30 orientation body. Whole-table native version admission and configuration-one references are now required for nonempty messages; exact header prefixes survive refusals. Identification block internals, unrecorded kill settings and other writer layouts remain pending. |
| Events and packet variants | Added 14 event bodies and corrected five fixed skips, game-event strings and eight reference-table entries. 1,494 independent event traces (including 415 fire cases) cover 42 upstream body codes; script/kill have separate context fixtures. All 123 handle-kind rows agree. 144 original-source projectile traces establish the conditional 19-bit tail. Damage flags retain individual field/truncation boundaries. Type-8 roster/type-12 bot packets now have bounded, count-driven shared player-body readers (ROSTER.md); wider version/snapshot admission and other packet bodies remain under audit. |
| Runtime reconstruction/data catalogues | Initial comparison is in LIBFILMSHELL_CAPABILITIES.md. Recorded controller/holder integration and source-gap invalidation now reach contextual replay; engine IDs stay separate from stable human/bot output indices and supported ownership intervals are distinguished from spawns. Statborg history retains 64-bit values, identities, extras/finalized rounds/outcomes and explicit gaps. Its selected-round sum requires recorded presence and returns every source word; whole-match coverage remains unproven. Engine/player history has source-linked clocks, phase and raw respawn/lives/active values; source-time roster-holder links are implemented. Timer roles, participant aggregation and final-score recovery remain; see SCOREBOARD.md. Rides retain direct seats, ancestor outcomes and current holders (RIDES.md); contextual replay integrates ride evidence. Vehicle tracks, ground-object and zone views have since been added (VEHICLES.md, GROUND_WEAPONS.md, ZONES.md), with physics/pose and attribution gaps recorded there. Unbound DELTA recovery stays explicit runtime evidence; scans/proximity are not canonical boundaries or relationships. |
| Transport/container resilience | Implemented valid RFC zlib-window admission and explicit bounded-prefix inflation status. Configurable defaults now cap source/decoded chunks at 256 MiB, both film byte totals separately at 1 GiB and packet framing at 65,536 envelopes per chunk. Fatal byte admission uses typed errors; packet exhaustion retains source, admitted envelopes and an opaque suffix with a typed stop. Eight additional tests cover byte/count boundaries and continuity. Sequential framing now admits zero-size envelopes and post-type-7 successors; runtime continuity marks the latter as uncertain. On-demand runtime views now report marker-delimited candidates both inside unread regions and across all replay chunks when any framing/transport is incomplete, preserving source positions, counter bits and framed-packet overlap. The full-film view also exposes the upstream marker-clock heuristic with explicit rejected/incomplete status and a wide tick accumulator. It does not decode candidate entity views or promote candidates to canonical packets. Existing bounded Bond decoding covers the new upstream depth/extent safeguards; its coordinate-fitting hint guards have no canonical counterpart. Broader runtime resource guarantees remain unproven. No marker recovery in canonical data. |
| Full corpus and differential verification | Source retention passes 32 films. Removing canonical inference resolves the initial aim mismatch; historical comparison now stops at appearance/01-cadet-blue:567583. The older device dispatcher lacks the group component. A separate recorded-level fixture now verifies 90 NEW records before an unresolved DELTA; historical oracle values remain unchanged. Source coverage is not complete format coverage. |
| Complete capability audit | Initial public runtime/catalogue comparison is recorded in LIBFILMSHELL_CAPABILITIES.md, distinguishing parsed fields from missing semantic views and upstream inference. Final requirement-by-requirement and field-level evidence remains pending. A dispatch-name count alone cannot establish completion. |

The user authorized the full objective. No commit or push is implied. Existing
uncommitted work from before this effort must be preserved.

## Marker fallback checkpoint

`TheaterRuntime::unframed_tick_candidates()` searches only source bytes that
packet framing left unread. `marker_fallback_frames()` exposes the whole-replay
marker view when any replay chunk needs fallback, including markers already
inside a framed type-0 payload. Each frame candidate retains its exact chunk
position, marker-delimited span, available counter, overlap with an established
packet and explicit clock acceptance. Both queries return an error at the first
marker beyond the 1,048,576-candidate resource limit, with its chunk and byte
offset, instead of allocating an unbounded result. A missing second counter byte
yields no tick estimate rather than a synthetic zero. The heuristic clock uses a `u64`
accumulator; it does not certify a 60 Hz clock or replace recorded packet
timestamps. Candidate entity decoding and semantic replay remain open.

The unchanged latest upstream `container::Film` research reader, invoked in a
disposable example with counters 250, 251, 200, 2, selects marker framing and
returns ticks `[0, 1, 1, 8]`: the 200 jump is rejected because the following
counter returns near 251. The local two-chunk integration test produces the
same estimates and source spans. This verifies the complete-counter path of the
fallback clock, not chance markers, incomplete counters or gameplay positions.

## Packet and highlight clock checkpoint

`TheaterRuntime::packet_clock()` exposes every framed type-0 packet in source
order, its original microsecond timestamp, an optional source-bit tick counter,
the reference-convention unwrapped game-tick estimate, and the timestamp-aligned
HighlightEvents tick estimate. The common counter step is selected from adjacent
recorded counters; gaps are interpolated only within the upstream acceptance
bound. Highlight ticks use 59.94 Hz from the first packet timestamp and monotonic
clamping. These estimates are runtime data and never replace Film fields or the
playback clock. Incomplete replay framing, invalid envelopes and a
1,048,576-frame resource limit have separate errors; such a film may still be
inspected through the marker fallback view.

An unchanged latest-upstream `container::Film` research reader on an authored
three-packet stream (counter 250, missing counter, counter 2; timestamps
1,000,000/1,010,000/1,020,000 microseconds) returns game ticks `[0, 4, 8]`
and highlight ticks `[0, 1, 1]`. The local Film/runtime integration test matches
both sequences and checks the source bit/packet indices. The pinned
[clock fixture](../fixtures/clock-a13efb/README.md) compares every game and
highlight tick for 403,465 frames across all 32 captured films to unchanged
latest-upstream readers; all match, with 359,692 frames containing a readable
counter. This does not prove death-to-kill alignment or physical event timing.

`PacketClock::hitch_adjusted_clock()` now exposes the separate tick-only
HighlightEvents fallback and source-linked recorder hitches. It follows the
current upstream modal-step rule, smallest-step tie break and six-extra-tick
threshold, using wide checked arithmetic. A disposable invocation of the
original upstream `lives::hitches` and `highlight_ticks(None)` confirms two
authored jumps at adjusted ticks 3 and 7, with eight and nine skipped ticks;
the local fallback returns the same consecutive frame ticks. It remains a
diagnostic over estimated game ticks, distinct from the source-timestamp
alignment that `packet_clock()` already exposes. This synthetic comparison does
not establish captured-film hitch/death alignment.

## Component checkpoint

The existing family organization is retained; vehicle/turret fields have their
own sibling `components/vehicle.rs`. Snapshot mode is private component context,
set by the enclosing keyframe reader; canonical models still own raw fields.
The complete pinned alias inventory and the 57 additions are retained in
[component-inventory.json](research/libfilmshell/component-inventory.json).
The [component fixture README](../fixtures/components-v41/README.md) records
261 authored cases, independent upstream traces and their limits. The 198-test
ordinary suite and all 10 doctests pass. Clippy with denied warnings, strict
Rustdoc, WASM checking, formatting, whitespace and all 20 fixture hashes pass.
The full goal remains active; component-name coverage is only one requirement.

Historical gaps confirmed after the shared-reader checkpoint (the first three
are addressed in the later point/default checkpoint below):

- `creation.rs::read` biped type 35: the present post-common reference includes
  position and player fields; local `defaults::state` currently reads only its
  optional word. Establish the full branch with independent creation traces.
- `creation.rs::manager_end` type 13: conditional property values follow each
  kind; local defaults retain only the kinds. Trace both masked/unmasked forms.
- Shared point users still need audit: desired respawn location, crew order,
  flock destination/position and simulation state. Several local delta paths use
  `(6 + row_version)` widths/gates where upstream uses map points. Preserve the
  evidence for these disagreements and validate before changing their fields or
  runtime observations. Snapshot media-frame mode is now explicit, but that alone
  does not complete these component grammars or their traversal policy.
- Snapshot vehicle type-state and physics still require `vehicle+0x818`. The
  included delta forms are now supported/traced; do not guess the snapshot flag.

These items supplement the full requirement table, not replace it. Creation,
header/Bond, map/build/catalogue context, events, runtime capabilities, transport,
shared aliases and the pre-existing historical endpoint failure still require
completion evidence before the objective can be marked achieved.

Performance evidence from the component checkpoint: the long corpus run remained
CPU-active with roughly 98 MiB resident memory during the sampled interval. A
2-second macOS sample of the task-owned test process attributed 295 of 323 test
thread samples to `frame/entities.rs::infer_unbound_archetype` (303 were inside
`read_production_frame`). Broader readers make this existing speculative schema
search more expensive. Its canonical/runtime placement and source-evidence
requirements need review alongside libfilmshell recovery; do not optimize it by
accepting weaker candidates or treating inference as recorded declarations.
The scratch sample is `/private/tmp/halo-components-corpus.sample`.

Final component-checkpoint corpus result: source preservation passes all 32 films
and 403,465 frame packets, with 3,667 summary source/timestamp checks. Independent
summary semantics/service totals also pass. The source run took 380.27 seconds
in debug execution. Neither corpus result closes the outstanding semantic,
structural-context, inference-placement or whole-capability requirements.

## Shared-reader and creation checkpoint

Extended component fixtures from 261 to 304 cases / 68 names. Added explicit
snapshot dynamic/object positions, present effect/crew/respawn/cutscene handles
and payloads, and both delta vehicle-physics forms. Snapshot dynamic prefixes
refuse only after both prefix bits; snapshot vehicle context refusal consumes
none of the body. Snapshot NaN payloads and negative zero remain raw words.

Added 176 independent creation traces: 96 projectile, 32 vehicle and 48 invalid
common-count cases for other object callers. Projectile defaults now dispatch in
keyframes with raw target words/positions; NEW records retain handles and tick
quantization. Vehicle media-frame snapshot words no longer require map widths.
The common reader rejects counts 5–7 at the count field. Three tests exercise
source fields/serde, every-bit truncation and actual NEW/keyframe/default/next-
component boundaries. New fixture provenance is in `fixtures/creation-v41` and
all 22 manifest hashes verify. Original LevelUp golden files remain untouched.

Focused component/creation and broader verification results for this checkpoint
are recorded in VALIDATION.md. This closes selected reader branches, not the
full active objective. Existing organization and canonical/runtime separation
are retained; no commit or push was made.

Transport source comparison (not yet implemented): upstream `chunk::is_zlib`
admits every valid RFC 1950 CMF/window header, while local
`parser/transport/compression.rs::inflate_film_chunk` only attempts CMF 0x78.
Upstream `MAX_CHUNK_BYTES` caps output at 256 MiB and reports `TooLong`; local
inflation caps initial capacity only and has no output limit. Retain original
compressed bytes, bounded decompressed prefixes and an explicit limit/refusal
status when adding this; do not conflate it with a malformed stream. Upstream
`container::walk_packets` permits zero-sized packets of any type and continues
past type 7, while local `transport/packet.rs` stops for zero size except type 7
and stops after type 7. Confirm terminal semantics/format evidence and keep any
suffix opaque before revising that policy. Upstream marker fallback is heuristic
recovery, not canonical framing, and must not silently become packet structure.

Inference placement audit confirms a concrete source-model issue beyond speed:
`frame/entities.rs` calls `infer_unbound_archetype` on an unbound DELTA during
normal traversal, tries up to 50 schema decodes, checks a later bound successor,
and marks the unique result as `soft` through `state::bind_inferred`. The emitted
`EntityRecord` omits inherited archetype (correctly avoiding a fabricated NEW
field), but its component names/fields and Complete status do not identify that
the schema was speculative. Keep recorded declarations and ordinary inherited
bindings in canonical traversal; move or explicitly separate speculative schema
recovery in runtime. Any performance change must preserve the two-record,
strictly increasing slot, generation and uniqueness checks for that recovery.
Do not optimize the hotspot by weakening those requirements or cache guesses
across incompatible registry/context/generation boundaries.

Final shared-reader checkpoint verification: both CI feature configurations pass
201 ordinary tests and 10 doctests each. Strict Clippy/Rustdoc, all-feature WASM,
formatting/whitespace and all 22 fixture hashes pass. Source preservation passes
32 films / 403,465 frame packets (240.79 seconds debug), and summary semantics
passes 3,667 pinned events plus service counts. The old `(3,166)` versus `(3,19)`
boundary mismatch remains unchanged. The source corpus used identical production
code to the final suite; only synthetic endian fixtures/assertions changed while
it ran. No further production change was made after that corpus began.

Prioritize the confirmed inference/source-model separation next: additional
reader coverage broadens speculative candidate decoding, so its provenance is
part of canonical correctness, not merely an optimization. Continue the shared
point/default, transport, structural/Bond, event and catalogue work in the table.
The full objective remains active and is not achieved by this checkpoint.

## Runtime recovery separation and bounded transport checkpoint

Moved unbound-archetype search and its negative fixtures into
`runtime/replay/schema_recovery.rs`. Canonical traversal has no recovery hook;
it stops before the unbound body's mask/components even when a unique hypothesis
exists. Contextual replay opts in and exposes `ReplaySchemaRecovery` diagnostics
with full candidate/successor IDs and source extents. The two-record, slot-order,
generation, soft-binding and uniqueness checks are preserved. New tests cover a
positive contextual recovery beside unchanged canonical output, ambiguous schemas,
generation/soft-successor refusals, and every-bit truncation of corroborating data.
The source-model defect described above is closed for this unbound-DELTA search;
other reference traversal policies still require their broader audit.

The earlier aim historical mismatch disappears without changing its oracle.
The next mismatch is `appearance/01-cadet-blue/chunk-001-type-2.bin:567583`, actual
`(3,4603)` versus `(1,382)`. The current and pinned upstream device-position-group
readers both consume 32 bits plus an optional eight-bit index at that boundary;
the captured group word and presence bit there are zero (33 bits). This locates
an older unsupported-reader boundary but does not justify waiving/regenerating
that oracle or establish correctness of every following record. Continue its
independent investigation. Source preservation still passes all 32 films and
403,465 packets; the recovery-only debug run took 61.60 seconds versus 240.79 at
the prior checkpoint (observations, not a controlled benchmark).

Transport now recognizes valid RFC 1950 CMF/window headers beyond 0x78, caps
inflated output at upstream's 256 MiB ceiling, and emits `ZlibLimitExceeded` with
the bounded prefix while original compressed bytes remain retained. It preserves
existing malformed/checksum/partial-stream distinctions. Five tests cover stored,
fixed and dynamic Huffman blocks, windows, limits and truncation; the 16-case
upstream export verifies the independent Huffman streams and limit boundaries.
Fixture manifest contains 25 entries. Source details and limits are in
`fixtures/transport-v41/README.md`. Zero-size/type-7 packet semantics and other
transport/version admission requirements remain open. Full-goal completion is
still unproven; shared/default readers, structural/Bond/event grammar and
catalogues remain required, alongside the complete capability audit.

Final combined checkpoint: both feature configurations pass 208 ordinary tests
and 10 doctests each. Strict Clippy/Rustdoc, WASM, formatting/whitespace and all
25 fixture hashes pass. Post-transport source preservation passes all 32 films /
403,465 packets in 58.98 seconds debug; summary semantics/service totals pass.
The later historical device boundary remains unresolved. Next work remains the
shared point/default branches (biped creation and manager property values),
structural/Bond/event/packet grammar, map/build catalogues, remaining runtime
recovery policies and the full requirement-by-requirement audit. Keep the full
objective active; this checkpoint does not establish complete canonical parsing.

## Shared points and remaining creation readers

Biped type 35 now retains the position and inverse-gated player index after its
present post-common word reference. Manager type 13 now reads each property's
value immediately after its tag, including masked arrays. A shared width helper
also corrects component masked property tags 12–15 from four bits to 32 bits.
Independent creation traces now cover 436 cases: the prior 176 plus 72 biped,
64 property-manager, 52 other manager and 72 item-family cases. All 23 nonempty
reader types supported by pinned upstream have traces. Selected branch coverage
is not proof of every combination or of game construction state.

The component fixtures now cover 464 cases / 75 aliases. Desired respawn, crew
order, flock position/destination and simulation state follow the shared point
grammar instead of deriving axis widths from registry row versions. Snapshots
preserve raw floating-point bits. With no map context, a default-box point still
reads its three fixed 22-bit words; an indexed point preserves the gate and stops
before unavailable widths. Canonical configuration admits the independently
traced simulation reader; the old historical oracle keeps its explicit original
policy. DesiredRespawnLocation's private observation retains recorded fields and
presence, without the old fabricated precision value; no runtime consumer uses
that obsolete shape. Film/parser/runtime organization remains intact.

The independent packet-envelope audit is reproducible with
`python3 src/theater/docs/research/libfilmshell/packet_inventory.py experiments/films`.
Its aggregate output is retained in `research/libfilmshell/packet-inventory.json`.
Across all 32 films, all 483 type-7 packets have zero payload and end their chunk;
no other zero-size packets or packets after type 7 occur. Both envelope policies
walk this corpus identically. This does not establish legality of nonterminal
zero-size packets or post-7 packets, so that policy difference remains open.

The creation source audit found no further nonempty reader types implemented
upstream but absent locally. Remaining creation requirements concern exhaustive
branches, external construction context and unknown/empty-default policy, rather
than a missing upstream type-dispatch entry. Header/Bond/event grammar, shared
component branches, map/build/catalogue context, runtime policies, the historical
boundary disagreement and complete capability audit remain open.

Final verification passes both feature configurations with 209 ordinary tests
and 10 doctests each. Strict Clippy (all targets/features), Rustdoc, WASM library
checking, formatting, whitespace and all 25 fixture lengths/hashes pass. The
original LevelUp fixtures remain unchanged. The historical comparison failure
above remains open; neither trace counts nor source preservation establish full
canonical format completion.


## Event bodies, strings and references

Added event bodies 11, 22, 27, 32, 41, 46, 47, 58, 64, 78, 100, 109, 116 and
118. Replaced fixed skips for 7, 9, 38, 40 and 76 with their recorded conditional
fields. Event 82 now retains each string byte through the first NUL or the
16-byte limit. Corrected reference configurations for codes 14, 17, 50, 79, 104,
106, 115 and 116; unsupported bodies still stop after their established headers.
All 123 reference rows match unchanged pinned upstream `handle_kinds`.

Full-precision points in teleport, respawn, AI jump and game events, plus the
existing fire/translocator helper, admit their self-contained default box (three
22-bit words). An indexed box needs an explicit descriptor for that region;
without it the parser retains the gate and refuses before unavailable widths.
This does not infer a world-space position or import upstream's fallback to a
play-space box. The retained captured Oddball test now stops at bit 305 instead
of 304 without context because that gate is established independently. Its
successful contextual endpoint, 1999, and captured bytes are unchanged. A default
box is a valid branch rather than an invalid-region negative case; the unmapped
indexed-region refusal remains tested.

`events-v41` contains 243 independently authored lists traced through the original
upstream walker, plus its full handle table. Four tests check exact primitive
boundaries/source values/serde, all-bit truncation, unavailable or invalid context,
and Film/contextual-frame integration with following entities/controls. No
original LevelUp oracle was regenerated. `research/libfilmshell/event-inventory.json`
records the dispatch comparison (44 upstream, 46 local), not branch completeness.
Local-only bodies 12, 34 and 117 are not validated by an absent upstream reader.

The next concrete gap is event 102 (`NetworkedActionRequest`). Upstream delegates
its five-bit variant to `components::action_end`, with variants 0–25 and an
explicit unsupported branch of variant 18. Local `components/biped.rs::action`
only reads variants 0–5 and silently skips other variant bodies before continuing
the enclosing list. Share the corrected bounded variant reader across biped
components and event 102, preserve source prefixes and refuse unknown variants;
trace both entry points and their following-field boundaries independently.
This also belongs to the unfinished shared-component audit.

Other open event requirements include the complete fire/projectile branch audit,
precision/region context, event versions, and event 15's unrecorded runtime gate.
The known upstream fixed-eight-bit detonation tail differs from the existing
locally traced conditional 8/19-bit tail; do not replace it without resolving
that source evidence. Header/Bond, catalogues, remaining components/runtime and
whole-format verification also remain open.

Final event checkpoint: strict Rustdoc, all-feature WASM library checking,
formatting and whitespace checks also pass. Production code did not change after
the corpus run. The full goal remains active and incomplete.


## Shared action variants and event 102

`components/actions.rs` now owns the action bodies shared by biped lists and
NetworkedActionRequest (102). Variants 0–25 follow the pinned reader; variant
18's true gate and variants 26–31 explicitly refuse. Biped lists no longer treat
unrecognized bodies as empty before reading their following sub-states and tail.
Existing field names for variants 0–5 remain intact. Event 102 adapts the same
bounded component fields into event fields and retains failed-read attempts;
no action semantics or speculative runtime state are added to canonical Film.

The independent author supplies 113 bodies, including all four generations of
the variant-0 handle. The component fixtures now total 696 cases / 76 aliases,
adding 232 action-list cases across snapshot/delta forms, maximum-length mixed
lists, all first-73-mask-bit sub-states, and a complete action followed by a
refused one. Event fixtures now total 356 cases, with 113 action events followed
by an independent zoom event when complete. Existing every-bit truncation,
source/serde and enclosing component/frame tests exercise all added cases.

Both upstream exporters run unchanged enclosing readers. The event exporter
also instruments the component reader's primitive reads/skips. A refused nested
variant 18 consumes one flag but leaves upstream's parent event cursor before
it; its output now records `reader_end` separately from the furthest traced
`end`. The test requires exactly that known one-bit difference and retains the
flag in canonical output. This is preservation of observed source reads, not an
upstream return-value patch or successful decoding of the refused branch.

All 44 upstream event body codes now have local dispatch routes (47 locally,
including pre-existing local-only 12, 34 and 117). This closes the missing-code
inventory, not the full event requirement: fire/projectile conditional branches,
precision/region context, version selection and event-15's runtime dependency
still need evidence. The broader shared-component, header/Bond, catalogue,
runtime and whole-format requirements remain active.

Final action checkpoint: both feature configurations pass 213 ordinary tests and
10 doctests each. Strict Clippy/Rustdoc, all-feature WASM, formatting/whitespace,
all 27 fixture hashes and both extended generators' reproducibility pass. Source
preservation passes all 32 films / 403,465 packets in 62.90 seconds debug;
summary semantics passes 3,667 pinned events across five films plus independent
service counts. No production code changed after that corpus run. The historical
oracle remains unchanged and still fails at appearance/01-cadet-blue:567583,
actual `(3,4603)` versus `(1,382)`. Full canonical parity remains unproven.


## Projectile point decoding versus map interpretation

Projectile body completion no longer depends on obtaining a position in the
supplied map region. A shared bounded event-point reader distinguishes fixed
default-box source fields from map-region codes internally. Event 5 consumes
three 21-bit default words (precision 15); event 6 consumes three 18-bit words
(precision 12). Canonical parsing can decode those self-contained forms without
map context. Indexed points still need an explicit matching region descriptor;
known prefix fields and the default-box gate are retained before refusal.
Runtime detonation interpretation only accepts the map-region result, so a
complete default-box event cannot publish an invented map coordinate.

Added 32 independent projectile lists (388 event fixtures total), including
both point forms and optional prefix/tail fields. Existing raw-field, all-bit
truncation, missing/invalid-context and Film/contextual-frame checks cover them.
A fifth event test explicitly checks that default-box events remain complete
while runtime map projection returns no position. Captured Argyle's no-context
stop now retains its prefix through bit 31 instead of stopping at body start 22;
its contextual endpoint 535 and captured bytes remain unchanged. The old
reference-continuation fixture's existing projectile exception now checks the
known prefix and zero indexed-box gate, rather than demanding an empty body.
No historical expected field/boundary data was regenerated.

The independent detonation traces deliberately select its eight-bit tail form.
The pinned upstream reader consumes a fixed eight bits after the final flag;
the earlier locally traced reader selects 8/19. That discrepancy remains open
and the existing 19-bit behavior is preserved. The source comparison of fire's
counts, markers, last-present barrel selection, target vectors and optional tails
found corresponding local branches; full independent fire traces remain next,
alongside multi-region precision/build context and the structural work.

Final projectile-point verification: both feature configurations pass 214 tests
and 10 doctests each. Strict Clippy/Rustdoc, all-feature WASM, formatting/whitespace,
all 27 fixture hashes and event-generator reproducibility pass. Source retention
passes all 32 films / 403,465 frame packets in 67.49 seconds debug, with summary
source checks; summary semantics/service totals pass all 3,667 reference events.
The historical comparison remains at appearance/01-cadet-blue:567583, actual
`(3,4603)` versus `(1,382)`. No production edits followed the corpus run, and no
old oracle was regenerated. Full-goal completion remains unproven; fire branch
traces, the detonation-tail discrepancy, map/build/context, shared components,
header/Bond and remaining runtime/whole-format requirements stay open.


## Fixed bootstrap header and fire branch checkpoint

Added 415 independently authored fire lists, traced through unchanged pinned
upstream `events::walk` (803 total event cases). These cover the conditional
fire reader, count encodings and maxima, marker kinds/indices, both barrel
coordinate widths, last-present-barrel ordering, target/vector gates, block
selectors, both point sites and optional tails. Six event tests verify primitive
boundaries, exact source fields/serde, every-bit truncation, absent/invalid map
context, independent configuration/v3 admission and following frame views.

The bootstrap models and sibling reader now live under
`parser/v41/chunks/registry`. For the known v41/27 registry (50 blocks, known
fingerprint, endpoint 0xcb208), the parser sequentially retains 123 event-version
words, three fixed build buffers, two words and the corruption-check flag. Every
field has exact raw bits and a source range; truncation preserves the prefix and
first unavailable field. Other layouts remain unadmitted. This is the 4,769-bit
fixed header only, not a decoded identity/variant/Bond suffix. No build anchor
scan selects it. `FilmRegistryRead` adds optional `bootstrap_header`; this is a
new public construction field and serialized data when that header is present.

Canonical fire v3 admission now comes from a complete supported header recording
version 3 for event 36. Layout admission and the optional map descriptor are
separate: short, map-independent and default-box bodies decode without a map;
indexed points preserve their gate before refusing unavailable widths. Runtime
contextual decoding retains its existing explicit v3 policy; the broader runtime
and event-version audit remains open. Three bootstrap tests cover independent
field goldens, every-bit truncation, unsupported registry/version rejection and
actual Film/event/entity/control integration. Missing or truncated headers do
not admit fire v3, even if a partial table already contains its version word.

The independently reproducible `research/libfilmshell/bootstrap-inventory.json`
checks all 32 films: each is v41/27, has the same original registry prefix, has
123 event words before the separately observed aligned build anchor, and records
fire version 3. `fixtures/bootstrap-v41` retains only that captured registry
prefix plus synthetic header fields; no identity/player data was added.

The detonation-tail disagreement remains explicit. The exact pinned LevelUp
`vue_a_charges_armes.go` confirms `chargeDetonation` reads 19 bits when the final
direction flag is set and 8 otherwise; libfilmshell consumes 8 in both cases.
The existing 19-bit branch is retained. Source agreement with LevelUp is not an
independent engine-writer proof, and the shared eight-bit traces do not settle
this disagreement. Do not replace the branch or rewrite its old oracle to match
libfilmshell mechanically.

Verification: both feature configurations pass 218 tests and 10 doctests each.
Source preservation passes 32 films / 403,465 packets (64.59 seconds debug), and
summary semantics passes 3,667 pinned events across five films plus service
counts. The historical device boundary still fails at
`appearance/01-cadet-blue/chunk-001-type-2.bin:567583`, actual `(3,4603)` versus
`(1,382)`; original LevelUp expectations remain unchanged. The fixture manifest
now has 29 entries. Strict Clippy/Rustdoc, WASM, formatting, whitespace and
fixture/export reproducibility are recorded in VALIDATION.md.

The full objective remains active. Continue with later bootstrap/identity/Bond
boundaries and full event-version admission, versioned multi-region catalogue
context, the remaining shared component branches, detonation evidence, packet
bodies, runtime policies and the historical mismatch. This checkpoint establishes
concrete fire/header support, not complete canonical format coverage.


## Shifted identification and canonical runtime identity

The canonical bootstrap header now owns an optional `identification` read. After
its corruption-check bit, the reader preserves two 256-byte buffers, a timestamp,
three words, three 4,096-byte blocks and two 16-byte map-variant IDs. Every field
retains exact raw bytes/ranges, including zero timestamps/IDs and nonzero unknown
fields. The 12,848-byte section is shifted by one bit; successful completion
establishes the body at `0xce68c * 8 + 1`. Its opaque block internals are still
unknown, and this is not a decoded game-options/Bond/player-table body.

The width evidence is the pinned LevelUp `film_identity.go` writer map for
`FUN_14299b198`/`FUN_14299b278`; a fresh original libfilmshell `header::read` export
independently confirms timestamp and both GUID locations. Four synthetic cases
cover both preceding flag values and zero/nonzero timestamps/IDs. Two tests
verify complete fields/serde/following bits and truncation at every scalar/GUID
bit plus endpoints/midpoints of larger buffers. Other fixed extents are from the
writer map, not an upstream read trace. No original corpus or oracle was edited.

Runtime identity now projects the canonical header and identification on this
supported layout. `IdentityRead::boundary_source` distinguishes
`CanonicalBootstrap`, `BuildStringSearch` and absent/old-export `Unknown`.
Unsupported layouts retain the existing runtime search, labelled explicitly.
A truncated canonical section never falls back to that search. Two runtime tests
verify identical legacy projected values/field traces and a deliberately embedded
`HI_` string that must not override the canonical build field. This changes the
public `BootstrapHeaderRead` construction shape with `identification`, and adds
`boundary_source` to runtime identity serialization; missing fields deserialize
with their documented defaults. Film/parser/v41/runtime organization is retained.

Final verification: both feature configurations pass 222 tests and 10 doctests.
Strict Clippy/Rustdoc, WASM, formatting/whitespace, generator/export reproduction
and all 31 fixture hashes pass. Source retention passes 32 films / 403,465 frame
packets (66.82 seconds debug). Summary semantics/service counts pass 3,667 pinned
events across five films. The original historical comparison still fails at
appearance/01-cadet-blue:567583, actual `(3,4603)` versus `(1,382)`.

The next owning source has been located: pinned LevelUp
`film_variante_de_partie.go`, cached at `/private/tmp/halo-pinned-film-variant.go`.
It maps the sequential game-options body (`FUN_1407ec560` writer,
`FUN_1407ee138` reader), including the initial ten fields totaling 210 bits,
first Bond message, bounded strings/optional blocks, second Bond variant message
and the following player-table boundary. Its Bond walker is selective and skips
unknown nested values; port the wire grammar into a bounded lossless field model,
not its selected projections or defaults. Pinned libfilmshell `variant.rs` has
broader primitive-type skipping but searches for variant/string anchors and
rounds numeric values, so it is an independent primitive reference, not the
canonical model. `film_identity.go` also identifies the first two body fields as
game mode and recorder simulation (three bits each); the latter may supply the
remaining script-event context, which still needs its owning consumer traced.
The source is a next-step lead, not completed support.

The full objective remains active: game-options/Bond and broader version/context
admission, opaque identification internals, multi-region catalogues, remaining
shared readers and packet bodies, runtime policies and the historical mismatch
still require completion evidence.

## Lossless Bond, game options and canonical player boundary

The generic Compact Binary v2 syntax and bounded readers now live under
`parser/v41/chunks/registry`. The envelope follows pinned
LevelUp's game-writer map and reads both Bond messages, prefix fields, bounded
strings, optional blocks and tail sequentially. All CBv2 value types 2–18 retain
raw encodings, ordered/duplicate fields and map entries, extended IDs, lengths
and terminators. Fixed-width arrays retain one packed raw payload. No semantic
schema defaults or rounded values are added to canonical data.

Malformed lengths/types/integers, truncation, enclosing-structure crossings and
resource limits stop explicitly. Maximum depth 64 and the 100,000 value/header
budget are resource policies, not wire-format maxima. Existing chunk source
bytes retain every unread suffix. Identification `end_bit` still denotes the
start of game options; only complete `game_options.end_bit` establishes players.

Independent fixture evidence covers 130 valid Bond structures through original
pinned libfilmshell value traces, plus eight authored full envelopes. Raw-bit
reconstruction/serde, every-bit truncation of small structures, representative
large-buffer cuts, all alignment shifts used by the envelopes and malformed/
resource-limit negatives pass. Both generators and original-reader export
reproduce exactly; all previous 31 fixture hashes remain unchanged and the
manifest now pins 34 files. See `fixtures/bond-v2/README.md` for exact provenance.

The independent captured comparison passes all 32 films (14.94 seconds debug):
canonical options endpoints and legacy pattern-search starts agree, and projected
slots are identical. The test forces legacy search by removing the canonical
header from a cloned film, avoiding a comparison against a forwarded endpoint.
Runtime now uses the canonical boundary directly; an incomplete canonical body
refuses without scanning. Other layouts retain explicitly labelled PatternSearch.
The vacant-slot reader additionally requires the entire personalization payload
before accepting a complete slot. A synthetic all-vacant table verifies all 32
slots, final truncation, invalid first record and no-search refusal.

Player-slot grammar and build-specific widths remain runtime interpretation.
Generic Bond syntax does not establish semantic variant fields/defaults or the
script-event gate. These, wider event-version/context admission, opaque blocks,
versioned map catalogues, shared component branches, other packet bodies, runtime
capabilities and the historical device mismatch remain open. The full objective
is active; this checkpoint is not complete canonical film parsing.

Final Bond checkpoint verification: both configurations pass 228 tests and 10
doctests; the first full run hit the preexisting wall-clock backoff test and a
full retry passed. Strict Clippy/Rustdoc, WASM, formatting/whitespace and all 34
fixture hashes pass. Source retention passes 32 films / 403,465 packets (71.28
seconds debug), and summary semantics passes 3,667 events plus service counts.
The historical device mismatch remains `(3,4603)` versus `(1,382)` at
appearance/01-cadet-blue:567583. Original LevelUp fixtures remain unchanged.

The next source was located: pinned LevelUp `vue_a_charges_execution.go`, cached
at `/private/tmp/halo-pinned-vue_a_charges_execution.go`. `scriptDuFilm` selects
the optional 15-bit prefix from recorder simulation != 2, following writer
`FUN_142eec4d8` / `FUN_1404f293c`. The playback reader `FUN_14080bb4c` uses the
opposite current-simulation predicate, because playback configures itself as a
distributed client; use the recorded writer context. Simulation 2 has source
analysis but no captured example in that reference's corpus. Version table
source `vue_a_versions.go` records script version 1. This is a concrete next
admission/test target; broader version admission remains separate.

## Script v1 recorded context

Canonical parsing and replay now share `context/events::recorded_script_prefix`.
A complete supported header/identification/options section, script version 1 and
valid recorder simulation 0–3 are required. Simulation 2 omits the optional
15-bit prefix; 0/1/3 include it. The following word/count/payload are unchanged.
Missing/truncated context, reserved simulation values and other versions refuse
before the body. Replay no longer assumes `Some(true)` for every recording.

Three tests cover 32 independently authored event lists (both forms, absent and
present references, lengths through 1,023 bits), every-bit truncation, raw field
values/extents, serde, context refusal and actual Film continuation into zoom,
entity and control views. Sixteen prefix-present cases are independently traced
through original pinned libfilmshell; its unconditional-prefix reader cannot
validate the other branch. All 32 captured films independently have simulation 3
and script version 1. The simulation-2 branch is supported by the pinned writer
analysis plus synthetic tests, not a captured example or replay-reader behavior.
The manifest now pins 36 files and all preceding 34 hashes remain unchanged.

This closes the specific Script gate identified in the initial study. It does
not close broader event version admission or variant-dependent event gates,
map/build catalogues, snapshot vehicle context, opaque identification internals,
canonical player/other packet bodies, shared branches, runtime capability audit
or the historical device mismatch. The full goal remains active.

Final Script checkpoint: both feature configurations pass 231 tests and 10
doctests, strict Clippy/Rustdoc, WASM, formatting/whitespace and all 36 fixture
hashes. Source retention passes 32 films / 403,465 packets (59.80 seconds debug),
and summary semantics passes 3,667 events plus service counts. The unchanged
historical oracle still fails at appearance/01-cadet-blue:567583, `(3,4603)`
versus `(1,382)`. No production source changed after these checks.

Next confirmed context gaps in the same pinned `vue_a_charges_execution.go`:

- Event 116 `chargeEffetsDeTeleportation`: when its first bit is set, the
  orientation grammar depends on game-engine type. The current local and pinned
  libfilmshell path uses `FUN_140c5fa84` without checking the variant. Writer
  `FUN_142efa2a8` / `FUN_141f86118` instead selects `FUN_142e29bac` when engine
  type is 1. The variant carries m_gameEngineType; its Bond path is documented in
  `film_variante_de_partie.go`. Derive only justified schema semantics from the
  raw AST, retain explicit unknown/absent states, and refuse the unported branch
  before treating later bits as the next field. First-bit-zero cases need no
  engine context. Do not mistake current independent traces for proof of both
  engine branches.
- Event 85 `chargeJoueurTue`: a conditional 68-bit tail (`32,32,4`) follows the
  retained kill fields. The guards depend on unrecorded runtime settings
  `kill_playback_enabled` / `play_of_the_game_enabled`, plus variant settings and
  other state. LevelUp explicitly *assumes* both runtime settings retain their
  false defaults; that external user's policy is not canonical wire evidence.
  The local `kill_fields` path currently stops at the base fields and calls the
  body complete. Check whether recorded variant values can prove both guards
  false; otherwise retain the known prefix and an explicit missing-context stop.
  Keep any chosen default hypothesis in runtime with provenance. Do not silently
  import the reference's assumption or use endpoint plausibility as proof.

These are concrete follow-up requirements, alongside the broader table; the
full objective is not complete and remains active.

## Variant-derived event boundaries and runtime assumptions

Added private schema projection over the canonical Bond AST in
`context/events/variant.rs`. It selects exactly one of the four optional variant
instances, scopes field IDs after the final base terminator, and rejects
conflicting/duplicate selected fields or wrong types. Missing parent structures
remain unknown; scalar defaults apply only inside their complete owning schema.
Engine default 0 and playback boolean defaults false are attributed to the
pinned writer/schema analysis in `LOT_VA_V3.md`. No projected/default value is
written into Film, and the original AST retains all duplicate/unknown fields.

The new 77-envelope fixture covers selection, signed engine values, explicit and
omitted booleans, missing parents, duplicates/wrong types and inherited IDs.
The original LevelUp walker independently agrees on 66 comparable valid cases;
its exact source Git blob is checked before execution. Its projection bodies
are unchanged; a small Go harness supplies bounded source primitives. Separate
negative/inheritance cases validate stricter local safeguards rather than claim
agreement. All previous 36 fixture hashes are unchanged; the manifest now pins
38 outputs. The generator and original-reader export reproduce exactly.

Canonical event 116 now requires known non-1 engine context after a set first
gate. Unknown context stops at that gate; engine 1 refuses the still-unported
orientation branch. A clear gate needs no variant context. Event 85 retains its
known base fields but declares completion only when the recorded variant proves
the optional 68-bit tail absent: play-of-the-game false, and killcam false or
engine 1. Otherwise it stops with MissingKillPlaybackContext. The reference's
unrecorded disabled-feature assumption no longer establishes canonical fields.

Replay keeps that previous default policy with explicit evidence. Actual uses of
KillPlaybackDisabled or TeleportEngineNotOne appear on EventListRead.assumptions
and survive in source-linked ReplayDiagnostics.event_assumptions. Recorded engine
1 is never overridden. Canonical walks have no assumptions. Existing reference
fixture helper entry points are now test-only; production uses explicit
EventLayout context. Unknown-tail canonical views can complete less often while
retaining all original bytes; that is a fidelity correction, not source loss.

Three ordinary tests cover schema projection, all 27 known/unknown engine and
boolean combinations, runtime/default provenance, every-bit kill-base truncation,
misleading following bits and actual Film/event/entity/control continuation or
refusal. An independent captured inventory verifies engine 2, killcam false,
play-of-the-game true in all 32 films. Real replay checks report 96 assumed
Octagon event lists and 246 Recharge event lists, with valid source ranges and
unchanged canonical assumption-free reads (43.49 seconds debug).

The current full goal remains active. The actual engine-1 teleport orientation
layout is still unknown here; no captured instance validates it. Unrecorded
kill-playback switches still prevent unconditional canonical completion. Broader
event-version admission, versioned map/region catalogues, shared/snapshot reader
branches, canonical player/other packet bodies, runtime capability comparisons
and the historical device mismatch still require completion evidence.

Final variant checkpoint static checks: strict Clippy/Rustdoc, all-feature WASM,
formatting/whitespace and all 38 fixture hashes pass. Both ordinary feature suites
and the source/summary/replay evidence above exercise the same event policies.


## Build-bound multi-region positions

Imported all 33 pinned levels and 65 BSP regions using exact exported f32 words.
The engine build ID is 269225 and branch HI_1_13_0; the catalogue's longer label
also carries date/time metadata. Film BuildVersion (`6.10026.19225.0` in the
retained corpus) is a different namespace and must not be compared directly to
that label. Complete canonical sections supply the numeric BuildId, branch and
LevelTag. All 32 retained films bind through those fields. Synthetic altered-ID,
branch/tag and incomplete-section tests refuse binding without searching.

World and projectile descriptors now hold indexed region tables. Components and
events read the selected region's widths, including mp_02's region 4, instead of
borrowing primary widths. A verified table overrides historical opaque aggregate
widths; an absent region refuses before axes and cannot be patched with a stub.
Missing layout propagates Unsupported rather than source Truncated. The object
point's default-box branch now uses three fixed 22-bit words without map context.
Runtime selects the same region for player/object/detonation conversion only
when caller primary bounds/index width agree with the bound catalogue. Default
boxes do not become primary-region coordinates.

New evidence: 329 authored point bodies, every-bit cuts and source-field checks;
324 independent original-reader traces and following markers; 131 actual DELTA
component continuations and unknown-region refusals; region-aware detonation/zoom
continuations; and player/object coordinate tests. Five mp_02 region-4 bodies
are deliberately excluded from upstream reader parity because its Params/Layout
have only four region slots. Their independent basis is the original exported
catalogue. All previous 38 fixture hashes remain intact; the manifest pins 41
files. Full source/feature/static verification is recorded in VALIDATION.md.

This closes the catalogue and indexed-region gap for this engine build. The full
goal remains active: broader version admission, remaining shared/snapshot
branches and packet bodies, canonical player grammar, runtime capability audit,
engine-1 teleport layout and the historical device mismatch still need evidence.

Next confirmed shared-reader lead: delta `object-position-component` with its
first (`high_precision` / upstream `attached`) bit set still uses the legacy
fixed 59-bit local skip. Pinned upstream `ObjectPosition` instead reads three
14-bit local coordinates, an optional object handle, an optional 11-bit region,
and a two-bit finite tail. The default/indexed branches verified in this
checkpoint do not cover that attached branch. Trace every gate combination and
row/context width before replacing it; retain local coordinates separately from
world positions. This is a concrete next requirement, not completed support.

Final region checkpoint: both configurations pass 242 tests + 10 doctests;
strict Clippy/Rustdoc, WASM, formatting/whitespace and all 41 fixture hashes pass.
Source retention passes 32 films / 403,465 packets (221.39 seconds debug);
summary semantics and replay assumption provenance pass. The larger retained
source walk has higher observed time/memory costs, recorded in VALIDATION.md,
and does not turn source coverage into proof of complete semantic decoding.
The historical mismatch remains unchanged. The full goal remains active.


## Attached positions, frequency tables and engine-1 teleport

Replaced the fixed 60-bit attached-object skip with its actual discriminants:
three 14-bit local coordinates, optional object handle, optional region, finite
tail. The 160 new cases cover 47/58/62/73-bit bodies, generations 0–3, row versions
0/1/2/4 and both independent gates, without map context. Parent-local codes stay
separate from world positions.

High-frequency now dispatches by the recorded archetype: 3 reads 16/8/2 bits,
4 reads the eight-bit counter, and unestablished tables refuse before the body.
This follows pinned LevelUp's descriptor/vtable analysis; the original
libfilmshell confirms both supported widths. Generic synthetic next-byte tests
now use equipment-being-hacked-component. Two older continuation-oracle tests
keep their original high-frequency names/bytes under an explicit test-only
8-bit calibration; their original fixture files/values are unchanged. Those
legacy tests validate continuation, not the corrected canonical frequency table.

The expanded original-reader suite covers 1,157 cases / 153 aliases: all prior
696 cases/traces unchanged, plus 160 attached positions, 40 mutually supported
orientation cases, 225 fixed-width cases across 75 existing aliases, and 36
frequency cases. The inventory now explicitly lists the 133 aliases outside this
fixture set; absence there is not proof of absence from other tests. Counts alone
still do not establish complete conditional-branch coverage.

The additional orientation comparison exposed a conflict in libfilmshell:
its raw selector wins over C, and a clear delta-roll gate reads eight bits.
Pinned LevelUp disassembly instead shows C overriding the mode and a clear
roll gate reading nothing. The local reader's existing behavior is retained;
32 independently authored cases execute the original LevelUp decoder functions
with Git blob verification, including row versions below 2. No upstream-parity
claim is made for the conflicting branches. All 32 also exercise canonical
DELTA/following-byte boundaries and every-bit truncation.

That same source establishes FUN_142e29bac as inverse-gated direction30 + roll30.
Pinned vue_a_charges_execution.go maps engine-1 event116 to it. Canonical parsing
now supports that branch with recorded engine context. Unknown context still
stops after the first gate; runtime defaults cannot override engine 1. Forty-eight
new event lists cover both direction gates, reference forms, flags, point pairs
and a following zoom; all truncate at every bit. The independent Go trace covers
the orientation subgrammar, while the event composition follows the documented
writer call graph. A synthetic complete Film verifies entity/control continuation.
There is still no captured engine-1 film; this evidence does not pretend otherwise.

The full goal remains active. Version admission, other shared/snapshot branches,
canonical player and other packet bodies, runtime capability comparison and the
historical device mismatch still require completion evidence. The old attached-
position and engine-1 refusal notes above are superseded by this checkpoint.


## Equipment stack count and conditional equipment readers

Corrected `equipment-tracked-object-handles-stack-component` from an inclusive
loop to exactly the recorded four-bit count. Pinned LevelUp
`equipement_porte_lecteurs.go` (blob cc09fe6b2d984591366004954d3fb643eb05cad5)
attributes this to FUN_140f72dec: the pointer advances before the comparison with
+0x544 + (n+1)*8. This independently resolves the libfilmshell disagreement.
The original-reader exporter agrees with all 128 stack bodies, including counts
0–15 and empty stacks. A regression run against the old production reader failed
at the first empty stack: it consumed the following component's first bit.

Also verified 42 activation, 160 control-signal and 36 command-tick bodies against
unchanged libfilmshell. These existing local readers needed no production
change. Both snapshot and delta modes, absent/present handles, generation zero,
all scalar values and tick presence masks are exercised. The existing nine
component tests now apply source/raw-field checks, every-bit truncation and
DELTA/keyframe continuation to all 366 new bodies. The suite totals 1,523 cases
across 157 aliases; the prior 1,157 cases/traces are unchanged. The 129 aliases
outside this fixture set still require an audit of their available evidence.

The production fix remains inside the existing components/equipment.rs module.
No runtime handle interpretation or model organization changed. The full goal
remains active: remaining shared/snapshot branches and version contexts,
canonical player/other packet bodies, runtime capability comparison and the
historical device-boundary mismatch still require completion evidence.


Next shared-reader audit: unit lists, malleable properties and weapon-state
branches remain outside this component fixture set. Preserve the already
implemented extended `unit-command-tick-component` branch: pinned libfilmshell
returns None after its extra gate, whereas pinned LevelUp `unit_weaponstate.go`
attributes the local one/two optional ticks and conditional two-bit tail to
FUN_1406CFB28. Upstream's refusal is not evidence that the local source-backed
extension should be removed. This inspection is a next-test lead, not a new
branch-coverage claim. All 279 non-test pinned grammar Go files are now locally
available, with directory structure and individual Git blobs verified, at
/private/tmp/halo-levelup-pinned-grammar for the remaining audit.


Final equipment checkpoint: both feature configurations pass 246 tests and 10
doctests; strict Clippy/Rustdoc, WASM, formatting/whitespace and all 45 hashes pass.
Source retention passes 32 films / 403,465 packets (201.18 seconds), summary
semantics passes 3,667 events, and replay provenance remains 96/246 assumed
lists with canonical reads unchanged (52.71 seconds). Changed downstream control
counts and the unchanged historical mismatch are recorded in VALIDATION.md.
The broader goal remains active.


## Canonical actor positions and shared unit/weapon readers

Replaced both actor-state non-object target branches' legacy R(16) skip with the
canonical point grammar. Pinned LevelUp identifies FUN_14076e494's 0x10 argument
as a precision level and explicitly calls its old R(16) behavior a dated
compatibility exception. libfilmshell independently traces the actual default/
indexed delta and raw snapshot point layouts. The reader now propagates missing
position context as Unsupported, preserving the known prefix. Recorded row-
version thresholds for the state header remain unchanged; upstream's fixed
12-bit implementation is not adopted for lower rows.

Actor-control action vector mode 0 now also honors the enclosing snapshot mode.
It reads three raw float words in snapshots, the canonical point grammar in
ordinary deltas, and preserves explicit full-precision non-snapshot context.
Default-box points no longer require external map widths; indexed points still
refuse after their recorded gate. Other action modes/tails and runtime
interpretation are unchanged. Production edits stay in components/control.rs
and its existing unit.rs dispatch.

Added 930 independently traced cases across 23 shared aliases: 122 actor-state,
200 actor-control, and 608 equipment/unit/property/weapon cases. This includes
52 weapon type-info bodies with frame-configuration and magazine boundaries.
All prior 1,523 cases/traces are unchanged. The suite totals 2,453 cases / 180
aliases; the inventory lists the 106 aliases outside this fixture set. Every-bit
truncation, exact fields/raw bits, serialization, actual DELTA/keyframe following
boundaries and missing-context tests apply to all these cases. Two pre-fix runs
fail the new actor-state and snapshot-action continuation tests; the corrected
readers agree with unchanged upstream traces. Fixture README pins the relevant
LevelUp source blobs and documents row/combination coverage limits.

No full-parity claim follows from these counts. Remaining object/biped/device/
manager/projectile/tacmap branches, the existing extended unit-command-tick
source suite, wider version contexts, canonical player/other packet bodies,
runtime capability comparison and the historical device mismatch remain open.


Next concrete position lead: tacmap-waypointstate still invokes the legacy
traversal body and ignores the row-version tail. Pinned libfilmshell has an
explicit raw snapshot form, while LevelUp's exception notes identify
FUN_140f04d88 / FUN_14076e524 and the additional bit when param_4 > 1. The same
notes distinguish canonical game behavior from the dated delta compatibility
reader. Audit this and the other tacmap position callers next; do not infer
canonical fidelity merely because old fixtures match the compatibility body.


Final unit/weapon checkpoint: both configurations pass 246 tests + 10 doctests;
strict Clippy/Rustdoc, WASM, formatting/whitespace and 45 fixture hashes pass.
Source retention passes 32 films / 403,465 packets (206.38 seconds), all 3,667
summary events still match, and replay provenance remains 96/246 assumed lists
with canonical reads unchanged (50.81 seconds). The original device-boundary
oracle failure is unchanged. Full verification and changed control counts are
recorded in VALIDATION.md. No production edits followed these checks.


## Canonical tacmap positions and waypoint tails

Corrected the five position sites identified by the pinned game-call analysis:
waypoints, co-op tether areas, display assets, areas of interest and POI icons.
They now read raw snapshot words or default/indexed delta points at precision
level 16. Waypoints also read the extra bit when row version > 1. Preceding
fields remain readable without map context; only an indexed point requires it.
Changes stay in the existing tacmap.rs and scene.rs families. No runtime recovery
or semantic projection is added.

Added 270 original-libfilmshell cases across seven tacmap aliases, including all
LOD counts and selected 32-player handle masks. All prior 2,453 case/trace
objects are unchanged. The main suite totals 2,723 cases / 187 aliases, leaving
99 upstream aliases outside this particular fixture set. Separately, 81 cases
cover the three aliases absent upstream: 27 full snapshots execute the original
LevelUp functions with blob verification, and 54 delta envelopes are composed
from documented game calls with the actual point bytes/boundaries compared to
original-libfilmshell tether traces. No full-body upstream parity is claimed
for those three aliases. Every case has source-field, truncation and real
DELTA/keyframe following-boundary checks; missing-context checks cover all five
position sites. The old tether reader demonstrably ended 48 bits early in the
new default-box regression (125 vs 173).

The snapshot test's hardcoded second-axis NaN assumption was generalized to
compare every field with its own source. Original fixture values are unchanged;
the newer zero, negative-zero, NaN and all-ones words remain exact. Reproduction
and independent-source limitations are detailed in the fixture README.

The full goal remains active. Object/biped/device/manager/projectile branches,
remaining point sites and row contexts, canonical player/packet bodies and
runtime capability comparison still need evidence. The historical expected
endpoint is now localized in the local decode: device-position-group-component
begins at bit 382 immediately after device-position-component. The local reader
consumes its 32-bit word and absent eight-bit gate through bit 415. Compare the
original historical reader's admission there and independently validate the
remaining device record before drawing a conclusion or changing any oracle.


Historical admission follow-up: inspected all five dispatch-chain source files
at the oracle's exact d61443ef59268ad734355db8e9974f68db5ca6d0 revision, retained
under /private/tmp/halo-historical-dispatch. The chain implements
`device-position-component` but has no `device-position-group-component` arm;
it reaches `consumeComposantsVueBM4b`'s `return variant, nil, false` default.
The old source therefore refuses that component before reading a body. This
explains admission at the old endpoint, but does not independently validate the
newly consumed device fields or later records, and the historical oracle has
not been changed or waived. Next audit should compare the full device record
with a source that implements it, preserving the old refusal as historical
evidence. The direct pinned source downloads have these Git blob hashes:

- dispatch_object.go: 048fe41cf002f98d3dd88a800584148d0cafc666
- dispatch_item.go: 2cb8ec06c0cea7131c118a3429584069ca852c96
- dispatch_player.go: f2006520fb534ddb7e6f6a4894d5f21cb5852874
- dispatch_biped.go: cc681da62be6dcc4c0f744dea8063a76f0e337f6
- composants_vue_b_m4b.go: ff5b8c3dfa574547237dddfa82b9080a4a5e1edf


Final tacmap checkpoint: both configurations pass 247 tests + 10 doctests;
strict Clippy/Rustdoc, WASM, formatting/whitespace and 47 fixture hashes pass.
Source retention passes 32 films / 403,465 packets (280.57 seconds), summary
semantics still matches 3,667 events, and replay provenance remains 96/246 assumed
lists with canonical reads unchanged (58.34 seconds). Control counts increased
substantially; their exact values and limits are recorded in VALIDATION.md.
The historical oracle failure is unchanged. The full goal remains active.


## Device branch and captured-prefix checkpoint

The pinned historical dispatch chain above refuses device-position-group at
bit 382. A new independently reproducible captured fixture executes original
libfilmshell entry readers sequentially without marker recovery, with the
recorded BuildId 269225 / HI_1_13_0 / level tag 88891201. It verifies 90 NEW records
(archetypes 9/38/43) through bit 64496, every component boundary and primitive
boundary, raw fields and the unresolved successor refusal. The first device
ends at 462. The later device beginning at 3958 ends at 4827 with recorded map
axes 15/15/17 (secondary region 18/18/18); under historical default axes
13/13/14, original upstream ends at 4520 and reports BadHeader next. Canonical
Film already binds the recorded level. No production defect was identified by
this comparison. Source hashes, reproduction and limits are in
[the captured fixture README](../fixtures/device-record-v41/README.md).

The old historical oracle is neither rewritten nor waived. Its absent dispatch
and default context explain this particular disagreement only. The new local
regression uses canonical traversal with no recovery and no controls after
rejection; it retains the known successor header before refusing the body, so
its final end is intentionally beyond upstream's pre-header refusal position.

Added 452 original-reader cases for 23 device aliases (snapshot and delta),
covering fixed words, groups, handle widths/generations, monitor count bounds,
layer masks/weights, dispenser slots and timers. All previous 2,723 case/trace
objects are unchanged. The suite now has 3,175 cases / 210 aliases; 76 aliases
are outside this suite, not necessarily unimplemented. Primitive/source,
serialization, every-bit truncation and DELTA/keyframe following-component
checks apply to all new cases. The snapshot test's vehicle-only refusal
assumption was extended to validate device monitor-count refusal. Existing
production device readers agree with upstream; no production edits were needed.

The full objective remains active: remaining shared object/biped/manager/
projectile branches, snapshot vehicle context, wider row/build/event admission,
canonical player/packet bodies and runtime capability comparison still need
requirement-specific evidence.


Device verification: both feature configurations pass 248 tests + 10 doctests;
strict Clippy/Rustdoc, WASM, formatting/whitespace and 49 fixture hashes pass.
Both original upstream exports and the authored generator reproduce exactly.
The captured regression covers 775 components / 3,846 primitive reads. No
production behavior changed at this checkpoint; the preceding tacmap corpus
results remain the latest full corpus evidence. The historical oracle remains
unchanged, with the specific discrepancy explained rather than waived.


## Rigid-body point grammar and projectile branch checkpoint

Corrected generic-rigid-body-transforms in the existing physics.rs family.
FUN_142f036f0's precision-16 point call requires raw snapshot words or a delta's
default/indexed box. The old reader always used traversal widths and refused
before the mask without map context. It now retains the mask/orientation and
only requires context at an indexed point. This follows original libfilmshell
and instruction-addressed LevelUp evidence (lecteur_position_exceptions.go,
blob 699712aa7591344f6b60831506ce1e74e8c14f78, call 142f03837), not that file's
dated traversal-width compatibility reader.

Added 594 original-reader cases for 15 aliases: 432 rigid-body combinations
and 162 velocity, orientation, projectile, item, playback, physics-state and
animated-mesh cases. All prior 3,175 case/trace objects remain unchanged. The
old reader demonstrably ends a new eight-body default-point case at bit 312
instead of 616. The expanded suite totals 3,769 cases / 225 aliases; 61 aliases
have no cases in this suite, not necessarily no implementation. Every-bit
truncation, source/primitive boundaries and DELTA/keyframe following-component
checks cover every new case. Missing-context tests now cover rigid-body masks,
raw/default points and refusal after a recorded indexed gate.

The non-wire orientation validity check described by LevelUp (FUN_140501798)
remains outside this structural reader's validation; agreement on arbitrary
raw codes is not proof of physically valid transforms. The other shared
projectile component branches tested here already agree with upstream. Wider
projectile event bodies, remaining shared object/biped/manager branches,
version/context admission, canonical player/packet bodies and runtime capability
comparison remain open. The full objective is still active.


Next shared-object audit candidate: original libfilmshell DamageSections refuses
a six-bit count > 32 before entries; object.rs currently reads all counts 0–63.
Pinned LevelUp components_object_state.go (blob
a2779aef639040593435d32694e114753051ffd6) confirms the damage-section widths but
its reader likewise has no count check. Its adjacent RegionState notes explicitly
say the game rejects counts >32, whereas original libfilmshell's RegionState
also reads all counts. Establish source-specific validity/refusal rules and
captured effects before promoting either historical consumption policy to the
canonical parser. Add count-limit and following-boundary cases as part of that
audit; a dispatch match alone does not settle this discrepancy. UnitCommandTick's
extra branch is another deliberate source divergence: upstream refuses after
the extra gate while local code reads the documented one/two-tick tail. Preserve
that richer grammar unless independent evidence contradicts it.


Rigid-body verification: both configurations pass 248 tests + 10 doctests;
strict Clippy/Rustdoc, WASM, formatting/whitespace and 49 fixture hashes pass.
The generator and original upstream export reproduce byte-for-byte. Full source
coverage passes 32 films / 403,465 packets (280.21 seconds). Complete controls
are [676959, 592, 499], secondary blocks 439; the changed counts are traversal
measurements, not semantic correctness proof. Summary semantics still matches
3,667 events, replay provenance/canonical invariance passes, and the historical
oracle discrepancy remains unchanged at (3, 4603) versus (1, 382). Detailed
results and limits are in VALIDATION.md. The full objective remains active.


## Shared object-state and damage-count checkpoint

Added 2,462 original-libfilmshell traces for 13 shared object aliases. The main
suite now contains 6,231 cases / 238 aliases, preserving all prior 3,769 objects
and traces. Forty-eight aliases remain outside this suite. New tests cover
parent row/archetype branches, dead-state optional/velocity/biped tails,
low-frequency masks/counts/tails, frame-configuration gates, vitality, dissolver,
scale, constraints, legal regions and damage-section count boundaries.

Corrected object-damage-sections in object.rs to retain and refuse a six-bit
count >32 before reading entries, matching original libfilmshell. The old reader
returned Truncated instead of Unsupported on the independent count-33 body;
new DELTA/keyframe tests also prove the following component remains unread.
Every case has primitive/raw-field/serialization, every-bit truncation and
following-component checks. Other tested object-state readers already agree.

The cap is explicitly an upstream robustness rule. LevelUp
components_object_state.go (blob a2779aef639040593435d32694e114753051ffd6) confirms
the body widths but does not supply an independent damage-count rejection
instruction. Its separate region-state validity note remains unresolved: both
reference readers consume counts >32 despite the documented game rejection.
Only legal region counts are certified by this new suite. This distinction and
other limitations are retained in the fixture README; the full 1:1 goal is not
complete. Remaining biped/manager branches, validity and version/context rules,
canonical player/packet bodies and runtime capability comparison remain open.


Next biped audit has a confirmed source lead: ability.rs::non_predicted still
uses the former measured two-branch anchor grammar (inner 1/2, a zero-only
three-bit prefix and ungated map axes). Pinned components_biped_anchor.go now
(blob 52551f8ad7a87851892d2d795ad37a098a8d59c6) documents FUN_142f25e90
and its writer with all eight encoded labels, category-1
optional reference, attached/default/indexed/raw point forms, six flag bits,
and the label-specific handle/vector tails. The source explicitly identifies
how the old Zero3/Mid7 fields accidentally combined gates and flags. Compare
these complete branches and update runtime observations with exact provenance;
do not preserve those old positional assumptions as canonical wire fields.

Predicted ability is a separate source conflict: libfilmshell's Ability tag-3
branch treats its six-bit value as flags and gates extra target fields, but
components_biped_spartan.go (blob 973e443021d6048ce4569e11c33924cb23a68162)
traces FUN_140f03dfc initializing dst[2]/dst[3] to zero before the branch test.
Its consumeSpartanAbilityTag3 therefore reads only the optional R(6) and the
optional precision-16 point, matching the local body's existing structure.
Do not blindly import the upstream extra-target or row-version guards. Source-
specific traces and absent-context prefix tests are needed here, alongside the
non-predicted anchor upgrade. Mobility/posture should likewise be checked for
snapshot/default points that currently demand map context too early.


Object-state verification: both feature configurations pass 248 tests + 10
doctests. Strict Clippy/Rustdoc, WASM, formatting/whitespace, all 49 fixture
hashes, and exact generator/original-export reproduction pass. Source retention
passes 32 films / 403,465 packets (283.41 seconds); controls are
[676954, 592, 499], secondary blocks 438. These changed traversal counts are not
semantic correctness proof. All 3,667 summary events still match, replay
provenance remains 96/246 with canonical reads unchanged, and the historical
appearance oracle remains (3, 4603) versus (1, 382). The full objective remains
active; next prioritize the source-confirmed non-predicted ability grammar above.


## Non-predicted ability anchor upgrade

Replaced ability.rs::non_predicted's old two-branch measured grammar with the
complete FUN_142f25e90 reader/writer layout. It reads all eight encoded labels,
a category-1 reference and attached/raw/default/indexed place, six flag bits,
and the per-label handle/vector/point/tail fields. Labels 7/8 are valid empty
variant bodies; label 6 has no nine-bit tail. Row version >1 still adds the
outer three-bit tail. Existing explicit legacy body-disable configuration is
preserved; canonical configuration enables the body.

The old zero-only prefix and Mid7 combined unrelated gates/fields. Internal
AbilityNonPredictedState now retains exact reference values and source ranges
for position forms, optional flags and variant fields, with unknown values
kept distinct from zero. No runtime code consumes this callback today; no new
semantic grappling event/trajectory is inferred. The raw canonical fields stay
in the existing film/parser/runtime organization.

Added 920 original-reader cases and independently authored observation values.
All prior 6,231 cases/traces are unchanged. The main suite totals 7,151 cases /
239 aliases; 47 aliases remain outside this suite. Every new case has primitive,
raw-field, serialization, every-bit truncation and DELTA/keyframe continuation
checks. A new observation test validates all 920 publications both with and
without map context, including a complete attached prefix followed by an
unavailable indexed destination. The old code demonstrably refused the new
label-1 default-point case. Upstream traces and the instruction-addressed source
blobs in the fixture README support the correction.

Predicted ability remains unchanged where its source analysis contradicts
libfilmshell's additional target fields. Mobility/posture point-context handling,
other biped/manager branches, invalid-region semantics, broader build/version
admission, canonical player/packet bodies and runtime capability comparison
remain open. The full objective remains active.


Anchor verification: both feature configurations pass 249 tests + 10 doctests.
Strict Clippy/Rustdoc, WASM, formatting/whitespace, all 49 fixture hashes and
exact generator/original-export reproduction pass. Full source retention passes
32 films / 403,465 packets (282.97 seconds); complete controls are
[676954, 593, 499], secondary blocks 438. The additional control list is not
semantic correctness proof. All 3,667 summary events match; replay provenance
remains 96/246 with canonical reads unchanged. The historical appearance oracle
remains unchanged at (3, 4603) versus (1, 382). Next work remains the separate
predicted-ability source conflict, mobility/posture context admission, other
shared components and the broader packet/runtime requirements.


## Biped mobility/posture and predicted-ability point admission

Corrected both mobility positions to read raw snapshot words. Mobility, posture
and predicted ability now use the shared precision-16 point helper, accepting
raw/default-box positions without map context and preserving the prefix through
the indexed-box gate when external widths are unavailable. The mobility handle
and index gates are no longer hidden by an early context refusal. Explicit
full-precision/baseline overrides and legacy mobility body-disable policy remain
supported. Existing film/parser/v41/runtime organization is unchanged.

Added 1,708 original-libfilmshell cases across biped control, desired sets,
properties, energy, slide, mobility and posture branches. All previous 7,151
case/trace objects are unchanged. The main suite totals 8,859 cases /254 aliases;
32 upstream aliases remain outside it. A pre-fix test demonstrated mobility
snapshot under-reading (450 versus 525 bits). All new cases have primitive,
raw-field, serialization, every-bit truncation and DELTA/keyframe continuation
checks; applicable point cases also verify precise absent-context refusal.

Predicted ability's existing tag-3 structure is retained on source evidence,
with point admission fixed. Separate 140-case source fixtures cover versions
0/1/2/4, optional six-bit values including low-bit/bit-4 combinations, and all
point forms. The exporter verifies two pinned LevelUp Git blobs and extracts
unchanged predicted-ability/point functions. Fixed external table widths are
injected explicitly. This certifies branch composition under that context,
not the level catalogue or execution of the game binary. The initializer clears
dst[2]/dst[3], contradicting libfilmshell's extra-target branch. Neither those
extra fields nor its row-version restriction are imported. Precision override
tests also protect the distinct control-context gate (full_precision alone).

Other manager/player/stat branches, invalid-region admission, broader build/
version rules, canonical player/packet bodies and runtime comparison remain
open. Next audit leads: music-state differs between libfilmshell's two optional
32-bit words and pinned LevelUp dispatch_player.go's two one-bit flags; local
scene.rs agrees with the latter. Treat this as another source conflict, not an
automatic port (dispatch_player.go blob 3e58e6eb74179f6f69c96e20592b1dfb2c98d2dc).
Managed timer masks are read as two 32-bit primitives upstream but one 64-bit
local field. The pinned components_moteur_de_partie.go (blob
4b6f127a8941c9f16097c7d512c2ec08b49e16c5) explicitly specifies R64 and increasing
slot order, so this is primitive grouping, not proof of a wire error. Preserve
that distinction when adding source/primitive comparisons for the remaining
component cases; inspect the four-field timer helper for exact value order.


Final biped source retention passes 32 films / 403,465 frame packets
(294.96 seconds under concurrent verification). Complete control kinds 0/1/2
remain [676954, 593, 499], secondary blocks 438. The final component
suite also passes all 23 tests after the lint-only conditional cleanup. These
checks preserve raw source coverage; they do not establish complete semantic
format coverage or resolve the unchanged historical appearance oracle.


## Manager, player, stat, text and source-conflict checkpoint

Added 3,152 original-libfilmshell cases, retaining all previous 8,859 case/trace
objects. The main suite totals 12,011 cases /284 aliases. Added 4,024 independent
source cases across alliances, timers, music, static splash and multiplayer TLV.
All 286 upstream aliases now have at least some fixture evidence. This closes
the alias inventory gap; it does not certify every branch or semantic behavior.
The source exporter verifies five pinned LevelUp blobs, extracts original
functions/constants and unchanged dispatch arms, and supplies bounded bit I/O.

Source traces caught the four-field timer's misplaced five-bit state at
primitive boundary103. Corrected basic.rs to read R16/R16/R5/R16, with authored
per-slot values verifying order. The 64-bit timer mask and 32-bit alliance mask
remain whole source fields. Upstream splits those into smaller primitives;
source tests preserve our canonical grouping instead of weakening trace checks.

Music and splash retain their existing source-supported grammar. Libfilmshell
adds two optional music words and a preceding splash kind4 not present in the
pinned source. TLV also retains its source-described variant-zero submessage
and field traversal; upstream's length-only interpretation conflicts with it.
The fixture README records the exact sources and limits of this evidence.

TLV resource limits now stop with Unsupported at the retained prefix instead
of silently truncating declared lengths/counts, omitting a nonempty deep scope,
or claiming completion after exhausting the field cap without a terminator.
Ten-byte overflowing/nonterminating varints stop after the offending byte.
These explicit representation/resource limits cannot be overridden by fallback
stubs. Twenty small DELTA/snapshot rejection cases receive every-bit truncation
and following-component tests; separate tests exercise exhausted message fields,
legal maximum counts/u64, and empty lists at the depth limit. The existing legal
field grammar is compared to unchanged source traces; malformed-limit policy
is explicitly a local safeguard, not claimed as independent game-disassembly proof.

All new main/source cases pass raw-field/serialization, primitive-boundary,
every-bit truncation and DELTA/keyframe following-component checks. Main filters
refuse tag15 without consuming a successor. No runtime semantic projection was
added. Remaining work is the broader event/build/packet/default-state and
runtime capability audit, plus the transport framing differences in this ledger.


Final manager/TLV verification: both feature configurations pass 253 tests +10
doctests; strict Clippy/Rustdoc, WASM, formatting/whitespace, all 53 fixture hashes
and exact generator/original-export reproduction pass. Source retention passes
32 films / 403,465 packets (291.59 seconds). Complete control kinds 0/1/2
are [676954, 593, 499], secondary blocks 438. Summary events remain 3,667;
replay remains 96/246 assumptions with canonical reads unchanged. The historical
appearance oracle remains (3, 4603) versus (1, 382). These results certify this
checkpoint, not completion of the full canonical format or capability audit.

## Event flags, projectile source traces and runtime capability audit

Extended the main event suite from 803 to 1,494 cases without changing any of
the preceding case/trace objects. The additions cover all 13 empty-body codes,
damage-section response, throw and AI dialogue gates, and 568 damage-aftermath
bodies. Damage flag boundaries failed at bit51 before the reader was corrected
to retain fourteen separate flags ahead of the final flag/optional-word gate.
This improves partial-field retention without changing complete event widths.

Added 144 original-source projectile bodies/traces. Blob-checked unchanged
LevelUp readers confirm the conditional 19-bit direction tail as well as the
eight-bit form; libfilmshell hardcodes eight. These cases retain the disagreement
explicitly and join the every-bit truncation, raw-field/serde, missing-map,
runtime point and event/entity/control continuation tests. External delta/table
context is explicit. Snapshot/full-precision event admission remains unproven.

The initial [capability audit](LIBFILMSHELL_CAPABILITIES.md) maps the public
upstream runtime APIs and game catalogues to local ownership. Raw components and
existing player/projectile series are distinct from the missing typed statborg,
vehicle/ride, stationary/ground-object and zone projections. Upstream scans,
nearest-object ownership, adjusted timestamps and catalogue defaults remain
interpretations. Field-level runtime parity and broader wire/context decoding
are still required; alias/dispatch coverage cannot close the goal.

Both feature configurations pass 254 tests and ten doctests. Strict
Clippy/Rustdoc, WASM, formatting/whitespace, all 55 fixture hashes and exact
generator/exporter reproduction pass. Summary semantics remain 3,667 events;
replay assumptions remain 96/246 with canonical reads unchanged. The unchanged
historical appearance oracle remains (3, 4603) versus (1, 382) at payload567583.
Source retention passes all 32 films /403,465 frame packets (293.33 seconds).
Complete control kinds 0/1/2 remain [676954, 593, 499], secondary blocks 438.
No production edits followed verification. The full objective remains active.

## Source-linked statborg runtime history

Added `runtime/scoreboard` and `TheaterRuntime::scoreboard()`, leaving canonical
models and parser grammar unchanged. The v41/27 version-1 statborg schema now
has projected identities, 56 current values and their gated extras, finalized
round values and outcome codes. Integers retain all 8/16/32/64 bits, initial
zeros and repeated observations. Named conventional slots are available beside
numeric slot access. Every projection retains its canonical record/component
source, and timestamp queries do not move the playback cursor.

Owner halves follow the pinned statlog mapping without later-snapshot/holder
inference. Unknown or duplicate owners, identity changes, generation reuse,
partial rows and baseline replacement remain explicit. Unread packet regions
produce gap markers after their established prefixes; queries invalidate state
through these gaps until ownership and values are reestablished. This exposes
existing canonical stops rather than concealing them in a final-score estimate.

Seven ordinary tests cover actual Film keyframe/DELTA bytes, source links,
ownership and lifecycle boundaries, signed extremes, malformed/schema refusal,
and gaps at equal timestamps. All 216 independent version-1 stat fixtures project
back to their source values. The new captured test passes for Octagon (1,160
updates /64,616 integers /35,879 gaps) and Recharge (12,128 updates /214,517
integers /137,892 gaps). These are observation counts, not full match coverage.

Strict Clippy/Rustdoc, WASM, formatting/whitespace and all 55 unchanged fixture
hashes pass. Summary semantics retain 3,667 events; replay assumptions retain
96/246 with canonical reads unchanged. The historical appearance oracle remains
(3, 4603) versus (1, 382) at payload567583. Both feature configurations pass 261
ordinary tests and ten doctests, with eight corpus tests ordinarily ignored.
Source retention passes all 32 films /403,465 packets (299.79 seconds).
Complete control kinds 0/1/2 remain [676954, 593, 499], secondary blocks 438.
No production edits followed the final regression run.

See [SCOREBOARD.md](SCOREBOARD.md) for the API and evidence boundaries. Match-clock,
engine-state/lives-left series, roster-holder linkage and final-score recovery
remain pending, along with the broader capability ledger. No completed-scoreboard
or full-format parity claim is made. The goal remains active.

Next engine-state pointers confirmed against the retained registry: archetype2
rows2/5 are current-state and round-timer; archetype5 rows1/14/18 are respawn
timer, lives remaining and active-in-game, all version1. The canonical timer
already retains two 16-bit words plus a five-bit state. Before adopting the
upstream linear seconds conversion, compare pinned LevelUp
`vitality.go::decodeGameEngineRoundTimer` / `DequantEndpoint`: the original
source uses explicit endpoint flags. Engine-player identity comes from the
default/creation body, not the statborg entry word; preserve that distinction.

## Engine clock and player-state history

Added `runtime/scoreboard/engine` behind `scoreboard().engine()`, preserving
film/parser/v41/runtime ownership. The view independently admits named version-1
engine/player rows in v41/27, even when statborg is unavailable. It retains
phase, recorded round absence, both round-timer quanta and state, raw respawn
integers, lives code and active state. Every row has a canonical source;
engine-player identity requires the complete current default-state body.

Queries preserve the scoreboard lifecycle/gap rules: NEW/keyframes replace
rows and identity, stale-generation deletes do not remove newer state, partial
rows invalidate values, and competing identities/engines remain ambiguous.
Shared gap markers take effect after the known prefix and before later records
at the same timestamp. Later DELTAs cannot recover player ownership lost at a
gap. Neither roster identity nor elapsed/remaining timer roles are inferred.

The timer conversion uses exact endpoints and 36,000/65,534 interior steps,
not libfilmshell's linear 65,535 divisor. An independent exporter checks three
pinned LevelUp blobs and extracts the unchanged timer reader/helper/quantizer.
It exercises all 65,536 quanta in both words and all 32 tail states. ARM64 Go
FMA contraction initially disagreed with separate float32 operations for 21,018
quanta, beginning at 8. The exporter now pins amd64/v1 to retain scalar
rounding; the Rust projection matches all retained reference words exactly.
The new binary fixture is documented in `fixtures/engine-v41/README.md`.

Six new tests cover actual Film keyframe/NEW/DELTA/deletion packets, source
pointers, both default-version gates, slot boundaries, missing/changed/duplicate
identity, lives limits, absent round, partial baselines, unsupported schemas,
malformed fields and gap ordering. The captured check adds 6,624 engine updates,
12,112 source rows and 2,656 complete-default identities across Octagon and
Recharge (37.27 seconds under concurrent verification). Existing statborg
counts remain 13,288 updates and 279,133 exact integers. These observations do
not certify complete match coverage. All 56 fixture hashes and exact timer
export reproduction pass.

This checkpoint changes runtime projection only. The historical appearance
boundary remains an unresolved item from prior verification; its oracle was
not rewritten. Roster-holder linkage, direct typed events, final-score recovery,
vehicle/ride/world/zone projections, broader catalogues and wire/build/context
admission remain. The full canonical-parser goal is active, not complete.

Final engine checkpoint verification: both feature configurations pass 267
ordinary tests and ten doctests. Strict Clippy/Rustdoc, WASM, formatting and
whitespace pass. The optimized release build also agrees with every timer
oracle word. All 56 hashes and exact exporter reproduction pass. The larger
corpus results above remain from the preceding checkpoint; this runtime-only
change reran the extended captured engine/statborg check. Details and logs are
in VALIDATION.md. No commit or push was made; the broad goal remains active.

## Direct event-body interpretations

Added `TheaterRuntime::actions(ActionFilter)` and `runtime/actions`, preserving
film/parser/v41/runtime ownership. This view borrows the existing indexed event
and canonical record, interpreting fields on demand with their original packet
time and order. It covers all 18 typed libfilmshell `events::Body` families plus
the source-supported translocator event. This is distinct from the derived
`model::Event` stream: no roster attribution, fatal-weapon inference, adjusted
headshot timestamp, damage estimate or nearby-equipment ownership is imported.

Every scalar carries its exact value, width and source-field index. References
retain domain, conditional narrowing, relative index and generation. Points
retain their region/default-box form, codes and widths. Game messages keep
ordered duplicate keys, exact text bytes/terminators and all typed parameters;
24-bit and 32-bit zero words remain distinguishable. Kill percentage words are
not clamped. Unnamed tails have explicit field ranges, and complete unsupported
semantic bodies remain Other. Incomplete records stay visible with errors.
No packet scan, reparse or inferred event boundary is used.

An independent exporter serializes unchanged libfilmshell reader results at the
pinned commit. It covers all 1,473 existing complete main fixtures plus two
synthetic kill boundary cases. Exact comparison spans all 18 upstream body
families and references, normalizing only deliberate representation differences
for the comparison (for example upstream string padding and point scaling).
The local values retain their original source representation. Separate original
LevelUp projectile and engine-1 teleport cases protect the existing disagreements.

Six ordinary tests cover the differential values, source fields, truncated and
malformed reads, missing kill context, exact strings/parameters/duplicate keys,
public time/code filters and unchanged Film/cursor. A new captured check passes
in 29.49 seconds: Octagon has 2,084 complete /96 incomplete records and 58,398
source scalars; Recharge has 28,626 /246 and 543,432. Complete counts include
Other bodies and do not mean all event semantics or whole matches are decoded.
All 58 fixture hashes and exact semantic-export reproduction pass.

See ACTIONS.md for the API and boundaries. The remaining public-event work is
roster/holder attribution, mode/HUD/flag/zone semantics and explicitly labeled
inferences. Broader packet/build/snapshot, world/vehicle and catalogue gaps
remain in the capability table; the full goal is active.

Next holder audit pointers: upstream `holders.rs::api` defaults an unmapped
engine id to the same roster index. `roster.rs::struct_xuids` scans all eight
bit shifts in type-8 payloads; its joiner takeover loop selects the first bot
whose departure is within TAKEOVER_TICKS. These are derived policies, not
canonical slot/ownership evidence. A local bridge must independently establish
its initial mapping and preserve competing takeover candidates rather than
silently importing either default or first-match behavior.

Final direct-event checkpoint: both feature configurations pass 273 ordinary
tests and ten doctests, with nine normally ignored corpus tests. Strict
Clippy/Rustdoc, WASM, formatting/whitespace, all 58 fixture hashes and exact
semantic-export reproduction pass. The new captured check is described above;
legacy corpus checkpoints were not rerun for this runtime-only projection.
No commit or push was made. The broad goal remains active.


## Roster and bot packet checkpoint

Canonical types 8 and 12 now own `PlayerMetadataRead`, using the shared player
body from pinned LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`. This replaces
the unit roster placeholder and opaque bot dispatch without importing libfilmshell
name/identity scans. Recorded format gates and build-specific personalization
widths control traversal; unknown contexts, count limits, truncation and excessive
bot tails are explicit stops. Original scalar/blob bits, UTF-16 units, mismatched
identity keys, absolute bot indexes and packet timestamps remain unchanged.

Thirty independent original-Go cases cover seven profiles, both option gates,
pre-21 format, maximum variable fields and failure/tail behavior. Every-byte
truncation tests preserve established fields and the first incomplete entry.
All 1,018 type-8/type-12 packets across the 32 retained films agree with the pinned
Go readers: 509 complete roster packets /5,347 entries, and 509 complete **empty**
bot packets. All are HI_1_13_0. Nonempty bots and other build profiles have authored
source-fixture evidence only, not captured-film validation in this corpus.

Decoded bot metadata retains an explicit scoreboard ownership gap, including
empty lists, until runtime population linkage is implemented. Incomplete roster
packets now create an explicit gap as well. The existing replay roster consumer
still reparses bytes, imposes stronger tail rules and assumes the option is an
index; promoting it to canonical-record consumption with proven ownership remains
pending. No canonical option-to-slot equality or proximity-based takeover was
introduced. Team byte offsets are source-profile evidence, retained raw here.

See ROSTER.md, the roster fixture README and VALIDATION.md for scope and checks.
This is verified parser progress, not complete roster/runtime or film parity.

Checkpoint validation: both feature configurations pass 278 tests and ten
doctests; strict Clippy/Rustdoc, WASM, formatting, whitespace and 59 fixture hashes
pass. Full source coverage passes 32 films /403,465 frame packets; captured
scoreboard, engine, replay and summary regressions pass. The historical appearance
oracle still fails at the same payload/values; no expectation was rewritten.

## Runtime roster population and replay adoption

`TheaterRuntime::roster()` now projects canonical session/bot listings, retaining
packet events, field pointers, exact UTF-16 units and separate identity domains.
A private event-offset index avoids scanning frame events for each population
query. Complete lists can establish recorded presence or absence; partial,
opaque, malformed, duplicate or conflicting lists produce unknown population.
Repeated and same-time listings remain ordered, and history retains reappearance
without inventing a departure time between observations. Bootstrap identities
remain separate rather than being backdated to packet times.

Bot team/mirror bytes now have direct runtime projections at the pinned build
profile offsets. Unknown locations stay absent, signed values remain exact and
byte disagreements are not normalized. The existing 30 original-Go cases provide
35 complete entry projections, including deliberately invalid UTF-16 and differing
identity words. Runtime membership refuses those conflicting human keys while
retaining their canonical source records.

Replay now consumes these canonical roster fields instead of its duplicate
SlotReader path. It accepts the native type-8 tail rule. The remaining legacy
option-to-engine-index hypothesis is recorded for every use in
`ReplayDiagnostics::roster_index_assumptions`: relative time, exact entry span,
option field bit, assumed index and XUID. This supersedes the preceding checkpoint's
note that replay still reparses roster payloads. Different-XUID index reuse,
bootstrap-to-engine ownership, bot takeover and final-score holder attribution
are still unfinished; no default index-equals-owner query was added.

Synthetic population tests cover partial/opaque lists, conflicting/duplicate
identities, independent domains, exact same-time ordering, reappearance, missing
options, malformed public fields and immutable Film/cursor behavior. Captured
Octagon and Recharge replay verifies source-linked index assumptions and original
canonical event reads. See ROSTER.md and the latest VALIDATION.md checkpoint.

Final validation for this runtime checkpoint: 282 ordinary tests and ten doctests
pass in both configurations; ten corpus tests remain ignored by default. The
indexed captured replay test passes (48/496 roster member observations and
42/496 explicit replay-index assumptions in Octagon/Recharge). Strict Clippy,
Rustdoc, WASM, formatting, whitespace and all 59 fixture hashes pass. No source
wire reader changed, so the prior source corpus/differential evidence remains;
the historical appearance discrepancy and wider capability gaps remain open.


## Native keyframe slot-table traversal

The canonical type-2 reader now follows the physical 8,191-entry table in pinned
LevelUp `keyframe_entrees.go`, corroborated by libfilmshell's sequential walk.
Every entry has a 108-bit header. Only the all-ones archetype skips a body; an
all-ones ID does not terminate the packet. Nonmonotonic IDs and generation zero
remain source values. Complete headers, including vacant entries, are retained
in `KeyframeTable.slots`; live bodies remain in `records`. The organization is
unchanged: models in film, traversal in parser/v41, interpretations in runtime.

The bounded body entry point preserves the entire archetype word. Exactly 8,191
entries and less than eight trailing bits establish End; missing entries remain
Truncated and extra data is TrailingData. Unsupported bodies stop at their first
unavailable field without scanning. Invalid ID domains stay recorded but do not
declare a subsequent parser schema. No new vacant-slot clearing, generation
ownership or vehicle branch assumption was introduced. Historical reference
chaining remains test-only with its old expectations; it is not the production
table algorithm. Replay diagnostics now locate the actual consumed prefix end.

Five tests cover original-Go traversal/guard results for 24 authored cases,
source header words and serialization, complete source coverage, truncation,
unsupported bodies, fixed framing and conservative schema binding after vacant
slots. The independent exporter checks three original blobs and extracts the
native table-count constant and reader functions. Default/component callbacks
panic if reached: these fixtures deliberately isolate nonpositive signed guards;
positive bodies retain their existing separate component/creation fixtures.
Synthetic scoreboard baselines now author full tables, preserving their existing
known-state assertions rather than accepting truncated snapshots as complete.

Both feature configurations pass 287 ordinary tests and ten doctests, with ten
corpus tests ignored by default. Strict Clippy/Rustdoc, WASM, formatting and
whitespace checks pass. All 60 fixture hashes match and the new Go exporter
reproduces its fixture exactly. Captured replay preserves 96/246 event assumptions
and 42/496 explicit roster-index assumptions in Octagon/Recharge. Captured
scoreboard retains 13,288 updates and 279,133 exact integers; engine retains
6,624 updates, 12,112 source rows and 2,656 default identities. Reported scoreboard
gaps drop to 35,486/110,664 from 35,879/137,892. These are observation counts,
not complete match coverage. Full-corpus results are recorded in VALIDATION.md.

The full source check passes 32 films /403,465 frame packets in 352.50 seconds.
It retains 2,068,021 vacant headers across 451 keyframe tables and reaches 35,634
live headers after vacant entries. Exactly 243 tables close; 208 remain Desync
with their nested body stops retained. Complete control counters change to
[1,063,261, 263, 243] for kinds 0/1/2, with 167 secondary blocks. These counters
describe current reachable canonical boundaries; source coverage alone is not
an independent semantic proof of every newly decoded frame.

Snapshot vehicle context, other partial bodies/builds, runtime vacant-slot and
holder semantics, world/vehicle/zone projections and the broader capability audit
remain open. The historical appearance oracle was not changed. The full goal
remains active; no commit or push was made.


## Snapshot stop audit and reference item catalogue

A snapshot-only audit reuses production Film configuration while excluding
frame packets. All 451 captured tables retain the preceding 243 complete /208
stopped result. The stops are now classified: 207 reach vehicle-type-state i33
without `vehicle+0x818`; one reaches object-damage-sections i7. Native source ties
the vehicle flag to the asset's physics type, which is not supplied by the item
catalogue. libfilmshell chooses that branch by a plausible next boundary; it
remains outside canonical parsing.

The separate Recharge snapshot refusal is independently reproduced through the
original libfilmshell walk. Both readers reach biped slot 755 at bit 226358 and
component 7 at bit 226997, where the six-bit damage-section count is 39. The original
component trace ends at 227003 after refusing the >32 count. No vehicle occurs in
this prefix, so the exporter proves no vehicle-layout probing participated. A
new fixture preserves exact component spans and source-bit coverage. This is a
shared refusal, not a missing damage-section reader, and does not certify the
semantic validity of preceding state or of the whole snapshot.

Added `film/items` beside `film/maps` and `film/weapon`, retaining all 2,001
upstream tags, 40 variant hashes, 45 parent links and 16 vehicle-part associations.
Nine object kinds preserve names, optional item classifications and power-weapon
flags. Explicit `CatalogRevision` selection carries the source commit/blob and
its unknown game build. Asset tags, variant string hashes and 64-bit weapon IDs
remain distinct; no catalogue data admits wire layouts or infers entity ownership.
See ITEMS.md for the query API and evidence boundaries.

The original-source catalogue exporter compares every entry through the original
lookup functions. The generator reproduces the pinned data with only import,
derive and format adaptations. Namespace, unknown-value, ordering, classification
and serialization tests complement the whole-table differential. The game-build
applicability of this reference and damage/sandbox/medal/option catalogues remain
open, along with typed world/vehicle timelines and the broader capability ledger.

Audit log: `/private/tmp/halo-keyframe-stops.log`; complete optional report:
`/private/tmp/halo-keyframe-stops.json`. The full objective remains active.

Final verification for this checkpoint: both feature configurations pass 290
ordinary tests and ten doctests; eleven corpus tests are ignored by default.
Strict Clippy/Rustdoc, WASM, formatting, whitespace, all 63 fixture hashes and
exact catalogue-generator /independent-exporter reproduction pass. No production
wire reader changed; no commit or push was made. The full goal remains active.


## Reference damage and sandbox catalogues

Added `film/damage` and `film/sandbox` alongside the existing item/map/weapon
catalogues. The shared explicit CatalogRevision selects all 468 damage effects,
151 firing entries, 294 projectiles and 64 vehicle render-model bounds from the
pinned source. Each data family exposes its own Git blob and the source's unknown
game build. Optional amounts and relations, unknown causes, multiple projectile
associations, signed magazine values and all damage flags remain exact. No wire
reader, runtime vitality, ammunition or damage observation uses these defaults.

The independent exporter invokes original upstream lookups and helpers and
serializes every f64 bit pattern. It checks all 32 flags on every numeric damage
entry and the fixed-amount predicate, plus reference Spartan vitality/constants.
An initial stdout-based harness lost one damage row through concurrent test-runner
output. The local 468-row comparison caught the missing row; separate export files
now prevent interleaving. Every prior reference row is unchanged, and the repaired
complete export reproduces exactly. Both data generators reproduce exactly as well.
See DAMAGE_SANDBOX.md and the fixture README for API and validation boundaries.

This supplies reference metadata for subsequent runtime work. It does not supply
the missing vehicle physics type, prove build compatibility, or establish actual
hit damage, collision geometry, current ammunition or player vitality. Medal and
option catalogues, typed world/vehicle/zone views and ownership remain unfinished.
The full canonical-parser/capability objective remains active.

Next wire audit: bootstrap retains CorruptionChecks, while production config in
`parser/v41/chunks/mod.rs` currently leaves the profile corruption flag at its
default. The frame/keyframe readers already consume configured corruption guards.
Verify native binding and incomplete-header admission before claiming support for
films with the recorded flag enabled; do not treat catalogue progress as closure
of this or other wire/context requirements.

Final verification: both feature configurations pass 292 ordinary tests and ten
doctests, with eleven corpus tests normally ignored. Strict Clippy/Rustdoc, WASM,
formatting, whitespace, all 64 fixture hashes and exact independent export/data
generation pass. No production wire or runtime interpretation changed, and no
commit or push was made. The full objective remains active.

## Recorded corruption-check propagation

The known bootstrap's `CorruptionChecks` bit now overrides the structural profile
in both Film parsing and contextual replay. `BootstrapHeaderRead::corruption_checks`
returns `Option<bool>` and preserves the difference between missing and recorded
false. Profile/map configuration cannot erase a recorded true or false value.
This closes the flag-binding gap identified in the preceding checkpoint.

Nonempty events now consume the native R1/optional R32 trailer after a complete
body. All 13 empty descriptors return before that trailer. Guard fields retain
their original values under `EventFieldStage::CorruptionCheck`; a truncated guard
keeps the established body prefix and prevents entity/control continuation.
Existing component readers consume R1/optional R32 for NEW, DELTA and snapshot
components. Positive snapshot default blocks consume an unconditional R32; NEW
defaults do not acquire that snapshot-only word. No sentinel equality rejection
or resynchronization scan was added. Runtime action projection validates the
trailer separately and excludes it from semantic payload/unnamed body tails.

An independent exporter checks six pinned native-source blobs and executes 44
cases: 32 events, four component envelopes and eight signed default-block guards.
Tests additionally cover exact truncation, malformed action trailers, missing
bootstrap flags, both profile override directions, and recognized-registry
Film/replay propagation. See fixtures/corruption-v41/README.md for the isolated
callbacks and evidence limits.

Missing or unsupported bootstrap layouts still retain the existing profile
fallback, generally disabled. The accessor does not call that recorded false;
strict missing-context admission remains a separate unfinished wire requirement.
This checkpoint does not establish broader build support, vehicle physics context,
full semantic parity or canonical encoding/round-trip proof. The organization
remains film models, parser/v41 grammar and runtime interpretations.

Final guard checkpoint validation: 297 ordinary tests and ten doctests pass in
both configurations; strict Clippy/Rustdoc, WASM, formatting, whitespace, all 65
fixture hashes and exact exporter reproduction pass. Captured replay retains its
prior canonical reads and assumption counts. All 32 films /403,465 frame packets
pass source coverage in 345.98 seconds, with unchanged snapshot/control counts.
All captured corruption flags are recorded false; enabled-policy evidence is
from independent authored native-source cases and public Film/replay integration.
No commit or push was made; the full objective remains active.

## Strict admission for missing corruption policy

This checkpoint supersedes the previous section's remaining fallback limitation.
Public Film and contextual replay now distinguish recorded true, recorded false
and unavailable corruption policy. A configured internal profile is a separate
context, used by research/fixture readers. Replacing a profile cannot erase an
already recorded value, and a missing source flag cannot inherit profile false.

A nonempty event stops after its established body and before any optional trailer
with `MissingCorruptionCheckContext`. A present NEW/DELTA/snapshot component stops
at the same boundary with a matching record stop and `ContextUnavailable` section
status. A positive snapshot default stops before either a possible sentinel or
n2, retaining its exact prefix with ContextUnavailable. No unavailable-width
field, zero sentinel or successor is invented. Earlier truncation/unsupported
body errors remain the owning stop. Empty messages and masks, vacant snapshot
entries and nonpositive default/component guards need no policy and can complete.
A recorded NEW ID/archetype still establishes its later DELTA schema; missing
guard context does not make that component state complete.

Four added tests cover original-source event prefixes, public Film admission for
truncated/unsupported bootstrap layouts, snapshot guard locality and contextual
replay without a source policy. Existing override tests now verify source policy
survives profile changes. Runtime action projection returns incomplete-record
errors for these stops. Toy scoreboard registries use explicit configured test
readers; their projection assertions are no longer presented as evidence that
public Film accepts invented bootstrap layouts. Public event tests supply a
recognized registry/recorded false flag, preserving their version/map assertions.
Captured-source tests continue to use Film::parse directly.

This closes the corruption-policy fallback gap. It does not prove all event
versions, alternate builds, vehicle physics context, semantic ownership or whole
film parity. The full objective remains active; no commit or push was made.

Next admission audit: pinned LevelUp `vue_a_versions.go` defines the 123 native
versions and rejects missing/mismatched tables; `vue_a_lecture.go` also refuses
nonempty message lists under configuration bit zero because the global reference
range is unavailable. Local event admission currently checks the fire/script
versions selectively, and category-relative references still need that configuration
rule. The upstream shorter-table/other-major prefix policy includes numbering and
closure hypotheses; do not promote those to canonical evidence. Keep exact known
prefixes when applying local admission rather than copying upstream cursor rewind.

Read-only corpus probes found all 32 recognized registries carry the same 123-word
table, exactly matching the pinned native version string. All 403,465 frame headers
have configuration one: 344,468 empty message lists and 58,997 nonempty heads.
Reports: `/private/tmp/halo-event-version-audit.json` and
`/private/tmp/halo-event-configuration-audit.json`. These probes establish corpus
coverage limits, not admission support for other tables/configurations. The new
fixed-header fixture helper still uses deliberately mixed bootstrap golden versions;
future public event-version tests must supply a native table explicitly while
retaining those raw-header fixtures for malformed/version-preservation tests.

Final missing-policy checkpoint: 301 ordinary tests and ten doctests pass in both
configurations; strict Clippy/Rustdoc, WASM, formatting, whitespace, all 65 fixture
hashes and exact source exporter reproduction pass. Captured replay retains its
canonical reads and assumption counts. Full source coverage passes 32 films and
403,465 frame packets in 345.94 seconds with unchanged snapshot/control counts.
No commit or push was made. Event-version/configuration admission and the broader
capability requirements remain open; the complete objective is still active.

## Recorded event-table and reference-configuration admission

Public Film and replay now check all 123 native event-version words from the
recognized v41/27 bootstrap. The first missing or mismatched word stops a nonempty
message after its continuation/code header, before references or body fields.
The reported event code identifies the failing table entry, even if this packet
contains a different event. Configuration zero also stops there because its
global reference range is unavailable. Empty list terminators remain readable.
No cursor rewind, successor scan or version/profile fallback was added.

This supersedes the preceding checkpoint's selective-version limitation. The
whole recorded table can admit the selected writer before unrelated later header
fields arrive; map, variant and corruption policy keep their separate stops.
Native fire-v3 selection now follows that table in both Film and replay. Internal
configured fixture readers remain distinct from public source admission.

The independent exporter verifies six pinned LevelUp blobs and reproduces 503
table classifications plus 87 event-head/list cases. Rust compares all native
versions, 371 applicable exact-table/major-41 cases, every truncated recorded table,
and all 87 frames. Four shorter-table cases are explicitly refused instead of
adopting upstream's numbering/closure hypothesis. The fixed bootstrap has no new
variable-cardinality detector; other count/major classes document the reference
reader, not claimed support for those layouts. Public tests verify exact source
preservation, stopped following views and replay defaults that cannot override
recorded admission. Malformed bootstrap goldens remain unchanged; public body
fixtures now supply the actual native table as their prerequisite.

All retained films use the native table and configuration one. Negative admission
therefore has authored original-source and public integration evidence. The
reference-range grammar for configuration zero, historical writer layouts,
snapshot asset physics context, identity ownership and complete semantic/encoding
coverage remain open. The original film / parser/v41 / runtime organization is
preserved. No commit or push was made; the full objective remains active.

Next configuration audit: pinned `varwidth.go` identifies `DAT_144706104` as the
packet's recorded flag for shared `FUN_1406d3140` component handles as well as
event references. Its comments trace type-2 header binding for format versions
above seven and unconditional delta-header binding. Local `components/reader.rs`
`Reader::handle` still selects category widths without this packet flag, and
`keyframes/slots.rs` retains the flag without passing it to body configuration.
An empty event list can still reach that shared component path. The event-head
fix above does not close this broader config-zero component/default requirement.
The pinned Go shared width reader itself assumes the active table; its parity
cannot independently certify configuration-zero bodies. Establish precise
refusal boundaries and propagation before claiming that support. Vacant slots,
empty lists/masks and other configuration-independent prefixes should survive.

A separate read-only envelope audit confirms all 451 retained type-2 packets also
record configuration one; report `/private/tmp/halo-keyframe-configuration-audit.json`.
Combined with the frame audit, the current corpus has no configuration-zero
coverage for the pending shared-handle admission work.

Final event-admission verification: 304 ordinary tests and ten doctests pass in
both feature configurations, with eleven corpus tests normally ignored. Strict
Clippy/Rustdoc, WASM, formatting, whitespace, all 66 fixture hashes and exact
native exporter reproduction pass. Captured replay retains its canonical reads
and assumption counts. Source coverage passes all 32 films /403,465 frame packets
in 339.94 seconds, with unchanged snapshot/control counts. No historical oracle
was rewritten. The shared configuration-zero handle gap described above and
broader capability requirements remain open; the full objective stays active.

## Shared handle configuration and control diagnostics

The packet's recorded configuration flag now reaches shared variable-width
handles in NEW/DELTA components/defaults, snapshot bodies and control actions.
Configuration zero retains known presence gates and the category-1 probe, then
stops before the unknown global-range value/generation. Entity and keyframe
records report UnsupportedReferenceConfiguration and mark the affected section
ContextUnavailable. No guard or successor is consumed. Vacant slots, nonpositive
snapshot guards, absent handles and empty masks remain readable; known NEW schema
declarations survive incomplete bodies without inventing complete entity state.

A follow-up call-site audit found the legacy position handle tail used explicit
precision widths outside the shared helper. That path now applies the same gate.
The fixed R15 posture primitive is separate and remains readable. Configuration
is selected per packet, without changing the inherited profile. Historical
configured readers remain test-only; the original film/parser/v41/runtime
organization is unchanged.

Replay already counted unclosed control views but did not record their reasons.
It now reports each incomplete control-view stop at its consumed-prefix end,
alongside existing event/entity/snapshot blockers. No semantic state is inferred
from the missing control suffix.

The new independent exporter verifies five pinned LevelUp blobs and runs 49
native category-table cases: all nine categories, both category-1 probes,
absent/present/direct references and minimum/maximum value/generation words.
The native Go shared reader itself assumes the active table. Configuration-zero
tests therefore certify native prefix preservation and explicit refusal, not
unknown global widths. Four tests additionally cover every handle-prefix cut,
NEW/DELTA/snapshot component envelopes and guards, legacy position/fixed-posture
paths, and public Film/replay with snapshot defaults, a NEW, a subsequent empty
DELTA and control actions. This supersedes the previous shared-handle fallback
limitation; successful global-range decoding remains unsupported.

The full objective remains active: historical writers/builds, snapshot asset
physics context, semantic ownership, missing runtime projections/catalogues and
complete canonical encoding/round-trip coverage are not closed by this change.
No commit or push was made.

Initial read-only catalogue follow-up: pinned libfilmshell has 157 medal definitions,
98 option-hash names, seven mode scripts and 104 mode-option entries. All 124
local empirical medal pairs exactly equal the same-named native medal's
(sorting_weight, index); no differences were found. Thirty-three native names
are absent from that empirical pair set. The comparison is saved at
`/private/tmp/halo-medal-catalogue-audit.json`. This supports a separately
revision-selected metadata catalogue, not a service-NameId mapping or automatic
replacement of empirical summary admission. Native personal score differs from
sorting weight (Double Kill: 50 versus 100); retain that distinction. Option
script defaults also remain separate from recorded Bond overrides. These are
concrete remaining catalogue capabilities, not completed imports.

Prepared the next catalogue oracle in `fixtures/game-catalogues-v41`: four pinned
blobs, unchanged original Rust APIs, all 157 medals /98 option hashes /seven mode
scripts /104 option rows, exact optional f64 defaults, and 261 binary hash vectors.
It reproduced exactly. At preparation time its README marked local catalogue
APIs, namespace/provenance selection and integration tests as unimplemented.
The fixture alone did not close either missing catalogue capability; the next
checkpoint records the completed imports and tests.

Final shared-reference corpus verification passed in 343.91 seconds, including
the legacy position handle gate: 32 films /403,465 frame packets, 451 snapshot
tables (243 closed /208 Desync), 2,068,021 vacant headers, 35,634 live headers
after vacancies, complete controls [1,063,261, 263, 243] and 167 secondary blocks.
The snapshot body stops and historical appearance oracle remain unresolved.

## Native medal and option catalogue checkpoint

The prepared catalogue APIs are now implemented as sibling `film/medals` and
`film/options` modules, using the existing explicit CatalogRevision selection.
All 157 medal definitions retain native indices, English/enum names, categories,
difficulty, personal score and sorting weight. Typed MedalIndex and OptionHash
retain unknown u32 values without truncation. Data provenance leaves game-build
applicability unestablished. No service-ID equivalence or summary-pair admission
is inferred from a catalogue hit.

All 98 hash/name entries, seven scripts /104 ordered options, every alias and
optional literal default match the original APIs. Numbers retain exact f64 bits;
boolean and missing/nonliteral defaults stay distinct. ASCII lowercase name hashing
and raw Murmur3 hashing are separate. Aliases absent from the original lookup
remain available in script metadata without silently extending lookup behavior.
Recorded Bond values are untouched; named variant interpretation remains pending.

Four new tests compare the independent upstream oracle and check the unchanged
runtime empirical boundary across all 65,536 possible summary byte pairs. Both
the oracle and the separate pinned-data importer reproduce exactly. See
GAME_CATALOGUES.md and VALIDATION.md for evidence and limitations. This supersedes
the pending-import status above, but not the missing runtime variant projection
or the remaining wire/context and whole-format requirements.

Validation: 312 ordinary tests and ten doctests pass in both feature configurations
(131.61 seconds all-features; 122.99 seconds no-default-features). The all-features
run preceded a slice-API-only hash-loop adaptation; final no-default, the 261 hash
vectors and strict Clippy cover that final code. Strict Rustdoc/WASM, formatting,
whitespace and all 68 fixture hashes/lengths pass. No new corpus run was needed
for these metadata-only additions: the shared-reference corpus above includes
the final packet readers, which are unchanged by this checkpoint. The full
objective remains active; no commit or push was made.

## Recorded variant runtime view

`TheaterRuntime::variant(VariantSchema::ReferenceLatest)` now borrows named
fields from the canonical, complete and unambiguous optional variant instance.
The API retains the existing film / parser/v41 / runtime organization. It does
not search for anchors, change Film, move playback, fill defaults, round floats,
deduplicate hashes or merge entries across loadouts/options. A complete message
is readable despite an unrelated later envelope-tail truncation.

The view exposes 24 settings, ten named flag bits, separate loadouts and weapon
entries, and exact eight-byte option payloads. Unknown types, missing members,
same-layer duplicate IDs and malformed shapes remain explicit. Inherited IDs
cannot supply missing derived fields. Recorded flag containers include Bond
sets as well as lists; the 32-film audit exposed that distinction and both have
independent authored/original-reader fixture evidence.

The 31 authored cases use four pinned source blobs for original native path,
value, option-payload and stat/playback projections. Native comparison is
separate from local malformed/ambiguity policies. The captured view test passes
all 32 films: 221 named settings, 325 option rows, 32 loadouts and 96 weapon
entries. Original upstream traversal matches all 325 option hashes/eight-byte
values in source order and all 32 base-structure endpoints. See VARIANT.md and
the variant-view-v41 fixture README for reproducible commands and scope.

This is not full variant or format parity. Localized text, scripts/mode mapping,
stat definitions/scoreboard columns, effective rules, broader writer/build
semantics and the earlier snapshot/runtime gaps remain open. Full ordinary
regression and strict-check results are recorded in VALIDATION.md.

Verification passes 315 ordinary tests and ten doctests in both feature
configurations (125.50 seconds all-features; 123.21 seconds no-default-features).
Twelve corpus tests are normally ignored; the new 32-film variant test was run
separately and passed. Strict Clippy/Rustdoc, WASM, formatting, whitespace and all
70 fixture lengths/hashes pass. Canonical packet readers are unchanged, so the
previous complete source-coverage run remains the packet-parser checkpoint.
The full robustness goal remains active; no commit or push was made.

## Variant stat definitions and column mapping

The existing runtime variant module now exposes all definition rows, seven
native metadata properties, source-linked column identifiers/matches and the
respawn-settings override tag. Definitions retain unsigned IDs, exact string
bytes, numeric signedness, omitted properties and original list ordinals.
Column resolution reports missing, absent, unmatched, ambiguous or incomplete
identity evidence rather than selecting the first row or substituting ID zero.
Repeated columns share match arrays; one index avoids scanning all definitions
for every column. Canonical Film and existing statborg observations are unchanged.

Sixteen additional authored cases bring variant-view fixtures to 47 trees. Two
new tests cover original native projections and source-preserving negatives,
including raw invalid UTF-8, absent/empty names, signed/unsigned attributes,
inherited/duplicate IDs and shared candidate arrays. The 32-film corpus passes
with 1,324 definitions and 130 column references. Independent upstream projection
matches every recorded property and mapping while identifying 5,876 omitted local
fields for which upstream supplies defaults. Local fields remain absent.

Stat-value presentation/aggregation, owner linkage, localized variant text,
scripts/mode inference, effective rules, broader builds and the outstanding
wire/context gaps remain open. See VARIANT.md and VALIDATION.md. The complete
robustness objective is still active; this checkpoint does not establish 1:1
whole-format parity.

Both serial feature suites pass 317 ordinary tests and ten doctests (123.58
seconds all-features; 122.67 seconds no-default-features). Strict Clippy/Rustdoc,
WASM, formatting, whitespace and all 70 fixture lengths/hashes pass. The existing
12 corpus tests remain normally ignored; the expanded 32-film variant test and
native comparison were run separately. No canonical packet reader changed.
No commit or push was made.

## Indexed variant strings

The runtime variant module now reads localized text through canonical Bond table
and offset fields, preserving the film / parser/v41 / runtime split. It retains
the raw entry flag, variable language counts/order, signed offsets, exact bytes,
and separate text/terminator source ranges. Shared and suffix offsets are valid;
negative/out-of-range offsets, absent members, wrong types, duplicate fields and
unterminated strings remain explicit. Immutable cache dependencies prevent callers
from invalidating the terminator index. Memory scales with distinct referenced
offsets, and NUL searches advance monotonically through the blob.

Fifteen new authored cases bring variant fixtures to 62 trees. Three new tests
cover original-source text/offset projections, source ranges and malformed cases.
The captured test passes all 32 films with 1,639 entries and 29,502 localized
values. Original upstream callbacks agree on every flag and offset. Original
text extraction agrees on every name/description pair, and all 1,323 upstream
label texts occur in the table. That last comparison proves availability only;
it does not equate table indices with stat slots.

The captured Alpha row has 18 identical offsets pointing at one string. It is
an indexed entry, correcting the preliminary hypothesis that it was unreferenced
blob text. Generic lookup requires no special skip or 18-language assumption.
Reference-convention English name/description helpers are available; remaining
string roles/stat-label linkage, scripts/mode semantics, effective defaults and
the earlier packet/context gaps remain open. See VARIANT.md and VALIDATION.md.
Whole-format 1:1 parity is not established; the robustness goal remains active.

Validation: 320 ordinary tests and ten doctests pass in the full all-features
suite (122.17 seconds for ordinary tests). Strict Clippy/Rustdoc, WASM,
formatting, whitespace and all 70 fixture lengths/hashes pass. The expanded
variant corpus and independent upstream comparison pass separately. This
checkpoint did not repeat the no-default-features suite; the preceding
stat-definition checkpoint records that run. No commit or push was made.

## Bounded script evidence and mode hypotheses

The runtime variant module exposes the canonical `[3,2]` byte array without
claiming its internal archive/bytecode grammar. Its separate filename-candidate
view adapts upstream's ASCII basename search within that bounded blob. Every
source occurrence, exact spelling, separator/basename range and following byte
is retained. A private index supplies lowercased names and all occurrences by
name. Missing, empty and malformed fields remain distinct; names in unrelated
variant fields cannot become script-data candidates.

Mode inference matches upstream's trigger precedence while retaining all
matching rules and their source evidence. An unrecognized set has no mode
classification and is not implicitly PvP. No filename inference alters recorded
engine settings, selects parser layouts, supplies defaults or claims a script
was declared/loaded. The prior canonical film / parser/v41 / runtime split is
unchanged.

The new original-source fixture has 272 authored cases, including all 256
combinations of eight mode-trigger names. Two new tests cover native name-set/
precedence agreement, conflict/repetition retention, scope, malformed/inherited
fields and exact source spans. The expanded captured variant test passes all 32
films with 3,336 per-film distinct names and 4,463 occurrences. Independent
original searches over both the blob adapter and each actual whole registry
return identical name sets and mode choices. No additional names appear outside
the canonical script blob in this corpus. Captured choices cover Slayer,
Strongholds and Oddball; other rules have authored/native-source evidence.

See VARIANT.md and the script-evidence-v41 fixture README for the distinction
between candidate observations and canonical declarations, including `.luac`
prefixes and blob-end matches. Script archive/load semantics, effective option
rules, other variant roles and the earlier wire/context/runtime gaps remain
open. The full robustness and 1:1 format objective is still active.

Validation passes 322 ordinary tests and ten doctests in the full all-features
suite (124.53 seconds). Strict Clippy/Rustdoc, WASM, formatting, whitespace and
all 72 fixture lengths/hashes pass. The expanded corpus, native script comparison
and unchanged existing-view comparison pass separately. No-default-features was
not repeated; no packet grammar changed. No commit or push was made.

## Reference option rules and uncertainty-preserving evaluation

The existing runtime variant module now joins all seven reference Options
scripts /104 rules to recorded option rows. It retains all 200 aliases in
precedence order, matching row ordinals, exact eight-byte values and separate
script-name/prefix applicability evidence. All reference groups remain
inspectable, including those without evidence. Defaults stay in the catalogue
and explicitly hypothetical evaluation; no Film or recorded option field changes.

First-alias precedence agrees with upstream for known unique values. Later
aliases remain visible. Case-only aliases share matching arrays and do not
create false ambiguity; repeated source rows stay ambiguous even with equal
bytes. Unidentified rows share incomplete-match evidence across rules. Missing
or malformed containers/values and unknown payload discriminators cannot become
a default or silently fall through to a later alias. Evaluation distinguishes
Recorded, ReferenceDefault, NoLiteralDefault and Unresolved.

The independent original-source fixture covers 227 authored Bond trees: seven
default cases, all 200 aliases, and twenty order/ambiguity/missing/malformed/raw
value cases. Three tests cover original public-model rule/default/effective
projection, explicit local uncertainty policies, alias identity/order and
applicability evidence. The corpus passes all 32 films with 465 evidenced rules:
285 recorded overrides, 60 hypothetical defaults and 120 absent literal defaults.
Original decoder outputs are compared separately without rounding local values.

This completes the reference-rule convenience projection, not proof of script
loading or actual build/default applicability. Script archive grammar, stat
roles, wider build/context admission and the earlier wire/runtime gaps remain
open. The general film / parser/v41 / runtime organization is unchanged. See
VARIANT.md, variant-rules-v41/README.md and VALIDATION.md. The full robustness
and 1:1 canonical-format goal remains active.

Validation passes 325 ordinary tests and ten doctests in the full all-features
suite (131.46 seconds). Strict Clippy/Rustdoc, WASM, formatting, whitespace and
all 74 fixture lengths/hashes pass. The expanded corpus, independent rule
comparison and unchanged existing-view/script comparisons pass separately.
No-default-features was not repeated. No commit or push was made.

## Recorded stat-text references

The runtime variant module now binds definition fields 8–11 to the indexed
string table in the same selected variant. Field IDs are explicit because their
individual label roles remain unestablished. Direct field queries retain missing,
negative, invalid and conflicting values; consensus requires all four recorded
indices to agree. Equal text at distinct indices is still a conflict. Returned
values retain the definition, each index, string-entry offset, text bytes and NUL
range. Neither stat slots nor marker scans supply an index. Missing or duplicate
stat IDs do not erase independently readable references.

Twenty authored trees and two tests cover reordered/shared references, duplicate
IDs, absent IDs, equal text at different indices, conflicts, missing/invalid
indices, inheritance/duplicates, missing/empty/ambiguous tables, and exact bytes.
The independent original reader matches 5,296 per-row indices across 1,324
captured definitions. All 23,832 localized values follow their recorded table
entries, with source ranges checked against Film. The existing native label
projection agrees on all 1,323 English labels it emits.

The extra association is Oddball definition 46, SkullScoringTicks, sharing entry
55 (Carry Time) with definition 45, SkullHeldTime. The original positional label
scan emits the shared text once and leaves the final definition unlabelled.
The new view preserves both associations, distinct definitions and their differing
metadata. This is not a new unique string or proof of equal units/precision.

Field-specific roles, numeric stat presentation, actual script/default
applicability, wider writer/build support and the earlier packet/context/runtime
gaps remain open. The full robustness and canonical-format objective stays active.
See VARIANT.md, stat-text-v41/README.md and VALIDATION.md for scope and evidence.

Validation passes 327 ordinary tests and ten doctests in the full all-features
suite (129.51 seconds). Strict all-target Clippy, Rustdoc and WASM pass; all 76
fixture lengths/hashes and previous independent view/script/rule comparisons
pass. The successful full-suite rerun disabled incremental compilation after a
disk-full linker failure and package-cache cleanup; see VALIDATION.md.
No-default-features was not repeated. No packet grammar changed; no commit or
push was made.

## HUD static splash-text observations

The existing film / parser/v41 / runtime organization is preserved. The new
`runtime/hud` view reads canonical archetype-47 row-0 components under the
recorded v41/27, version-1 schema. It exposes original event time/order/source,
localization keys and every ordered argument: empty, optional player index,
integer, real code, localization key and unknown-kind word. Every scalar retains
its original component field. Baselines, repetitions, clears and partial records
remain distinct; unrelated DELTAs do not repeat cached text.

The numeric helper explicitly follows pinned `splash::Value::arg`; the actual
game real quantizer is not established. Unknown arguments are retained where
upstream discards their meaning. Component suffixes remain accessible without
invented roles. No viewer count, roster/holder binding, continuous screen state
or visibility duration is inferred. Native identical-message grouping is not
evidence of distinct viewers.

Thirty-five authored bodies exercise every argument kind, signed/real boundaries,
optional players, repeated/max-count arguments and nonempty suffixes. Unchanged
original text/numeric readers supply the oracle. Four tests check all eight bit
alignments, every truncation bit in the mixed body, malformed source fields,
schema/duplicate-row refusal, actual Film baseline/DELTA observations and source
fidelity. The expanded two-film corpus passes in 38.76 seconds; 541 complete
HUD observations agree with the original reader. Recharge supplies 488 present
texts/numeric arguments, while all 37 Octagon observations have absent text.
Other argument kinds and partial-state policies have authored evidence only.

See HUD.md and hud-v41/README.md. Wire grammar is unchanged. The wider context,
writer, ownership, mode-event and round-trip gaps remain open, and the full
canonical-format objective remains active.

HUD validation passes 331 ordinary tests and ten doctests in the full serial
all-features suite (129.85 seconds). The four final targeted HUD tests, strict
Clippy/Rustdoc, WASM, formatting, whitespace, all 78 fixture hashes and the
independent 541-observation comparison pass. See VALIDATION.md for final test
scope and logs. No-default-features was not repeated. No commit or push was made.

## Vehicle-context evidence and refreshed upstream scope

The previous HUD turn was verified implementation progress. This turn returns
to construction context: an all-32-film source-order audit finds 207 stopped
snapshots, 130 vehicle NEWs and seven masked type-state/physics records. Six
selections have complete rows, but none records the gate and complete asset tag
in the same NEW. Two DELTAs match earlier complete full IDs; they remain
candidates until lifecycle continuity is established. No gate is guessed or
stop removed. The research report preserves source spans, independent packet
envelope checks and hashes; see
research/libfilmshell/vehicle-context-20261110.md. All-film/focused tests and
strict Clippy pass. Production grammar is unchanged.

Refreshing GitHub revealed new HEAD `8f0f8d078e513c800baa21fac416fd4c0c127d03`,
with listing-derived engine-player assignments, unit-control ownership and
parent-state attachment/ride tracking. This changes the next implementation
priority. The earlier pin's proximity-only holder/ride descriptions do not
describe current upstream. LIBFILMSHELL_UPDATE_20261110.md maps the new owning
code, retained local source fields, exact source blobs and policies that cannot
be treated as canonical (backdated IDs, implicit index identity, carried zero
rows, inferred reset/clock behavior and bounded-depth parent fallback).

Existing oracles/catalogues keep their old pin. Next implement listing-linked
assignment resolution, controller ownership and generation-aware attachments
with explicit missing/conflicting states and independent direct-value fixtures.
The larger writer/context/round-trip and semantic requirements stay open. No
commit or push was made; the full goal remains active.


## Listing-derived engine-player assignment observations

Added `runtime/roster/assignments.rs` within the existing film / parser/v41 /
runtime organization. The caller explicitly selects
`AssignmentSchema::Reference8f0f8d0`; semantic admission is restricted to v41/27.
Assignments borrow the original listing, member and Option/Slot field. Session
options remain optional and bot slot values keep all 32 bits. The canonical wire
reader is unchanged. Earlier fixtures and catalogues retain their original pin.

Holder queries consider both latest independent populations and expose Unique,
Unassigned, Ambiguous or Unknown, plus every claim and uncertainty source. Full
lists can establish observed departures/reuse; partial, unavailable or duplicate
populations invalidate earlier claims without proving absence. A missing engine
ID blocks a false unique result. Histories preserve every listing and same-time
order, including repetition. No bootstrap backdating, first-later/index fallback
or takeover proximity supplies an identity. These are recorded observations,
not continuity proof across missing updates.

Four new tests cover original readers, all source alignments, malformed/missing
population policy, distinct participant domains, conflicts, departures/reuse,
source-word references, unsupported semantic versions and unchanged Film/cursor.
The separately pinned native fixtures contain 117 records: all 108 scanner-visible present
assignment values agree, while eight absent options remain absent locally even
though the upstream scanner extracts a number from preceding bits. A further complete record with first short 21 is omitted by the original
scanner, which requires 183. These are intentional source-fidelity differences.
Bot slot values outside the scanner's
range remain exact local words; no game-validity claim is added.

See ROSTER.md and assignments-v41/README.md. Controller ownership, attachment
histories, downstream scoreboard/replay attribution, wider versions and the
remaining packet/context/round-trip requirements are still open. The full
robustness goal remains active. No commit or push was made.

Validation passes 335 ordinary tests and ten doctests, with the final fixture
covered by a subsequent successful eight-test roster run. Strict Clippy/Rustdoc,
WASM, formatting/whitespace, all 81 fixture hashes and native reproduction pass.
The 32-film runtime/metadata run passes in 199.39 seconds: 5,347 recorded session
assignments, 5,333 native matches and 14 scanner omissions caused by short 21
instead of the hardcoded marker 183. Identical-payload retained original-Go
outputs corroborate all 5,347 values. Nonempty bots remain authored evidence.
See VALIDATION.md for scope and exact reproduction; no wider parity is claimed.

## Unit-controller observations and exact-event holder attribution

Added `runtime/controllers` while preserving canonical field ownership and the
film / parser/v41 / runtime organization. The view reads unit-control components
by their unique recorded registry row under v41/27, parameters 1/2. It preserves
the optional controlling player, optional six-bit secondary value, independent
32-bit reference and all original fields/events. Partial, duplicate or unsupported
components cannot become controller clears. Raw keyframe namespaces remain
separate from runtime generation IDs; no biped slot range establishes identity.

Every component change and clear is retained. Source histories additionally
retain missing baselines, complete/incomplete deletes, unread partial records
and other-generation records at the same runtime slot. These are lifecycle
boundary observations, not a proof of continuous ownership across missing packets
or generation wraps. No first-controller/first-end aggregation or inferred death
is introduced. Whole life ownership and consumer migration remain open.

`roster.at_event` selects only populations known at the exact borrowed event and
rejects foreign/cloned events. Controller-to-holder linkage uses it to prevent
later same-timestamp listings from backdating an identity. Missing, empty,
competing and unknown populations preserve their existing explicit resolution.
The synthetic actual-Film test observes Unknown, then Unique, then Unassigned
within one timestamp as listing packets arrive and remove that participant.

130 independent component bodies are compared with the unchanged original
UnitControl block at parameters 1 and 2. Six controller tests cover every player
ID, optional branches, all eight bit alignments, every-bit truncation, malformed
fields, corruption trailers, schema/duplicate refusals, source identity,
lifecycle boundaries and exact-event attribution. The roster suite adds a test
for stable event-time queries and foreign-event refusal.

The two-film corpus retains 863 complete observations (135 Octagon, 728 Recharge).
Every native body value and endpoint agrees: 725 present controllers, 138 clears.
All 725 present values have Unique holder observations in these captures. This
does not establish lifetime continuity; ambiguity/refusal policies have authored
evidence. Full suite, native reproduction and final-check scope are documented
in VALIDATION.md. Remaining attachments, life ownership, downstream attribution,
writer/context and round-trip requirements keep the full goal active.

Controller validation passes 342 ordinary tests and ten doctests in the full
all-features run; the final six-test controller rerun also passes. Strict final
Clippy, Rustdoc, WASM, formatting/whitespace, all 83 fixture hashes and native
reproduction pass. The expanded two-film corpus passes in 36.73 seconds.
Whole-format and life-ownership completion is still unproven; no commit or push
was made.

## Parent-state attachments, snapshots and source histories

Added typed `runtime/attachments` views under the explicit
`AttachmentSchema::Reference8f0f8d0` schema, retaining the existing film /
parser/v41 / runtime organization. Admission requires v41/27, a unique matching
object-parent-state registry row, parameter 0–4 and an established archetype.
Attached/detached, typed handles, both generation bits, node/seat, local transform
codes and legacy detached fields retain their source words. Unknown archetypes
and partial components cannot become detached or acquire guessed layouts.

Snapshots distinguish runtime generations from keyframe namespaces, explicit
deletes, missing baselines, source discontinuities, unread packet/chunk suffixes
and incomplete transport. A fresh target incarnation cannot inherit an older
child reference. Root paths have no depth cutoff and distinguish detached roots,
missing/unmapped links, cycles, lifecycle boundaries and unresolved evidence.
Unit handles retain the reference's base 512 and generation; broader Object
handles remain unmapped pending applicability of their reference-table writers.

Histories and direct observation spans share the snapshot transition visitor.
They retain repeated reads, clears and gap/lifecycle evidence in source order.
Packet-end boundaries explicitly follow readable prefixes, including same-time
recovery. An absent final boundary is not a supplied film end or infinite life.
Ancestor changes can affect roots without changing a direct component span;
these spans are not ride durations or continuous root/owner proof. See
ATTACHMENTS.md for the API and policies.

150 authored bodies execute the unchanged pinned native ParentState reader;
all exposed native values/endpoints, parameters 0–4 and eight alignments agree. Captured native
comparison admits 8,525 observations: 1,457 Octagon and 7,068 Recharge, comprising
2,039 attached and 6,486 detached. Of the handles, 881 are Unit and 1,158 Object.
The two additional Recharge DELTAs have complete source fields but no established
archetype, so remain explicit unbound refusals. Captured seats are all absent;
present-seat and lifecycle/error policies have authored evidence only.

The attachment snapshot checkpoint passes 351 ordinary tests, ten doctests,
strict Clippy/Rustdoc, WASM, all 85 fixture hashes and byte-identical native
fixture reproduction. The captured run passes in 36.05 seconds. The subsequent
history/span validation is recorded in VALIDATION.md. Wire readers, historical
appearance expectations and earlier catalogue pins remain unchanged. Continuous
life ownership, ride/consumer integration, wider writer/context admission and
field-driven whole-format round-trip proof remain open. No commit or push was
made; the full robustness goal remains active.

Final history/span verification passes 354 ordinary tests and ten doctests,
with 13 corpus tests normally ignored (78.98 seconds ordinary). Strict all-target
Clippy, Rustdoc, WASM, formatting and whitespace checks pass. The twelve attachment
tests and expanded two-film corpus pass. All verification jobs have completed.
No commit or push was made; the broader canonical-format goal remains active.

## Controller-holder continuity across records and populations

Added `controllers/ownership.rs` under the existing runtime organization.
`ownership(entity, schema)`, `ownership_at` and `ownership_at_event` combine the
recorded controller with contemporaneous human/bot assignment populations.
Histories retain reads, repeats, controller clears, reassignment, competing claims,
missing IDs, failed lists, lifecycle changes and source gaps. Each entry retains
the direct controller source and the listing sources separately. It never assigns
the first observed controller to the whole generated slot, backdates a later
listing or turns a clear/delete into an inferred death.

Extracted the attachment packet-order logic into private `runtime/continuity`;
both views now share prefix/suffix and source-sequence boundaries. Unread entity
suffixes invalidate controller evidence while retaining separately framed lists.
Unread chunk regions or discontinuous packet/chunk sequences also invalidate
both population domains. A new controller value alone cannot resurrect old
population assignments; new human and bot listings must establish them again.
`population_gap` retains the invalidating boundary. Failed replacements retain
their own uncertainty. Runtime generations and keyframe namespaces stay distinct.

Ownership spans retain unknown/uncontrolled states and close at every relevant
record, listing or gap, including repeated values. No duration or recording-end
value is invented. PacketEnd follows the readable prefix even though its source
is the earlier packet envelope. The direct controller history API remains a
record-only view; the new ownership queries supply continuity handling.

Six new tests cover roster reassignment without a controller update, conflicting
human/bot claims, absent IDs, departures, controller changes/clears, malformed and
opaque lists, generation reuse, complete/incomplete deletes, missing baselines,
partial/duplicate components, same-time event boundaries, keyframe separation,
unknown/missing chunk indices, unread tails and incomplete transport. The twelve
controller tests pass; the twelve attachment tests also pass after extraction.
Source-model policy mutations are distinguished from the independent native
controller and listing wire fixtures, which remain unchanged.

The current walker is indexed-event based. Chunks producing no packet event
still need a source-level boundary audit; whole-recording continuity is not
claimed by these queries. Player-life/replay/scoreboard integration, ride
projections, broader writer/physics context, eventless-chunk coverage and full
field-driven format proof remain open. See CONTROLLERS.md and VALIDATION.md for
API scope and final checks. No commit or push was made; the full goal stays active.

The eventless-chunk issue is grounded in `runtime/resolved/events.rs::index`,
which emits entries inside the `chunk.body.packets` loop, and the shared visitor,
which handles chunks when visiting their packet events. A trailing opaque or
refused-transport replication chunk with no framed packet cannot yet supply an
invalidation through this path. The next source-boundary audit must represent
that raw chunk evidence without inventing an event or a timestamp. The current
queries' documented indexed-event scope must not be reported as whole-source
continuity.

Final controller-continuity regression passes 360 ordinary tests and ten
doctests, with 13 corpus tests normally ignored (78.40 seconds ordinary). Final
strict Clippy, Rustdoc, WASM, formatting/whitespace and all 85 fixture hashes
pass; the expanded captured-film run also passes. All verification jobs have
completed. No commit or push was made. The broader robustness goal remains active,
including the identified eventless-chunk gap and life/consumer integration.

## Untimed source chunks now invalidate state

Fixed the eventless-chunk gap identified in the preceding checkpoint. The shared
continuity visitor now also reads canonical replication chunk coverage/transport.
An unread chunk with no framed packet contributes an UnframedChunk boundary that
borrows the actual source chunk, with no Event or synthesized timestamp. Complete
empty clear/zlib chunks are not mislabeled unread. Attachment unknown evidence
now carries the same boundary type; its event accessor is optional.

Source-order bounds place the gap after the preceding known prefix and before
following source data. Reversed clocks choose the earlier bound conservatively.
Timestamp queries cannot carry state through the uncertain interval; exact-event
queries retain the preceding readable prefix. Leading, trailing, adjacent and
entirely unframed chunks stay inspectable. Pending unread packet suffixes precede
subsequent chunk gaps. Both controller and population evidence invalidate, and
fresh observations can restore later state. No manifest offset becomes a clock.

Six new tests cover genuine short unframed bytes, truncated zlib input, complete
empty clear/zlib chunks, original source pointers/bytes, leading/trailing/internal
and adjacent gaps, wholly unframed recordings, reversed source clocks, roster
recovery, packet-prefix ordering and attachment-root/span refusal. Independent
wire-reader fixtures and source parsers are unchanged. Full regression and
captured-film scope are recorded in VALIDATION.md. This closes the specific
no-packet-event continuity gap, not the remaining life/replay integration,
vehicle/writer context or whole-format canonical-proof requirements.

Scope: this shared source-boundary fix applies to attachment and controller
ownership queries. Scoreboard and replay retain separate state/gap machinery;
their handling of untimed source gaps still needs an equivalent audit. In
particular, scoreboard::gaps currently walks packet events and ScoreboardGap
requires a timestamp. No claim is made that those consumers gained this fix
merely because the shared runtime helper now supports it.

Final unframed-chunk regression passes 366 ordinary tests and ten doctests,
with 13 corpus tests normally ignored (79.73 seconds ordinary). The two-film
corpus, strict Clippy, Rustdoc, WASM, formatting/whitespace and all 85 fixture
hashes pass. All verification jobs have completed. No commit or push was made;
the full robustness goal remains active, including the separate scoreboard/replay
source-gap audit, life integration and unresolved writer/format proof.

## Scoreboard and engine source-boundary migration

Scoreboard and engine/player histories now use the shared continuity visitor
through a source-only entry point that works during runtime construction. Existing
scoreboard-specific population/partial-record policies remain explicit. Untimed
unframed chunks, unread transport/tails and source discontinuities invalidate
accumulated rows and identities. Complete keyframe packets reset the whole view
before admitting their baseline records, so omitted entities cannot survive.

ScoreboardGap now retains optional packet time, packet-or-chunk provenance and
an absolute before_event position. StatUpdate/EngineUpdate add event_index;
the original timestamp-local order is retained. Both histories share gap and
event-time indexes. Timed future boundaries are excluded from earlier queries;
untimed gaps use conservative source bounds without inventing a timestamp.
No inferred deletion, owner backdating or recovered statistic fills a gap.

Eight new source-boundary tests cover keyframe omissions, future reset exclusion,
absolute indices across timestamp resets, unframed/empty chunks, incomplete zlib
checksums and clear tails, missing chunk indices, same-time gap/recovery ordering,
engine identity invalidation and valid post-delete recovery. Older ambiguity
fixtures now place coexisting entities in one authored full table rather than
assuming that successive full tables accumulate. The 22 scoreboard/engine tests
pass. Captured-film validation and final checks are recorded in VALIDATION.md.

PlayerReplay retains separate state/lifetime machinery; its gap audit and new
controller/holder integration remain open. This checkpoint does not settle
vehicle physics/writer context, timer roles/final score recovery or full canonical
field/round-trip proof. No commit or push was made; the complete goal stays active.

Final scoreboard/engine source-continuity verification passes 374 ordinary
tests and ten doctests, with 13 corpus tests normally ignored (79.57 seconds
ordinary). The two-film corpus, strict Clippy, Rustdoc, WASM, formatting/whitespace
and all 85 fixture hashes pass. All verification jobs have completed. No commit
or push was made. PlayerReplay source continuity and life/holder integration,
remaining writer/physics context and full canonical-format proof keep the broader
goal active.

## Contextual replay source-continuity checkpoint

Replay now explicitly interrupts transient ownership/state at unread entity
suffixes, refused frame bodies, keyframe resets, refused/changed rosters and
structural source loss. The suffix applies after supported prefix records, which
remain available as observations. Sparse camouflage/inventory no longer bridge
unread updates. Player/team bindings, equipment identities, projectile tracks and
reference transforms are cleared together. Unknown roster continuity cannot be
recovered from a new controller alone; the legacy replay population hypothesis
still needs fresh population evidence after source loss.

The new replay-local continuity module keeps the existing organization and does
not substitute canonical entity stops for contextual decoder results. Structural
source gaps have optional clocks and original input positions; complete empty
chunks preserve index continuity. All nonzero packet clocks are checked before
replay. Untimed loss uses a labeled preceding-packet visibility bound rather than
inventing event/death timestamps. LifeEnd::ContinuityGap distinguishes interruption
from observed lifecycle events. Same-clock prefix observations and later NEWs
remain ordered, while sampled state cannot survive the unread suffix.

The owners implementation at libfilmshell 8f0f8d0 aggregates the first controller
and first end per life. That policy is not adopted. Existing source-linked
controller/holder histories remain separate; their integration into replay and
whole-life/ride projections is still open. This conservative checkpoint can reduce
replay coverage and is not a claim of complete format or derived-output parity.
Validation results are recorded in VALIDATION.md. No commit or push was made.

Final replay-continuity validation passes 382 ordinary tests, ten doctests and
the explicit Octagon/Recharge replay corpus check. Strict Clippy, Rustdoc, WASM,
formatting and whitespace checks pass. All jobs completed. The broader robustness
goal remains active; this checkpoint does not complete a 1:1 canonical parser.

## Replay controller/holder integration

The contextual replay now replaces its option-equals-output-index and
spawn-default-equals-controller hypotheses with the existing shared v41/27
controller admission/value reader and listing-derived holder resolution. Human
and bot populations remain independent, must prove uniqueness, and are cleared
when source loss could hide a listing. Engine-ID reuse changes the holder without
renaming an earlier participant or failing the whole replay. Initial humans keep
bootstrap output indices; later participants receive new stable indices. Bots
have ParticipantId::Bot and no fabricated XUID; unavailable names remain optional.
The former roster_index_assumptions output is removed.

Complete controller DELTAs and later population listings can establish current
ownership without backdating a spawn. PlayerLife now distinguishes Spawn from
Ownership interval starts and links to diagnostics.ownership. That evidence
retains original input/packet addresses for the trigger, controller, both current
populations and uniquely resolving assignment record. Explicit clears, malformed
control, missing/ambiguous populations and competing bipeds remain distinct. The
first controller/end aggregation and first-later holder fallback in libfilmshell
are not imported. Readable prefix observations survive a same-clock transfer,
while sampled pose/inventory/vitality never copy to the new holder.

Fire attribution compares the recorded engine ID against the current controller,
then returns the stable participant index; projectile owners use the same binding.
Team associations also resolve through current holders and require a fresh team
value after reassignment. Shared assignment results for the supported six-bit
controller/team query range are cached per population change, without narrowing
the original full-width bot Slot fields.

This integrates supported contextual ownership intervals, not complete life,
vehicle/ride or whole-format proof. The broad goal remains active. Validation and
API changes are recorded in REPLAY_RESOLUTION.md and VALIDATION.md.

## Source-linked ride histories

`runtime/rides.rs` joins established biped/vehicle parent observations with
current controller/holder evidence. Direct seats retain raw field references;
unknown higher ancestors, cycles and unknown/ambiguous owners remain independent
of the direct recorded link. The view follows full parent chains and watches
ancestor changes, including lifecycle barriers to reused raw IDs. It does not
copy vehicle poses, infer parts by proximity, select first-later owners, classify
AI from any actor-control update or synthesize exit/film-end times.

The attachment and ownership passes share continuity-step ordinals, preserving
packet-end and untimed-gap order. Ordinary attachment histories keep their direct
semantics; ride histories also end at watched ancestor and owner boundaries.
No canonical Film models or parser readers changed for this checkpoint.
Eleven authored integration tests and a retained-film audit cover the new view;
test scopes and results are recorded in RIDES.md and VALIDATION.md.

The full regression passes 400 ordinary tests and ten doctests, with 13 corpus
tests normally ignored. The expanded two-film source audit and strict Clippy,
Rustdoc, WASM, formatting and whitespace checks pass. All jobs completed.

Contextual ride/replay integration, vehicle tracks, full lives, other missing
runtime projections, unresolved writer/physics context and complete field-driven
round-trip proof remain open. The full robustness goal remains active.

## Contextual replay ride integration

The existing film / parser/v41 / runtime organization remains intact.
`runtime/replay/rides.rs` interprets contextual parent components through shared
attachment schema/value admission. Canonical observations keep their original
source Events; contextual observations have input/packet/bit addresses and
separate gap evidence. Preceding sequential keyframe/NEW declarations supply
class context, with duplicate/namespace/generation restrictions. Future headers,
scanned baseline candidates and inferred schemas do not establish ride classes.
Keyframe parent values are not equated to runtime attachment state.

Replay diagnostics now retain admitted parent fields once and use indices for
ride paths, current ownership and per-player ride observations. Samples expose
explicit clears across gaps, ownership changes and life support bounds. The
same-clock earlier evidence remains inspectable without holding it on a former
owner. Direct seats are preserved through ancestor changes, missing higher
roots and cycles. Existing pose reconstruction is independent; this adds no
vehicle-center coordinates, synthetic exits or proximity-based relationships.

Nine authored tests and expanded retained replay audits cover source fields,
current human/bot attribution, same-clock order, long/cyclic paths, loss/recovery,
lifecycle refusal and declaration ambiguity. Test policies and public-capture
limits are documented in RIDES.md and VALIDATION.md. Vehicle tracks and seated
poses, full lives, other runtime projections, unresolved writer/physics contexts
and complete field-driven round-trip proof still keep the whole goal active.

Final verification passes 409 ordinary tests and ten doctests, both retained
replay captures, strict Clippy, Rustdoc, WASM, formatting and whitespace checks.
The ownership-bound life filter prevents same-clock updates crossing respawns or
raw-ID reuse, while keeping earlier source observations in diagnostics. Captured
pose/ownership counts are unchanged. All jobs completed; no commit or push was
made. This checkpoint does not complete the full robustness/1:1 format objective.

## Source-linked replay vehicle tracks

Added runtime vehicle tracks and separately retained namespace snapshots without
changing canonical Film models or parser readers. Sequential class40 declarations
supply admission; current complete NEW defaults supply optional source-linked tags.
Ordinary absolute world positions require explicit regions and matching quantizer
widths. Snapshot positions preserve raw little-endian floats, including signed
zero, while refusing nonfinite values. Other branches keep fields with explicit
errors. NEW/delete/generation changes, partial records and source gaps close support;
same-clock observations remain ordered and unknown positions clear held state.

No later-tag backfill, snapshot-to-runtime identity conversion, generation-zero
filter, proximity-based part link or copied rider pose is added. Authored native
bodies and packet fixtures distinguish reader evidence from model-policy tests.
VEHICLES.md documents API/admission; VALIDATION.md records final verification.
Wider vehicle position branches, seated poses, full lives, ground objects/zones,
unresolved writer/physics contexts and complete field-driven round-trip proof
remain open. The full robustness/1:1 canonical-format goal stays active.

Final vehicle-checkpoint verification passes 416 ordinary tests and ten doctests,
the expanded two-film replay audit, strict Clippy, Rustdoc, WASM, formatting and
whitespace checks. Both captured audits contain zero admitted vehicle observations;
positive position/tag/lifecycle evidence is authored and documented separately.
Player pose/ownership counts are unchanged. All jobs completed; no commit or push
was made. The broader robustness and canonical-format goal remains active.

## Direct mode-qualified zone events

Added runtime/zones over the existing canonical action view. Explicit mode/schema
selection exposes all 23 pinned mode table rows and Strongholds' separate capture
broadcast, retaining original sources, unknown IDs, repeated messages, recipient
masks, keyed points and reference words. Duplicate zone keys, wrong types, missing
markers and partial events remain explicit. Recipient indices never become actors
or teams, and marker removal never establishes a runtime generation. Same-clock
captures do not borrow another message's target; missing hill targets do not use
future HillLive events.

The independent source oracle exercises every ID in 0..1024 across four modes.
Native packet tests and the expanded retained direct-action audit check source
provenance and refusal policies. See ZONES.md and VALIDATION.md. Zone geometry,
manager/navpoint links, ownership/contest timelines, hill scores, current recipient
attribution, ground objects, full lives, wider writer/physics context and complete
field-driven round-trip proof remain open. This is progress toward the full goal,
not a replacement definition of canonical format completion.

Final zone-event verification passes 423 ordinary tests, ten doctests, the
expanded two-film direct-action audit, strict Clippy/Rustdoc, WASM, formatting,
whitespace and all 86 fixture hashes. The native table oracle reproduces exactly.
Recharge has 1,718 matching original Oddball messages; the audit preserves all
12,420 unknown IDs and checks source fields. These counts are not deduplicated
scores/actions or proof of actors. All jobs completed; no commit or push was made.
The full robustness and 1:1 canonical-format goal remains active.

## Navpoint source-state checkpoint

Added runtime/zones/navpoints over canonical class12 docking, progress and visual
component reads. Unique registry declarations and source extents govern admission;
keyframe namespaces remain separate from runtime IDs. Filters retain every sparse
slot/tag/negation/payload, while visual groups retain both default and per-clause
state/order words. Unknown/missing/partial data cannot become a default owner,
empty inside mask or held prior group. Named-state and progress helpers remain
explicit reference interpretations without viewer, manager or zone attribution.

The upstream geometry audit identifies a separate nested-Bond boundary/selection
problem; no first-object or missing-dimension-zero convention is made canonical.
Boundary-aware shape interpretation, manager/zone/condition/lifecycle joins,
hill scoring, ground-object/full-life/writer contexts and whole-format field-driven
round-trip proof remain open. See ZONES.md for API/evidence and VALIDATION.md for
verification. The full goal remains active.

Final navpoint verification passes 428 ordinary tests, ten doctests, the expanded
two-film audit, strict Clippy/Rustdoc, WASM, formatting and whitespace checks.
Recharge supplies 4,851 admitted component observations, including 865 conditional
state/order items and 2,067 explicit group clears; Octagon supplies none. Every
captured field matches original bits. Existing action/zone-message counts remain
unchanged. All jobs completed; no commit or push was made. The full canonical
format and robustness objective remains active.

## Bounded crate geometry checkpoint

Added runtime/zones/geometry over class38 multiplayer-properties components,
sharing the existing registry/bootstrap Bond reader through a bounded internal
entry point. Canonical film fields and component endpoints remain unchanged.
Admission checks exact registry/name/parameter, status, source extents, retained
field bytes and corruption trailers. Nested parsing cannot consume following
components; incomplete Bond trees remain inspectable without publishing geometry.

All root items and geometry structures/list/set entries retain source order.
Unknown shape codes, exact signed/unsigned integers, optional dimensions and
map-team assignments survive. Missing dimensions, cylinder right extent, team
narrowing and default orientation do not inherit upstream defaults. Duplicate
requested fields and unresolved inheritance refuse selection. The native oracle
pins unchanged upstream source; authored tests cover malformed lengths, all bit
alignments, truncation, source mismatches, schema/duplicate refusal, namespace
separation and equal-clock clears. See ZONES.md and the geometry fixture README.

Orientation, manager/navpoint/zone joins, condition evaluation, zone lifecycles,
hill scores, ground objects, full lives, wider writer/physics contexts and complete
field-driven canonical round-trip proof remain open. The full objective is active.

Final verification passes 434 ordinary tests, ten doctests, the expanded two-film
audit, strict Clippy/Rustdoc, WASM, formatting and whitespace checks. All 88 fixture
hashes pass and the 17-case native oracle reproduces exactly. Captures provide
721 geometry entries, with source bits verified throughout; 64 recorded empty
right-dimension wrappers remain explicit MissingWrappedValue results. Existing
action/zone/navpoint counts are unchanged. See VALIDATION.md for counts and limits.
All jobs completed; no commit or push was made. The full goal remains active.

## Crate orientation source/projection checkpoint

Added runtime/zones/orientation over canonical class38/parameter2 dynamic
orientation fields. Exact source/schema checks precede typed raw, absolute or
incremental interpretation. Configuration overrides remain faithful to the
existing instruction-backed reader. All selectors, raw vector words, direction
codes, keep gates and roll increments survive; omitted updates do not supply a
heading. The source Film and parser organization remain unchanged.

The libfilmshell helper's compatible 30-bit default-up turn convention is an
explicit reference projection. A separate engine-heading helper uses complete
LE engine-memory raw vectors or instruction-backed config/default-up midpoint
rolls, with its f64 numerical limit documented. Upstream raw offsets skip the
config bit and ignore overrides; those paths are not copied. The independent
fixtures retain the original helper output as well as an unchanged LevelUp f32
roll oracle. Neither projection becomes canonical orientation or a zone pose.

The retained audit verifies 5,501 admitted absolute component occurrences,
including 1,933 projectable config/default-up cases, 2,776 unresolved nominal
headings and 792 unresolved nondefault-up cases. There are also 1,208 unsupported
schema candidates. Raw/incremental positive coverage is authored, not captured.
All source fields match packet bits; prior action/zone/navpoint/geometry counts
remain unchanged. See ZONES.md and VALIDATION.md.

General orientation reconstruction, manager/navpoint/zone joins, viewer conditions,
continuous zone state, hill scores, ground objects, full lives, wider writer/physics
contexts and complete field-driven round-trip proof remain open. The full goal
remains active; no commit or push was made.

Final orientation verification passes 438 ordinary tests, ten doctests, the
expanded two-film audit, strict Clippy/Rustdoc, WASM, formatting and whitespace
checks. All 91 fixture hashes pass and both independent oracles reproduce.
All verification jobs completed. This remains a checkpoint toward the full goal.

## Manager reference-state checkpoint

Added runtime/zones/managers for source-linked class10 default target handles and
all16 navpoint rows. Repeated registry names are legitimate; duplicate record
indices and unsupported declarations refuse. Exact source bits, version/presence,
category-zero index/generation and full navpoint words survive. Reference index
projections retain discarded bits and do not claim runtime/keyframe equivalence.

Point queries through exact Events or recorded time use shared continuity.
Independent rows survive a matching row's clear, and missing/partial/new/deleted/
generation/gap evidence cannot inherit old links. Source snapshots and tests keep
keyframe namespaces distinct. Native checked packets additionally establish both
canonical corruption-trailer spellings, including default-state checks.

The retained audit validates 11,359 manager observations in Recharge: 798 targets
and 10,561 rows, including 10,188 clears and 373 reference index projections.
Octagon has none. All source bits match, with no captured schema/read refusals;
negative coverage is authored. Existing audit counts remain unchanged.

The full objective remains active. Identity-domain/base proof must precede
manager/navpoint/zone joins; condition evaluation, zone timelines, hill scores,
ground objects, full lives, wider writer/physics contexts and whole-format
field-driven round-trip proof remain open. The distinct keyframe/DELTA corruption
trailer names also require auditing the older runtime consumers (attachments,
controllers and zone geometry/orientation/navpoints) with native checked packets.
No commit or push was made.

Final manager verification passes 444 ordinary tests, ten doctests, the expanded
captured-film audit, strict Clippy/Rustdoc, WASM, formatting and whitespace checks.
All91 fixture lengths and hashes pass. All verification jobs completed; the full
robustness and canonical-format objective remains active.

## Shared component-check trailer checkpoint

Completed the follow-up corruption-trailer audit. A native cross-consumer fixture
reproduced checked-keyframe rejection in all six older readers before the fix.
Controllers, attachments, geometry, orientation, navpoints and vehicle positions
now share strict suffix validation with manager navpoint rows. Both canonical
spellings are accepted as exact pairs; no source names or parser endpoints change.
Geometry excludes either trailer form from its bounded Bond payload. Missing,
malformed, misordered and extra trailer fields remain refusals. This does not
claim to recompute the stored check value.

Tests cover disabled checks and both enabled gate values through real packet,
record and component readers, semantic values, original bytes, unchanged Film,
vehicle snapshot projection and malformed suffixes. Existing independent fixture
bytes remain unchanged. The earlier note about auditing these runtime consumers
is addressed by this checkpoint. The full objective remains active, including
identity-domain proof, zone joins/conditions/lifecycles, wider contexts and full
field-driven canonical round-trip proof. No commit or push was made.

Final shared-trailer verification passes 447 ordinary tests, ten doctests, all
three targeted retained-film audits, strict Clippy/Rustdoc, WASM, formatting and
whitespace checks. All91 fixture hashes pass. Checked-body positive evidence is
native authored data; retained captures verify regression behavior. All jobs
completed. The full objective remains active.

## Source-supported navpoint history

Added `runtime.navpoint_history(schema)` with occurrence iteration and timestamp /
exact-event queries. Existing `runtime.navpoints(schema)` callers keep their
iterator API. Both paths share the original source observation reader; Film and
parser/v41 are unchanged. Each docking/progress/group slot keeps its own evidence,
including decode errors and explicit group absence. All eight visual groups retain
separate support even when their group hashes or order words match.

Shared continuity invalidates stale support on NEW/keyframe, deletion, another
full-ID generation, partial records and packet/chunk gaps. Readable prefixes remain
inspectable at their exact events; timestamp queries include same-clock later
records and unread suffix boundaries. Foreign/cloned events refuse. Keyframe
namespaces are separate from runtime generations; opaque chunks create no clocks.
Three new native/model-policy tests exercise these cases, and the retained-film
navpoint audit samples source-event queries. ZONES.md documents the API and limits.

This adds lifecycle state needed for later zone work. It does not resolve viewer
filters, manager/navpoint/crate identity joins, ownership/contest timelines, hill
scores, general orientation, ground objects/full lives, wider writer context or
whole-format field-driven round trips. The full robustness goal remains active.

Final navpoint-history verification passes 450 ordinary tests and ten doctests,
the expanded two-film source audit, strict Clippy/Rustdoc, WASM, formatting and
whitespace checks, and all91 fixture hashes. Existing captured observation counts
are unchanged. All jobs completed; no commit or push was made. This checkpoint
does not complete the full robustness or 1:1 canonical-format objective.

## Native masked-property enum widths

The hill-score source audit found that the earlier shared-property checkpoint
adopted libfilmshell's 32-bit fallback for masked tags 12–15 without checking the
native switch default. This is now corrected: tags 11–15 all carry four enum bits.
Pinned LevelUp instruction analysis (`LOTCBIS_PHASE0.md` sections 2.2–2.4) and its
original captured vectors establish the branch. The shared component/default
helper preserves the distinct raw tag and code. Scalar/masked domain gates remain
unchanged. A new captured-vector test reproduced a 36-versus-8-bit overread before
the fix; the following 28 bits no longer enter the property's value.

The original libfilmshell component/creation bytes and oracles remain unchanged.
Four component cases now assert the explicit disagreement. Thirty-two original
masked default bodies encode the conflicting layout and are retained as upstream
comparison evidence; native default tests now use original-Go source traces.
The new property-v41 suite includes 128 component bodies, 128 default bodies and 33
original captured vectors with provenance/chained flags. All functions used by
its exporter remain unmodified, with Git blobs checked against the pinned source
tree. Eight alignments, every-bit truncation, source bits/serde and actual NEW /
DELTA/keyframe following-component boundaries are covered. See the fixture README.

This supersedes the earlier ledger's claim that 32 bits were a canonical correction.
Upstream parity alone did not prove those widths. The pinned source also records
an unresolved class 13 NEW-envelope issue; its tag-only fallback is not imported.
Hill property history/attribution, viewer filtering, identity joins, broader writer
contexts and full-format proof remain open. Film/parser/runtime organization is
unchanged; no commit or push was made.

Hill-score implementation findings for the next step: class 13 row 0 is a recorded
32-bit property-name update, row 1 is scalar, and rows 2–33 are the 32 masked values
(the retained Recharge registry confirms all names/parameters). Upstream's hill
reader caches names only at NEW and removes them at DELETE; a source-faithful view
must also handle row 0 updates and keyframe/default names. It must retain scalar
versus per-viewer domains, independent row lifetimes and conflicting observations.
The upstream bank formula and numeric thresholds are reference projections;
`team_of(row-2)` and `1-own` require separately established current viewer/team
identity and a two-team context. A later limit update cannot retroactively make
earlier missing score limits known. These are remaining implementation requirements,
not capabilities claimed by the property-width fix.

Final property-width verification passes 453 ordinary tests and ten doctests,
all 32 retained films /403,465 frame packets for source preservation, all three
two-film runtime audits, strict Clippy/Rustdoc, WASM, formatting/whitespace and
all 94 fixture hashes. All jobs completed. The full goal remains active; there is
no claim that source retention resolves the remaining packet/context/semantic
or whole-format proof requirements.

## Source-linked networked-property history

Added runtime/properties over canonical class 13 defaults and all 34 component
rows. The API exposes original occurrences and timestamp/exact-event state, with
independent name, scalar and 32 viewer evidence slots. Name updates are retained;
values from earlier records retain their own sources instead of being retroactively
relabeled. Full defaults retain their version/name/array gate and all ordered
variants. The supported component layout requires exact row names/parameters;
legitimate repeated masked names remain distinct by row index.

Shared continuity handles NEW/keyframes, deletes, generation changes, partial
records, packet suffixes and unframed chunks. Values retain errors and source
ranges, absent payloads stay distinct from zero, and a partial default supplies
no inferred name or baseline. Both corruption trailer spellings and default
checks are validated structurally. Runtime and keyframe identity namespaces remain
separate. No parser reader or canonical Film model changed in this checkpoint.

Four tests reuse all 256 source-traced property bodies with every truncation,
plus native configured keyframe/NEW/DELTA/DELETE fixtures across all 32 viewer rows
and checked/unchecked packets. They exercise name changes, default/row overrides,
scalar/masked domain replacement, clears, same-clock ordering, partial reads,
generation reuse and gaps. Schema/duplicate/default-status mutations are labelled
model-policy tests. The captured audit now includes property observations and
sampled exact-event state/source queries. See PROPERTIES.md and VALIDATION.md.

This completes the raw property-history prerequisite for hill scoring. Name/value
binding, normalization/limits, viewer/team identity and conflict-aware score
timelines remain open, alongside the existing wider context and whole-format
requirements. The complete objective remains active; no commit or push was made.

Final property-history verification passes 457 ordinary tests and ten doctests,
the expanded two-film captured audit with eight exact-event history queries,
strict Clippy/Rustdoc, WASM, formatting/whitespace and all 94 fixture hashes.
Captured observations total 6,139 on Octagon and 36,694 on Recharge, with no schema
or decode errors; counts describe source occurrences, not scores. All jobs
completed. The full objective remains active; no commit or push was made.

## Source-bound hill inputs and reference points

Added runtime/zones/hill_score with an ordered ledger and timestamp/exact-event
queries. Four explicit property-name hashes bind only subsequent values; changed
names, NEW/default/keyframe baselines, deletes, generation changes, partial records
and source gaps invalidate prior support. Same-name updates preserve existing
samples' original name evidence. Scalar/viewer scopes and runtime/keyframe IDs
remain separate. Invalid values replace prior success with source-linked errors;
duplicate named entities refuse even when values agree.

The labelled bank/point projection matches libfilmshell arithmetic, requiring
masked tag-7 banks and typed, bounded integer limits in the same scope/namespace.
There is no scalar/cross-viewer fallback, future-limit backfill or implicit team
assignment. The upstream reader's raw-word coercion and two-team mapping are not
treated as canonical facts. ZONES.md and fixtures/hill-score-v41 document the API,
source pin, harness scope and deliberate differences.

Six tests pass, including 60 unchanged upstream arithmetic results exercised in
native checked/unchecked packets, lifecycle and ordering boundaries, duplicate and
type refusal, and explicit corrupt-model policies. Both retained captures pass
source checks and contain recognized limits but no supported bank inputs; this is
not real hill-score parity evidence. Full verification passes 463 ordinary tests,
ten doctests, the expanded captured audit, strict Clippy/Rustdoc, WASM, formatting
and whitespace. All 94 prior fixture hashes remain unchanged; the new arithmetic
JSON is fixture 95. All jobs completed.

This closes the ordered binding/limit/reference-projection portion of hill work.
Viewer/team identity and score timelines, broader zone joins, complete lives and
ground objects, unresolved writer/physics contexts and field-driven whole-format
proof remain open. The film / parser/v41 / runtime organization is unchanged.
No commit or push was made; the full objective remains active.

## Snapshot objects and stationary reference candidates

Added runtime/objects with source-checked class 37/40/42/43 keyframe headers,
tags and raw world positions. Native namespaces remain separate from runtime
identities. Raw observations retain absent/partial defaults, missing components,
zero coordinates and unsupported forms. Complete reads validate fields against
original packet bits, registry rows, source ranges and component-check trailers.

The explicit reference projection compares complete snapshots using the original
flags/nonzero/part filters, first-anchor 0.05f32 tolerance and two-sample threshold.
Runs retain all samples, singleton evidence, exclusions and confirmation events.
Different tags/classes start new comparisons; duplicate identities refuse.
An address present outside the selected projection is not reported as absent.
Incomplete tables, missing chunks and source gaps break support. Unread framed
tick-entity suffixes are attached warnings because they cannot hide a snapshot;
matching snapshot positions do not prove continuous immobility or a physical life.

Seven tests pass, including nine sequences through unchanged upstream world and
raw-position readers, native checked/unchecked packets, boundary policies and
explicit corrupt-model tests. Regeneration normalizes the original unspecified
ordering of equal-position/tag results and is byte reproducible. Captured Octagon
and Recharge audits produce 43 and 165 reference candidates respectively, retaining
all sources and exclusions. These are not verified lifetimes or pickup events.

Final verification passes 470 ordinary tests and ten doctests, the expanded
captured audit, strict Clippy/Rustdoc, WASM, formatting/whitespace and all 96 fixture
hashes. Prior 95 fixtures remain unchanged. All jobs completed; see OBJECTS.md and
VALIDATION.md. No parser or canonical Film changes were made in this checkpoint.

Ground-weapon work still needs tick-based source histories and position-domain
handling. The inspected inventory.rs reader keeps NEW tags pending until an UPDATE
position, overwrites prior positions, and pairs same-packet deletions with same-tag
pickup events. Those are distinct claims: later/resting coordinates must not be
backfilled to earlier timestamps, and tag/packet coincidence cannot prove which
object was picked up. Generation reuse, partial records and gaps need explicit
boundaries. Broader identities, zone/life work, writer contexts and complete
field-driven format proof remain open. The full objective remains active; no
commit or push was made.

## Tick weapon-object history checkpoint

Added runtime/objects/weapons with full-ID intervals, current NEW tags, every
source-linked position and explicit lifecycle/gap ends. Indexed, default, local
and raw position domains remain distinct. Initial NEW positions and readable
partial prefixes survive; no future coordinate or snapshot tag backfills a tick.
Exact-event queries preserve same-clock order and reject foreign events. Missing
tags and invalid positions remain visible. Packet/tag candidate indexing retains
all matches without assigning pickup ownership or consuming an ambiguous event.

Seven authored sequences execute the entire unchanged inventory.rs at libfilmshell
8f0f8d078e513c800baa21fac416fd4c0c127d03 (blob
c5cc80c8f3cf7093da53b73cf00ca9a4937ffc79). Nine focused tests cover native
checked/unchecked packets, position domains, partial/source/schema boundaries,
generation reuse, snapshots and pickup ambiguity. Reference regeneration agrees
byte for byte. See GROUND_WEAPONS.md for differences from upstream inference.

Final verification passes 479 ordinary tests and ten doctests, the expanded
captured audit, strict Clippy/Rustdoc, WASM, formatting/whitespace and all 97 fixture
hashes. All prior 96 hashes remain unchanged. Capture counts and final log paths
are in VALIDATION.md. All jobs completed; no parser/Film behavior changed.

Next concrete evidence: class 42 in the retained Recharge registry has row 18
`item-at-rest-component`, parameter 1. The canonical component reader already
retains its one-bit `value`; the existing components-v41 fixtures contain both
values in snapshot and delta modes. LevelUp grammar at
413971bcc485be53a2fa8cf229e170539a5146f8, dispatch_item.go blob
b083f1af71c3f7bc478aa578eb07fccf27f67f6d, independently records ReadBit for
this component (FUN_140ff91e4). A runtime rest layer should preserve explicit
false, omitted and invalid values and source/gap timing, without inferring rest
from unchanged coordinates. Parent/world transforms, physical pickup identity,
zone/life joins, writer contexts and full format proof remain open. The full
objective remains active; no commit or push was made.

## Recorded weapon item-rest checkpoint

Weapon-object tracks now retain `rest_flags`, and timestamp/exact-event states
expose `rest_flag`. Each occurrence keeps its event, component, one-bit source
field and explicit true/false or read error. Omission remains distinct from false;
omitted updates retain the original observation only within a supported interval.
Invalid fields replace prior success. Complete rest prefixes in partial records
remain inspectable at their exact event, then close with the unread suffix.
Snapshots do not seed runtime IDs. NEW, DELETE, generation, partial-record and
shared gap boundaries apply to rest support just as they do to positions.

The layer reuses the canonical one-bit component reader and existing original
libfilmshell fixtures. Shared local component validation checks registry/name,
parameter, duplicate rows, source bits, extents and check trailers. Four new tests
cover native checked/unchecked packets, initial/missing/true/false values,
same-clock ordering, omission, readable partial prefixes, snapshot and opaque
boundaries, generation replacement and nine malformed-model cases. No physical
rest onset, ground placement, position or pickup identity is inferred.

All 13 focused weapon-history tests, 483 ordinary tests and ten doctests pass.
The captured audit observes 52 true flags and no false flags on Recharge, with no
read/schema errors; Octagon has no recorded weapon rest flags. Four exact-event
queries retain the original Recharge components. Other captured counts remain
unchanged. Strict Clippy/Rustdoc, WASM, formatting/whitespace and all 97 fixture
lengths/hashes pass. All jobs completed; final logs and timings are in
VALIDATION.md. No parser/Film behavior changed, and no commit or push was made.

Recorded flag history is complete for this admitted class-42 row. Physical
rest-onset interpretation, default-domain coordinates, parent/world transforms,
physical pickup identity, broader life/zone joins, writer contexts and complete
field-driven format proof remain open. The full objective remains active.

## Fixed-domain reference position checkpoint

Added explicit `FixedPositionSchema::Reference413971b` and typed DefaultWorld /
ParentLocal results under runtime/coordinates. Weapon positions expose
`fixed_midpoint(schema)`, retaining the selected schema, domain and original
codes with their f64 bin centers. This is precision-16 reference projection:
22-bit default-world codes use +/-20,000 and 14-bit local codes use +/-100.
Indexed/raw forms refuse this method. Width/range/raw-word errors are explicit;
no missing map becomes a default box and no local result becomes a world pose.
Selecting a reference is an explicit applicability assumption, not automatic
recorded-build admission. Parser/Film behavior is unchanged.

The new fixed-positions-v41 exporter executes unchanged LevelUp bound/width
functions and libfilmshell LevelBox::engine, verifying four Git blobs. It retains
150 edge/center/seeded vectors with exact bound, width and f64 result bits.
Four new tests verify all vectors, every axis's invalid widths/ranges, malformed
source words, and source-linked NEW/DELTA integration with all check-trailer
forms. Existing position-domain tests also verify indexed/raw refusal. The
fixture regenerates byte for byte; source scope and reproduction are documented.

The original libfilmshell ground assembler drops a fixed default box because its
`place` helper requires a known level box. This projection goes beyond that
behavior without adopting inferred map calibration or later resting positions.
It preserves existing f64 midpoint semantics, not bit-exact engine f32 rounding
or the unknown original unquantized coordinate.

Final verification passes 487 ordinary tests, ten doctests, captured audit,
strict Clippy/Rustdoc, WASM, formatting/whitespace and all 98 fixture hashes. All
prior 97 hashes remain unchanged. Recharge supplies one parent-local position;
Octagon has no fixed-domain weapon positions and neither capture supplies the
default-world form. Default-world validation is authored/native/oracle evidence.
All earlier capture counts remain unchanged. See VALIDATION.md for final timings.
All jobs completed; no commit or push was made.

Physical rest-onset, automatic fixed-bound build admission, parent/world
transforms, proven pickup identity, broader life/zone joins, writer contexts and
full field-driven format proof remain open. The full objective remains active.

### Next verified discrepancy: zero-bit position axes

An independent execution of the same unchanged LevelUp width law returns
`[0,0,0]` at precision 16 for positive extents 0.001 and 0.016, `[1,1,1]` for
0.017, `[14,14,14]` for 200, and `[22,22,22]` for 40,000. Harness and output:
`/private/tmp/halo-position-width-audit/{main.go,result.txt}`. It compiles the
verified original profile sources; this is executable evidence, not a proposed
interpretation of a film.

Current `film/maps/mod.rs::axis_bits_at_precision` applies `.max(1)` to the
computed width, so it disagrees for those small positive extents. Canonical
`components/position.rs` also admits only 1..=32 widths at its selected-region
guards. Supporting this branch needs an audit through width derivation, primitive
zero-bit reads, following-field boundaries, canonical raw-field representation
and runtime conversion. A zero-bit code is schema-defined, not a missing read
that may be silently replaced with zero. No width/parsing change was made in the
fixed-midpoint checkpoint; its fixed widths 22/14 are unaffected. This remains a
concrete next wire-format robustness task.

## Canonical zero-bit position-axis checkpoint

The verified discrepancy above is corrected. PositionBounds no longer forces a
one-bit minimum for a single-bin positive range. Position-component and event
admission accepts explicitly supplied zero-width axes; canonical readers already
retain the named zero-bit field without advancing the cursor. Runtime dynamic
positions, weapon positions and ActionPoint views preserve the same explicit
field. Missing context, unknown regions, missing fields and failed earlier reads
do not manufacture zeros. The delta-width zero-as-no-override convention is
unchanged and remains separate from an explicitly selected axis width.

An independent exporter executes the original width law and indexed point reader
on 80 authored cases, with two verified source blobs. Seven new tests cover exact
traces, width thresholds, zero/nonzero mixtures, every-bit truncation, following
components/events, native NEW/DELTA envelopes, check trailers, raw-field shape,
source-linked runtime queries and model immutability. The detonation view now
validates its known suffix while preserving the unnamed tail; a removed axis can
no longer shift a direction word into a valid point unnoticed. Two obsolete
invalid-zero-width cases were replaced with out-of-range width cases, while both
single-region and complete-table zero-width contexts have positive coverage.

Final verification passes 494 ordinary tests and ten doctests, the retained
runtime audit, strict Clippy/Rustdoc, WASM, formatting/whitespace and all 99 fixture
lengths/hashes. Prior 98 fixtures are unchanged and the new oracle regenerates
byte for byte. Full source preservation passes all 32 films and 403,465 frame
packets in 368.93 seconds, including the hour-long raid. The 451 keyframe tables
still comprise 243 closed and 208 explicit partial stops; these are not declared
fully decoded. Details and logs are in VALIDATION.md. All jobs completed.

This is a source-backed canonical correction, not simply libfilmshell parity:
its first-axis-positive availability test can fall back to other widths. The
existing film / parser/v41 / runtime organization is preserved. Wider component
branches/builds, unresolved writer contexts, remaining identity/pose/zone joins
and complete field-driven format proof remain open. No commit or push was made;
the full objective remains active.

## Scoreboard holder checkpoint

`runtime.scoreboard_holders(AssignmentSchema::Reference8f0f8d0)` now joins
every statborg/engine update to recorded populations at its exact event. Source
identity and raw rows remain separate from holder resolution. Slot reuse, later
same-timestamp lists, unknown identities, missing domains/IDs, competing claims
and duplicate statborg entities retain their evidence. Old rows and finalized
round values are not retrospectively assigned to a new participant. Complete
bot lists preserve raw slot state; partial human/bot lists retain explicit gaps.

Missing packet/chunk sequences, unread chunk tails and unframed chunks forget
both populations until fresh independent listings re-establish them. Existing
entity/keyframe boundaries invalidate identity without inventing roster changes.
The six new integration tests cover these policies, source links and unchanged
Film/playhead state. Existing independent assignment/component/default fixtures
remain the native field evidence; this runtime layer adds no wire admission.

Verification passes 500 ordinary tests, ten doctests, the retained Octagon /
Recharge runtime audit, strict Clippy, warnings-denied Rustdoc, WASM, formatting
and whitespace. Prior captured runtime counts remain unchanged; new holder
counts and exact logs are in VALIDATION.md. All jobs completed. Timer roles,
participant aggregation, recovered final-score coverage and the broader format
proof remain unfinished. No commit or push was made.

## Timer semantic audit and next format prerequisite

The [timer audit](research/libfilmshell/TIMER_AUDIT.md) prevents an unsupported
universal limit/remaining interpretation: Octagon's second timer word increases
while Recharge's usually decreases. Every changing positive-time pair crosses
a known gap/reset. Native field order and conversion remain established, but
state meanings, respawn units and derived countdown starts do not.

Most opaque packet gaps come from the per-tick type-10 companion: 26,100 in
Octagon and 72,145 in Recharge; type 6 contributes 22 and 61 respectively.
Neither body is decoded locally; upstream skips them after envelope framing.
Investigate their native body grammar/effects before relaxing continuity.
Unread entity suffixes remain an independent problem. The captured source
checks, deterministic report and input hashes are retained; no production
semantic claim or wire admission was added by this research checkpoint.

## Outer packet-body investigation

The [32-film body inventory](research/libfilmshell/PACKET_BODIES.md) confirms
451 type-6 packets with four zero bytes and 403,465 type-10 packets with eight
observed lengths and 1,028 distinct payloads. Exact source examples and input
hashes are retained. Pinned upstream container/source code frames these packets
but supplies no native body decoder. Corpus regularity cannot establish their
state effects, so they remain opaque with explicit gaps. Native writer/reader
evidence is required to close this specific canonical-format requirement.

A separate actionable capability gap is movement input: libfilmshell `control.rs`
projects complete kind-0 control records into forward/left/stance series, while
local Film already retains the raw fields but PlayerReplay has no input series.
Continue that source-backed runtime work; this packet-body limitation does not
block the entire remaining objective. No production behavior changed in the
packet investigation. Counts and deterministic regeneration pass.

## Movement-input replay checkpoint

The preceding input-series gap is now addressed by `runtime/replay/inputs.rs`
within the existing organization. It retains complete raw kind-0 controls,
source fields/ranges, optional baselines and source-time holder evidence.
Forward/left follow the unchanged upstream subtraction, including code 63 as
+32; stance remains an optional raw word. Repeats and absent blocks remain
inspectable. Unknown/conflicting holders are not backfilled from later lists.

Samples end at unread control suffixes, opaque packets, population loss or
reassignment, existing replay clears and the requested endpoint. Untimed opaque
packets retain no clock and use the preceding envelope's support bound.
Controls need no invented biped life. Film and parser admission are unchanged.
These policies intentionally preserve more evidence and uncertainty than the
upstream deduplicated, fully-readable-frame series. See INPUTS.md.

The 57-case unchanged-reader oracle regenerates identically as fixture 100;
all prior fixture hashes match. Four focused tests, 504 ordinary tests, ten
doctests, retained contextual replay/source audit, strict Clippy, Rustdoc and
WASM pass. Counts and logs are in VALIDATION.md; all jobs completed.

The next concrete runtime gaps in pinned `model/state.rs` / `ammo.rs` are
reserve-ammunition history and equipment energy/recharge observations. Local
weapon history currently retains loaded rounds, consumed charge and raw heat;
equipment history retains selection/object identities. Those do not supply
these missing channels. Upstream equipment attribution uses preassembled life
slots and drops countdown-like repeats; any local addition must instead retain
raw updates and explicit current parent/controller/holder support. No countdown
units or physical recharge semantics follow merely from its field names.
Broader canonical-format requirements remain open. No commit or push was made;
the full objective remains active.

## Reserve-ammunition replay checkpoint

`runtime/replay/weapons/reserves.rs` now projects the existing canonical
`weapon-state-rounds-inventory` reader into weapon history. Each complete
record retains the unsigned eleven-bit count and exact component source; its
slot uses the reserve family's independent registry rank among four rows.
Optional component-check trailers are validated. Repeated values and zero are
observations; omitted components and weapon capacity do not create counts.

All four latest supported reserve values are exposed in each weapon sample's
`reserve_slots`. Sparse active/secondary fields follow the existing weapon
presentation and republish supported values on selection/identity changes.
Explicit packet, life and source gaps invalidate support. Opaque non-frame
packets now create reserve-only boundaries with original packet addresses;
untimed bytes retain an absent clock and the preceding envelope's support bound.
Reserve-only clears do not erase legacy magazine/charge/heat inventory state.
Source order governs equal-time clears and subsequent observations.

The new reserves-v41 fixture executes unchanged pinned component parsing and
the complete ammo reader on nine authored updates / twelve reserve components.
The adapter supplies an explicit preassigned life; it does not certify upstream
life attribution. Existing component fixtures remain the primitive evidence.
Four new tests cover reference agreement, independent slots, selected/hidden
weapons, repeated zeros, maximum counts, identity changes, full/partial guard
trailers, absent values, life ends, source ordering and reserve-only timed/untimed
boundaries. The input capture audit now uses a set for index membership instead
of quadratic repeated linear searches, preserving the assertions and additionally
rejecting duplicate observation indices. Final validation is in VALIDATION.md.

The film / parser/v41 / runtime split remains unchanged. Canonical readers and
prior fixtures are untouched. Equipment energy/recharge is the next concrete
state capability gap: native fields are fourteen-bit energy and ten-bit delay
on class 37. Any participant projection must join current source-linked parents,
controllers and holders, retain raw changes, and keep time units unproven.
Upstream's preassembled life-owner map and countdown deduplication do not supply
that proof. General opaque-packet continuity for older replay channels and the
broader canonical-format obligations remain open. No commit or push was made.

## Equipment energy/delay source view

`runtime.equipment(EquipmentSchema::Reference8f0f8d0)` now exposes raw
energy/delay observations, per-object histories and exact-event queries over
canonical components. Fourteen-bit energy and ten-bit delay retain every source
update, including zero and repeated countdown-like values. Optional reference
energy conversion matches the original endpoint-aware f64 quantizer and rounded
percentage; no physical clock or recharge-start semantics are inferred.

Field admission checks declared class 37, registry rows, completeness, exact
field bytes/ranges and corruption-check trailers. Unknown/partial candidates
remain inspectable. Snapshots separate runtime generations and keyframe
namespaces and invalidate state at partial records, lifecycle/source/opaque
packet boundaries. Backwards/late-zero source clocks explicitly refuse temporal
state and ownership for the supplied recording, while raw observations survive.

On-demand holder queries retain the contemporaneous attachment path, direct
biped, controller and independent human/bot populations. They reject future
same-time assignment, deleted/stale parents, conflicting populations and parent/
controller support older than an opaque packet. No whole-life owner map or
arbitrary same-tag item selection is adopted. These queries walk source prefixes;
a cached contextual PlayerReplay energy channel is still outstanding.

The equipment-energy-v41 oracle runs unchanged pinned component parsing,
quantization and the complete ammo reader on 26 updates / 49 scalar fields.
Its explicit parent/life adapters isolate reference value/history behavior.
Native local packet tests separately establish holder ordering and conservative
state policy. Existing component fixtures remain the primitive width evidence.
See EQUIPMENT.md and VALIDATION.md for API limits, counts and final checks.
The existing film / parser/v41 / runtime organization is preserved. No canonical
reader changed in this checkpoint; no commit or push was made. Full format proof,
legacy replay continuity, cached equipment replay and other ledger obligations
remain unfinished.

## Contextual equipment energy/delay replay

The cached equipment replay gap above is now implemented in the existing
`runtime/replay/equipment` family. Shared native field validation retains exact
source fields, fourteen-bit energy, ten-bit delay and scalar errors. Diagnostics
retain runtime and keyframe readings in separate namespaces; snapshots never
seed runtime state. The unchanged component/ammo oracle is reused without new
fixture bytes or canonical reader changes.

Each source observation distinguishes candidate ownership evidence from an
admitted source-time participant. Fresh direct class-37-to-class-35 attachment
and controller evidence is required after gaps. Later ownership may support a
cached object reading without rewriting its historical attribution. Per-player
samples retain all supported objects, independent energy/delay references,
current attachment paths and life-bound ownership. Empty samples are unknown
support, not assertions of empty equipment inventory.

Opaque packets and unread control suffixes clear this channel; general replay
gaps, object NEW/DELETE, generation reuse, reclassification and partial records
also invalidate cached values. Complete partial-prefix readings remain visible.
Source discontinuities are scheduled before the following chunk/packet, allowing
fresh evidence there to re-establish support. Untimed packet boundaries retain
an absent clock and the preceding envelope's exclusive support bound.

Five integration tests cover exact original quantization and repeated readings,
multiple objects and holder handoffs, human/bot conflicts, partial suffixes,
timed/untimed opaque packets, missing-source restoration and generation reuse.
Their small registry uses an explicit bootstrap policy, not public bootstrap
admission evidence. The capture audit checks original scalar fields and sampled
ownership/life links; zero supported attribution remains a valid unknown result.
See EQUIPMENT.md and VALIDATION.md for API details and validation results.

Final verification passes 517 ordinary tests, ten doctests, the retained
contextual capture audit, strict Clippy/Rustdoc, WASM, formatting/whitespace and
all 102 registered fixture hashes. Octagon retains 100 runtime / 70 snapshot
scalar readings; Recharge retains 1,887 / 1,892. Both have zero scalar refusals
and zero positively source-attributed readings under the current support rules;
positive attribution is established by the authored integration tests. All 14
prior contextual capture lines remain unchanged. All jobs completed.

Physical delay units/recharge semantics, opaque-packet continuity in older replay
channels, unresolved snapshot/writer contexts, remaining entity/zone/score joins
and full field-driven canonical-format proof remain open. The full objective
remains active. No commit or push was made.

## Weapon charge/heat reference quantization

The remaining approximate weapon-value projection now has explicit source-backed
reference APIs in `runtime/replay/weapons/quantization.rs`. `WeaponCharge` wraps
the existing twelve-bit consumed word; charge and heat provide reference f64
fractions and lossy rounded percentages. The exact endpoints and interior-bin
operation order match the pinned component reader. Out-of-domain public model
words refuse conversion. Raw fields, sparse samples and native gate-zero
semantics are unchanged. Baseline variants are not relabelled as transmitted
charge observations; heat flags retain their wire order without gameplay labels.

The new weapon-quantization-v41 oracle executes all 4,096 charge codes, all 128
heat codes with four flag combinations, and nine sparse updates / 19 components
through unchanged pinned components.rs and ammo.rs. Three tests compare exact
f64 bits, rounded values, independent slot families, raw source fields, repeated
updates, open zeros, closed gates, absent values, invalid words and unchanged
source models. The preassigned-life harness isolates scalar/history behavior;
it does not establish owner or lifetime inference. See VALIDATION.md for final
verification and captured source comparisons.

Final checks pass 520 ordinary tests, ten doctests, strict Clippy/Rustdoc, WASM,
formatting/whitespace and all 103 fixture lengths/hashes. The reference fixture
regenerates identically. Captures validate 200 Octagon and 1,352 Recharge charge
words against source bytes; all are closed-gate zeros and neither capture has
heat observations. Nonzero/heat evidence comes from the exhaustive native oracle.
All 16 prior contextual audit lines remain unchanged. All jobs completed.

A fresh source audit confirmed that upstream still probes both snapshot vehicle
layouts for a plausible next slot. Its event reader supplies no additional
full-precision projectile context either. Neither is a new canonical admission
rule. Snapshot construction/writer contexts, older replay-channel opaque
continuity, remaining entity/zone/score joins and full canonical-format proof
remain open. No commit or push was made; the full objective remains active.

The next concrete continuity discrepancy is in contextual replay's non-frame
`body.is_err()` branch: it currently interrupts inputs, reserves and equipment
energy only. Legacy pose, magazine/charge/heat, grenade, vitality, selection,
ride/vehicle and ownership state can still span that unread body. Unread control
suffixes likewise have channel-specific handling. The existing general
`continuity::State::interrupt` clears these channels at structural/entity gaps;
extending that boundary handling requires source-order and fresh-owner tests,
including late-zero packets and complete prefix observations. No native proof
yet allows type-6/type-10 bodies to be treated as harmless to held state.

## General opaque-packet and control-suffix continuity

The preceding discrepancy is addressed: unread non-frame bodies now invoke the
same general State interruption as structural and unread entity boundaries.
Unread control suffixes do so after retaining their readable entity/control
prefix. When both sections are partial, there is one general UnreadFrame gap.
Pose, weapon selection/magazine/charge/heat/reserves, grenades, vitality,
camouflage, equipment, team bindings, reference transforms, vehicle/projectile
tracks and ownership cannot silently span those boundaries. Later controller
and current holder evidence may restore ownership; parent/vehicle paths require
their own fresh evidence. Unknown bodies still have no invented state effects.

Repeated gaps retain every original source address and optional clock in
diagnostics. Already-cleared biped channels and rides do not gain redundant
per-subject transitions; independent input support can resume and clear again.
Energy's interrupted-support diagnostic also recognizes general attachment gaps.
The ride capture checker uses set membership for exact gap keys instead of
repeated linear searches across the now per-tick gap list.

Three new tests cover all held player channels, nonredundant clears, native
type-6/type-10/unknown packets, native unsupported control kind 3, vehicle track
closure, fresh owner/parent restoration, late-zero clock preservation and later
same-clock recovery. The existing readable-prefix and energy tests still pass.
The small native registry uses the documented explicit bootstrap fixture policy;
it is not public bootstrap admission proof. The captured audit additionally
requires that no ownership interval contain a known gap strictly inside its
support bounds; equal-clock source-order restoration remains admissible.

This can substantially reduce contextual player coverage because upstream's
opaque bodies have no established harmlessness proof. Raw canonical reads are
unchanged. See VALIDATION.md for final counts, changes and checks. Packet-body
grammar, snapshot/writer context, remaining participant/zone/score relationships
and complete field-driven film-format proof remain open. No commit or push was
made; the full objective remains active.

Final verification passes 102 focused replay tests, 523 ordinary tests, ten
doctests, the strengthened two-capture interval/source audit, strict Clippy and
Rustdoc, WASM, formatting/whitespace and all 103 unchanged fixture hashes. Both
captures now have one supported pose observation per ownership interval (100
Octagon / 264 Recharge), with 26,148 / 72,281 general gaps. This reduction is
documented explicitly in VALIDATION.md; all raw equipment/input/event comparison
summaries and canonical Film serialization remain unchanged. All jobs completed.


## Nonempty end-packet admission

The v41 replication dispatcher now admits `EndPacketBody` only when the type-7
payload is empty. Previously a nonempty payload produced the empty body variant:
coverage retained those bytes as unparsed, but contextual replay did not see an
unread-body boundary. Nonempty bodies now use the existing `UnsupportedLayout`
refusal, preserving envelope, raw bytes, serialization and opaque coverage. The
existing general replay interruption consequently closes held support until
fresh evidence resumes. No framing or other packet-type reader changed.

Pinned upstream `container::walk_packets` supplies envelope bounds, not semantic
proof for a nonempty end body. The new guard does not classify such a packet as
illegal, and does not decide zero-size/post-type-7 framing questions. Public
parser tests cover empty, zero-byte, nonzero and header-shaped payloads, each
with/without a trailing packet-shaped suffix. A native-body runtime integration
test verifies empty-marker continuity, nonempty-body closure and fresh evidence
in the next chunk, while retaining the original Film unchanged.

A current raw-envelope audit found 451 replication end markers in 451 chunks
across 32 films, all empty and terminal. This is the replication-only subset of
the prior 483 end-marker count, which also included summary chunks. The new
branch therefore changes no captured replication body. All 103 registered
fixture lengths and hashes remain unchanged. See VALIDATION.md for checks.
The full canonical-format objective remains open; no commit or push was made.

Final verification passes 525 ordinary tests, ten doctests, strict Clippy/
Rustdoc, WASM, formatting/whitespace and all 103 fixture hashes. The targeted
all-envelope corpus audit confirms 451 replication plus 32 summary empty terminal
markers. All jobs completed; full canonical-format proof remains open.


## Vehicle fixed-domain position projections

The vehicle runtime previously refused complete default-world and parent-local
absolute-position branches even though canonical parsing retained their fields.
They now expose raw `Fixed` codes and their domain, with an explicit pinned
`fixed_midpoint` conversion using the existing shared quantizer. Sparse
`position_read_at` retains known/fixed/unknown distinctions and the existing
exclusive support bounds. Ordinary world queries still refuse fixed positions;
a source-local handle or region is not promoted to a world transform.

Both new branches validate all field names, widths, extents, raw encodings,
optional local handle/region fields, finite word and complete corruption suffix.
Unknown regions remain unknown. No Film reader, parser context, transport,
source bytes or registered oracle changed. The module organization is unchanged.
The existing original-Go bounds/original-Rust arithmetic fixture provides 150
vectors, now exercised as native vehicle components at all alignments and local
gates. Native packet integration, malformed-field/source-preservation tests and
the retained captured replay audit validate the projection and support policy.
See VEHICLES.md and VALIDATION.md for exact scope and verification.

Vehicle predicted/delta/writer contexts, parent-to-world transforms, snapshot
construction context, opaque packet bodies and full canonical-format proof
remain open. This extension does not import upstream calibration or proximity
inference. No commit or push was made; the full objective remains active.


Final verification passes 11 focused vehicle tests, 528 ordinary tests and ten
doctests, strict Clippy/Rustdoc, WASM, formatting/whitespace and all 103 fixture
hashes. The retained capture audit passes with all 18 prior summaries unchanged;
its two films contain no vehicle positions, so positive fixed-domain evidence
comes from the native fixtures and independent scalar oracle. All jobs completed.


## Projectile observation retention and sampling boundaries

The legacy projectile view discarded tracks with fewer than two positions,
discarded zero-length support intervals, replaced previous positions at the same
clock and treated the 100 ms sampling-age limit as a reason to drop a later
complete DELTA. These presentation filters no longer remove admitted source
observations. Every tracked position is retained, including singletons and equal
clocks; sparse updates can resume sampling after the age limit without filling
the unknown interval. Actual source gaps, malformed records, deletion and
replacement still close tracks and require a fresh creation to restart.

Pinned upstream `grenades.rs::reader` appends its accepted DELTA positions without
same-frame replacement or a wall-clock freshness cutoff. Its throw-window and
proximity ownership rules remain excluded. Local creation/known-tag/current-owner
admission stays unchanged, and the 100 ms sampling limit remains explicit. No
canonical reader or fixture changed. The remaining dynamic-position branches
are not additional upstream support: pinned components.rs::Position rejects any
nonzero bit in its initial four-bit prefix. This checkpoint makes no new
predicted/delta position interpretation.

Authored native NEW/DELTA records reuse the independent creation fixture's
layout with one same-width recognized-tag substitution and explicit owner binding.
Tests cover eight alignments/equal timestamps, single and zero-length intervals,
same-ID replacement, late sparse updates and source-gap refusal. They validate
history policy, not automatic owner inference or a new wire grammar. The captured
replay audit now checks every retained projectile source position, timestamps,
map-region conversion and absence of a known source gap inside supported bounds.
See REPLAY_RESOLUTION.md and VALIDATION.md for results and remaining limitations.


Final verification passes seven focused projectile tests, 531 ordinary tests,
ten doctests, strict Clippy/Rustdoc, WASM, formatting/whitespace and 103 unchanged
fixture hashes. The capture audit passes with all 20 previous summaries unchanged.
Both captures currently have zero admitted projectile tracks; positive retention
evidence comes from the native-record tests. All jobs completed. No commit or
push was made, and complete canonical-format proof remains open.


## Control-list continuation at the resource boundary

The canonical control reader previously returned `RecordLimit` immediately after
its 64th record, before checking the list terminator. It now follows the recorded
continuation bit before applying the budget: zero completes, an unavailable bit
is truncated, and one is retained before refusing the next kind/body. This
preserves both exactly-full valid lists and the exact partial boundary. The
64-record cap remains an explicit resource policy; upstream's longer lists are
not newly admitted or declared invalid.

The independent control-limit-v41 oracle runs original libfilmshell control_view
on 24 authored lists, including 65/128-record lists. Three new tests compare all
retained read/skip boundaries at eight alignments, exact-end truncation, complete
prefixes, source coverage, serialization and runtime prefix access. The original
17-case control oracle regenerates identically through the exporter, whose only
new interface is an optional cases path and disposable offline dependency setup.
Native source commentary separately confirms continuation-before-budget order
and explicitly says its own 64-iteration cap is not wire grammar. Kind-3 and
unknown reference-layout refusals remain unchanged.

The whole 32-film canonical source audit is repeated for this reader change and
now reports maximum control-list length and budget-stop count. Validation results
are recorded in VALIDATION.md. No commit or push was made; complete capability
parity and canonical-format proof remain open.

Final verification passes 11 focused control tests, 534 ordinary tests and ten
doctests, strict Clippy/Rustdoc, WASM, formatting/whitespace and all 105 fixture
hashes. The complete source audit passes in 354.58 seconds across 32 films and
403,465 frame packets. Its maximum decoded control-list length is 20, with zero
budget stops; positive evidence at and beyond the changed boundary comes from
the independent fixtures. Keyframe/control corpus counts are unchanged. All
jobs completed. The explicit 64-record resource policy remains a difference
from upstream's unbounded list traversal, not a wire-format maximum.

## Source-time kill and game-event participant joins

The new `action_holders` view adds the holder links established explicitly by
libfilmshell 8f0f8d0 `assemble.rs`: kill victim/killer/assistant IDs, game-event
recipient bits and player-typed extra arguments. It retains the original action,
each optional source word, each recipient bit and each argument's original
position. Unknown, unassigned and conflicting holders stay distinct, including
human/bot namespace conflicts. It does not fold recipients into a participant
mask or supply first-later/raw-index fallback identities.

The scoreboard's existing population cursor now lives under `runtime/roster`
and serves both views without changing its boundary policy. Exact event order
excludes future same-timestamp listings. Unframed chunks, unread chunk suffixes
and source discontinuities clear both populations; each independently recorded
domain must become available again. Filtering events preserves preceding roster
history. No Film/parser reader, source bytes, fixture or module boundary changed.

Five new native-packet tests pass. The existing retained capture audit now
compares participant joins with an independent bounded roster scan, checks
source words and claims, and preserves all prior action results. Octagon has
263 admitted game bodies and 264 holder readings; Recharge has 14,138 game
bodies and 15,546 readings. All captured readings resolve uniquely under the
explicit schema. Neither capture admits a complete canonical kill body, so
positive kill-role evidence is confined to the native fixtures. Partial kill
records remain explicit errors. Broader actor-handle, damage/weapon and mode-role
attribution remains open; this is not full canonical-format completion.

Validation and completed-job status are recorded in VALIDATION.md. No commit or
push was made. The full objective remains active.

Final checks pass 539 ordinary tests and ten doctests in each CI feature
configuration, strict Clippy/Rustdoc, WASM, formatting/whitespace and all 105
unchanged fixture hashes. Five focused tests pass on the final public result
model. The retained action capture audit passes with the prior source/semantic
counts unchanged. All jobs completed.

## New upstream revision and nine event readers

A fresh remote query found f050371a6cda83b9887eeb83a1884336fce28e53. The separate
checkout and source comparison are recorded in LIBFILMSHELL_UPDATE_F050371.md;
old fixtures/catalogues retain their stated pins. This revision adds event
bodies, resource budgets, component corrections and further runtime/data work.
The full goal must include that remaining audit, not merely the original alias
inventory.

Nine source-backed event readers now handle codes 2, 8, 12, 13, 14, 80, 104, 105
and 106 in the existing event modules. Melee-clang (12) replaces a fixed 94-bit
skip with its conditional players/tag/point/direction body. Precision-12 and
precision-16 points reuse existing explicit context/default-box rules. Code
117, also new upstream, was already locally supported. Raw fields, truncated
prefixes and refusal at unknown indexed context remain intact.

The independent new-pin oracle covers 162 authored complete lists. Eight focused
tests pass at all alignments, every-bit truncation, exact primitive/source
boundaries, following-event/entity/control integration and missing/invalid
context. The old event oracle regenerates identically. Six old reference-only
fixtures and 128 historical continuation cases retain their original data;
their former unsupported-body expectations are explicitly superseded only for
the newly proved bodies. The focused historical suite passes after that update.
No old golden was regenerated.

The retained action audit passes with unchanged Octagon/Recharge counts. Those
captures have no newly admitted event bodies from this extension, so the new
wire fixtures supply positive branch evidence. Full-suite/source-audit results
are recorded in VALIDATION.md. The old snapshot-context and opaque-packet
limitations remain; new resource limits and component corrections are next
source-backed work. No commit or push was made.

Final verification passes 540 ordinary tests and ten doctests in each feature
configuration, the focused historical check, strict Clippy/Rustdoc, WASM,
formatting/whitespace and all 107 fixture hashes. The 32-film source audit passes
across 403,465 frame packets in 374.77 seconds, with prior keyframe/control counts
unchanged. All jobs completed. Full capability/canonical-format proof remains
open; the new source audit records the next concrete implementation priorities.


## Transport byte and packet budget checkpoint

The shared transport now applies configurable `ParseLimits` while preserving
existing module ownership. Defaults: 256 MiB source/decoded chunk limits,
separate 1 GiB source/decoded film totals and 65,536 envelopes per packet chunk.
Fatal admission returns a typed resource error and stops consuming input;
chunk-local inflation retains its prefix/status, and packet limits retain exact
envelopes plus the unread suffix and a typed stop. Both continuity paths treat
those unread suffixes as gaps. See FORMAT.md for the public contract.

Eight new boundary/retention/continuity tests pass. Both feature configurations
pass **548 tests and ten doctests**; strict Clippy/Rustdoc, WASM, formatting,
whitespace and all **107** fixture hashes pass. The **32-film /403,465-frame**
source audit passes in **354.61 seconds**, with zero packet-policy stops and
unchanged keyframe/control counts. All jobs are terminal; no commit or push.

LIBFILMSHELL_UPDATE_F050371.md now records the concrete remaining secondary
control differences, already-supported orientation/command-tick/Bond safeguards,
raw-position tail questions and the unresolved predicted-grapple source conflict.
Those wire corrections and the remaining identity/mode/data views still require
work. Neither byte-budget completion nor retained source counts establish a
complete canonical parser.

## Corrected secondary-control checkpoint

The second optional reference now uses category 1, local positions consume
three 14-bit words, and target presence gates both its handle and 11-bit word.
The existing film/parser/v41/runtime organization is unchanged. The 336-case
authored oracle checks the corrected grammar, following records and all bit
alignments. The old 17-case oracle remains byte-identical, with three historical
full-body expectations explicitly superseded.

The 32-film audit passes across 403,465 frames in 354.73 seconds. Keyframe counts
are unchanged, but complete control counts fall from [1063261, 263, 242] to
[1063258, 260, 231], and secondary blocks from 168 to 166. A captured differential
regression retains 72 views: all 47 complete endpoints match corrected upstream,
46 differ from old upstream, and 25 remain explicit partials. Complete records
contain 70 full secondary bodies, 28 local positions and 27 present targets.
This is positive captured evidence for the branch corrections, not proof of
complete format coverage. See VALIDATION.md and the controls-f050371-v41 README
for the exact source pins, reproduction commands and limitations.

Raw-position scope/tail handling, predicted-grapple source disagreement and the
remaining identity/mode/data projections are still open. No commit or push.

The latest raw-position correction applies the native byte-reversed exponent
predicate to three known raw reader branches. Nonfinite words now refuse before
the two-bit footer, keep their exact source and cannot be made complete by a
stub. The retained 32-film audit passes with unchanged aggregate counts. Seven
old synthetic component cases and one world-reader runtime case that accepted a
NaN remain as historical oracle evidence. Set-parent tail behavior remains
unresolved: a captured snapshot with a set gate confirms its existing endpoint,
so a blanket refusal was reverted. See LIBFILMSHELL_UPDATE_F050371.md and
VALIDATION.md for the exact scope and evidence.

Final verification for this raw-position correction passes 556 ordinary tests
and ten doctests in both feature configurations, strict Clippy, formatting and
whitespace checks, plus the 32-film source audit. No commit or push; the full
robustness objective remains active.

The next type-9 correction admits the recorded six-bit creation participant
only below 32 through typed complete-record accessors. Replay uses the typed
value for managed team controllers, while every raw six-bit word remains in
the default-state source. An original f050371 function oracle covers 1,024
authored values/version forms/alignments. See VALIDATION.md for the fixture and
final verification. Broader managed-player/life joins remain open.

Final type-9 verification passes 558 ordinary tests and ten doctests in both
feature configurations, strict Clippy, formatting/whitespace, all 114 fixture
hashes, and retained Octagon/Recharge replay. This change does not alter wire
reader boundaries; the prior 32-film source audit remains valid. No commit or
push. The overall robustness objective remains active.

Final verification: 553 ordinary tests and ten doctests in each feature
configuration, strict Clippy, formatting/whitespace and all 112 fixture hashes
pass. The final captured-test-only iterator adjustment was checked separately;
no production code changed after the successful full source audit. All jobs
completed; the overall goal remains active.

## Latest dynamic-position prefix comparison

The reproducible 32-film inventory reaches 50 non-`00` ordinary dynamic-
position prefixes, all in the raid film. Two hash-verified captures now have
an f050371 primitive-read oracle. Its `11` branch stops after two bits versus
the native-source local 99-bit keep-baseline read; its `10` branch reads a
101-bit raw position versus the native-source local 99-bit keep-baseline read.
The pinned LevelUp position reader expressly carries the latter path under
both second-bit values. The local grammar is retained; this upstream branch
cannot replace it on the captured evidence. The comparison does not settle
later packet boundaries or the separate snapshot parent gate. See
LIBFILMSHELL_UPDATE_F050371.md and the position-prefix fixture README.

## Life-bounded contextual playback

`PlayerReplay` now exposes unique-life `life_at`, strict `sample_at` and
visibility-scoped `alive_at`; `PlayerReplaySet::samples_at` yields all supported
players in stable output order. The existing `at` still returns raw timeline
clear markers after a life. Exclusive end bounds prevent timestamp-only
queries from carrying a pose across death, ownership loss or an unread gap.
An active life with no established coordinates still has a sample; overlapping
publicly mutated life intervals and samples predating the selected life refuse.
The focused replay tests pass in both feature configurations and strict
all-target Clippy passes. These queries do not resolve equal-clock event order,
complete life histories, inferred rides or snapshot vehicle context.

## Life-bounded riding evidence

Contextual replay now filters source-ordered ride observations to one unique
supported life and exposes direct Riding support spans with exclusive ends at
the next sample or life boundary. Full entity IDs and opening ownership evidence
guard same-clock transfers; gaps cannot be bridged. This supplies the supported
portion of latest libfilmshell's per-life ride lookup without importing its
inferred exits. Eleven focused ride tests pass in both feature configurations;
strict all-target Clippy, formatting and whitespace checks pass. The retained
Octagon/Recharge replay audit passes but contains no established Riding states,
so captured positive riding remains unproven. Snapshot vehicle context and the
full canonical-format objective remain open.

## Optional summary/highlights section

Latest libfilmshell accepts replay films without a terminal highlights chunk.
`Film.summary` is now optional; missing, present-empty and present-unread sections
remain distinct. Canonical packet/source indexing and runtime replay events work
when it is absent, while summary events and reports are empty. The parser now
requires at least one replication chunk and still rejects a duplicate or
nonterminal summary. A type-0 synthetic integration case covers absence and
serde/source/runtime behavior. The full 32-film source audit passes across
403,465 frame packets; all captured films have a summary, so that corpus does
not positively exercise absence. Latest upstream also rejects films with no
ticks after assembly; `Film::parse` checks tick evidence after reading
the chunks.
All-feature
Theater tests pass 523/523 nonignored cases after the final shape change.

## Manifest indices and replay tick evidence

The parser's `ParsedReplicationStream::replay_tick_status()` scans framed type-0 payloads for the current
upstream high-frequency tick-update signature and its eight-bit counter. It
returns a source location, proven absence after complete framing, or an
undetermined result when bytes remain unread. This is signature evidence, not a
new canonical entity boundary. All 32 captured films contain the marker; the
source audit verifies that assertion while retaining its 403,465 frame checks.

`Film::parse(chunks)` is the complete downloaded-film path. It accepts
manifest chunks in any order, checks contiguous indices and
duplicates, retains manifest indices after sorting, rejects incomplete
inflation, and requires tick evidence. Fully framed no-tick inputs return
`NoTicks`; an unread replay suffix returns `TickEvidenceIncomplete`. Resource
errors report manifest chunk indices. Private structural parser tests can
inspect partial source reads; every chunk still has a numeric index. Synthetic
negative tests and one reversed-order captured film
exercise these checks. These capabilities do not close snapshot
vehicle context, opaque packet bodies, wider build support or the full format.

## Latest upstream mode-icon addition (a13efb3)

The current upstream HEAD checked on 2026-11-11 UTC is
[`a13efb323b230c96b59685defbc376720043240f`](https://github.com/OpenSpartan/libfilmshell/commit/a13efb323b230c96b59685defbc376720043240f).
The one new capability since f050371 is the variant mode icon. The selectable
`VariantSchema::ReferenceLatest` now reads the selected Bond header's path
`[0, 10]`, preserving the recorded integer source. An omitted field yields a
labeled serializer-default zero only when the header exists. An unchanged
current-source oracle agrees on all 32 retained headers (30 zero, one eight,
one two); synthetic cases reject duplicate, wrong-
type and negative fields. This uses the canonical Bond tree rather than
upstream's backward byte scan. The older catalogue blobs were unchanged by the
upstream commit and retain their historical extraction provenance.

## Current-source motion-encoding vote diagnostic

`TheaterRuntime::motion_encoding_evidence()` now reports the latest upstream
first-three-replay-chunk marker vote used to choose its motion extraction path.
It retains both vote counts, first source-linked examples, an explicit caller
player-count hint and incomplete-transport positions. Labels require a complete
vote window unless a positive multiplayer vote or the caller's count already
selects that branch. It does not select a canonical parser layout.

The independently exported current-source oracle agrees for all 32 retained
films without a player-count hint: 29 `PerEntity`, three
`MultiplayerComposite`. The exporter checks the a13efb3 `container.rs` Git
blob; its manifest hashes verify and regeneration is byte-identical. The
focused threshold/marker-span test, all-feature Theater suite (540 passed, 18
ignored), strict all-target/all-feature Clippy, formatting and whitespace
checks pass. The last source edit after those broader checks was a test-only
threshold case, which passed separately. This closes the missing diagnostic,
not the upstream motion extraction grammars, inferred body reconstruction,
snapshot vehicle context or whole-format field-driven round-trip proof.

## Source-linked carrier position during supported rides

Contextual replay now resolves a uniquely supported rider, direct vehicle or
vehicle part, and the current carrier's sparse position at a requested time.
`ride_carrier_position_at` keeps the original position observation and optional
world point separate from the player's pose; missing/ended/replaced carrier
tracks and detached or unsupported lives refuse the join. The existing
`VehicleTrack::position_read_at` now shares its owning observation lookup with
this query. Synthetic native ride fixtures cover direct/part joins, missing
positions, detachment and track ends. All-feature Theater tests pass 541/541
nonignored cases, strict all-target Clippy, formatting and whitespace checks
pass. No captured positive seated-motion oracle or seat transform exists, so
this does not complete libfilmshell's spliced motion output or canonical film
decoding.

## Raid NEW-default refusal audit

After the source-backed NEW-gate correction, the optimized 32-film source
assertion passes and the release inventory reproduces its checked-in totals.
The inventory now records all 32 `UnsupportedDefault` NEW stops: they occur
only in the long raid, span 15 chunks and apparent archetypes 35–43, and all
end at `mpp.count` values 5–7. The pinned original creation reader admits
at most four entries. This is a concrete boundary/context investigation target,
not evidence to relax the native count guard. The extra inventory fields leave
prior source coverage and rejection totals unchanged. The canonical parser,
vehicle context and whole-format round-trip remain incomplete.

The first raid refusal has now been traced to a missed generation-2 type-42
create after a `MissingKillPlaybackContext` event stop, followed by a stale
type-9 binding. The resulting local DELTA ends 35 bits before libfilmshell's
resumed entity walk and manufactures the apparent NEW. See the
[source-lineage audit](research/libfilmshell/RAID_SCHEMA_DIVERGENCE_20261111.md).
Other refusals require independent checks. The upstream create is recovered
after an unread section and is not promoted into the canonical `Film`.

Contextual replay under its explicit `KillPlaybackDisabled` assumption reaches
the same full-ID type-42 NEW at relative bits 5,021–6,001 in chunk 140, packet
1182, and the frame closes. `ReplayDiagnostics.assumed_creations` now exposes
all such NEW candidates with exact source, status, closure and assumptions;
the raid has 2,621 across all packets and one for slot 1333. The canonical
packet remains `MissingKillPlaybackContext`/`IncompleteMessageList`.

## Vehicle snapshot successor evidence

The runtime's on-demand pair of bounded vehicle snapshot reads now includes
next-slot evidence for each branch and the current upstream's default-first
layout hint. It also reports a stricter hint only when one branch has a
plausible successor and the other is explicitly rejected. Missing successor
words remain unknown. The retained Cadet Blue snapshot yields a vacant next
slot for the no-type branch and a rejected typed branch; the canonical Film
still stops at its missing construction gate. This adds upstream-style
diagnostic recovery without treating successor shape as the unrecorded flag.

## Explicit script-event writer context

The public Film parse context now accepts the event-15 optional-prefix gate
when complete recorded game options do not establish recorder simulation.
The source-backed simulation remains authoritative: contradictory external
context is rejected before replication decoding. A reached event reports the
external selection separately from its raw fields; default parsing still stops
at `MissingRuntimeGate15`. Authored film-envelope cases exercise both
prefix branches, incomplete game options, recorded-context precedence and
conflicts. This does not identify unrecorded writer simulation in captured
films or establish other event/body grammars.

## Configurable control-list budget and longer-list admission

The former fixed 64-record control cap is now `ParseLimits.max_control_records`
with a 65,536-record default. Authored 65- and 128-record lists complete through
public Film parsing with their original traced endpoints and source coverage.
The independent 24-case oracle still checks exact source fields at eight bit
alignments under an explicit 64-record budget; exact-limit termination,
truncation and a present successor retain their distinct outcomes. The cap is
resource policy, not a claim about writer grammar or a measured corpus need:
the retained 32-film audit previously found a maximum of 20 records. Full
canonical-format and runtime capability coverage remain open.
