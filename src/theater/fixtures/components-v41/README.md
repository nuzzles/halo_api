# Component reader differential fixtures

Reference: OpenSpartan/libfilmshell
`18067987ebc3628ad499c819c6aee814692df43b` (also public HEAD when checked during this audit).

`wire_cases.py` independently authors 12,011 wire layouts for 284 component aliases:
the 57 aliases absent at the start of this component audit plus 227 shared aliases
with additional snapshot, filter, handle, property, point, action and vehicle-physics branches. This expands the original 38-name inventory beyond
names ending in `-component`. Zero/nonzero words, optional handles and generation
zero, masks, variable lengths, count limits, text parameter variants, row versions,
legacy metadata interleaving and both position encoding modes are represented.
The position cases retain negative zero and a NaN payload as exact raw bits.
The seven original synthetic snapshot positions with a NaN remain byte-identical
historical oracle rows. The native position reader refuses that NaN before its
two-bit footer; current tests mark those old complete outcomes as superseded.
New `components/tests/position_finite.rs` checks the byte-reversed exponent
predicate, finite closing bits, nonfinite refusal, exact raw source fields and
the retained snapshot parent-gate boundaries. A proposed blanket parent refusal
was reverted after a pinned captured snapshot and downstream checks contradicted
it. Parent-tail width still needs branch-specific source and engine context.

`reference.json` is produced by `export_reference.py` from a disposable `git
archive` of that pinned upstream commit. It instruments only `Reader::bits` and
`Reader::skip`, then calls the original `Reader::block` with the original name
dispatch and explicit row version and snapshot mode. The exporter rejects any
case whose authored boundary/status disagrees with upstream. It never loads
halo_api or generates expectations from our parser. Data is synthetic; no real
film contents or player data are embedded in these fixtures.

Tests exercise the normal component adapter and verify every upstream primitive
boundary, contiguous source fields, raw bytes and serialization. Our fields may
split aggregate upstream skips, but cannot erase their boundaries. Every case
is truncated at every bit; its retained end is aligned to a byte boundary so
zero padding cannot satisfy a missing read. Truncated fields keep their original
names and prefixes. Integration cases use registered component names/versions
inside entity records and keyframes, with an independent following counter to
expose boundary drift. Unsupported filter tags and invalid radial-menu counts
must stop without reading that next component.

Snapshots explicitly select raw float positions for `low-frequency` and
`managed-navpoint-position-offset`; deltas use recorded region gates and
quantized values under supplied widths. This mode is established by the packet
reader, not inferred from plausible payload values. Snapshot vehicle type state
still requires its unrecorded enable flag and is refused without consuming its
body. The fixture validates that refusal separately; it does not adopt upstream's
snapshot-layout guessing. Delta vehicle type state follows upstream's explicitly
included-component layout. Vehicle physics now reads the explicitly included delta raw/quantized forms;
its snapshot likewise stops for the unavailable enable flag. Dynamic snapshot
positions consume both prefix bits before refusing a nonzero prefix. Object
snapshot positions preserve attached-parent/region gates, and present effect,
crew, respawn and cutscene branches are traced. Shared desired-respawn, crew-order, flock and simulation positions now use
the point grammar, including raw snapshots and default/indexed delta boxes.
Simulation completion is enabled in canonical configuration. A sixth test proves
that raw snapshots and the fixed-width default box need no map context, while
an indexed box without context stops immediately after its recorded gate.
Scalar/masked properties include all 16 tags. These original libfilmshell fixtures
encode masked tags 12–15 with 32 bits; canonical parsing now uses the instruction-
backed four-bit enum width. The four explicit disagreements remain tested here;
[property-v41](../property-v41/README.md) supplies original-Go traces, captured
vectors and canonical boundary/truncation tests for the corrected grammar.
Other shared readers still need the remaining context/branch audit in the ledger.

The biped action cases use the independently authored bodies in
`../actions-v41/wire_cases.py`: every variant 0–25, all generation values for
variant 0, explicit refusals, mixed maximum-length lists and refusal after a
complete action. Both snapshot and delta forms check the first-73-bit mask rule
and following sub-states/tail. Unknown variants cannot be skipped as empty.

These checks establish reader agreement on these authored branches, not complete
format coverage or real-film occurrence. Other row versions, alternative legal
combinations, existing shared readers, creation bodies and missing game context
remain part of the broader robustness audit. Ordinary tests do not run the
separately retained film corpus.

Reproduce from the repository root:

```sh
python3 src/theater/fixtures/components-v41/wire_cases.py
python3 src/theater/fixtures/components-v41/export_reference.py /path/to/libfilmshell > /tmp/component-reference.json
cargo test --offline --lib components::tests
```

Review exported upstream traces before replacing `reference.json`. The exporter
requires the stable Rust toolchain and cached offline upstream dependencies.
Original LevelUp oracle files are unchanged.


## Attached positions, descriptor collisions and orientation evidence

The expanded cases add 160 attached-object layouts (all handle generations,
region gates, finite tails and row versions 0/1/2/4), 40 mutually supported dynamic
orientation layouts, 225 fixed-width alias cases and 36 high-frequency layouts.
All original 696 case and trace objects are unchanged. The original exporter now
passes an explicit fixture archetype (default 0), so high-frequency's archetype-3
26-bit and archetype-4 eight-bit bodies are independently exercised in snapshots
and deltas. Unknown archetypes refuse locally, supported by pinned LevelUp
`components_frequences.go`'s descriptor/vtable analysis. Generic following-byte
fixtures use equipment-being-hacked-component, whose grammar is not overloaded.

`orientation-source.json` is a separate 32-case suite from the instruction-
addressed `components_dynprec_orientation.go` at LevelUp commit
`413971bcc485be53a2fa8cf229e170539a5146f8`, Git blob
`a32dc647e20c3d32146358145ad93b979adbe83e`. Unlike libfilmshell, its disassembly
shows C overriding raw mode (CMOVNZ at 0x140c5f83b onward), and no payload after a
clear delta-roll gate (0x14076e835/0x14076e865). It also establishes the absence of
C below row version 2. We preserve the local reader's existing behavior.
`export_orientation_source.py` verifies the blob, extracts three original decoder
functions and their result type unchanged, and executes them with bounded trace
primitives. Its unselected normal-absolute helper panics instead of supplying a
replacement grammar. The separate source suite has every-bit truncation and
actual DELTA/following-component checks. This is source-analysis evidence, not
proof that the conflicting branches occur in the retained films.

```sh
python3 src/theater/fixtures/components-v41/orientation_source.py
GO_BIN=/path/to/go python3 src/theater/fixtures/components-v41/export_orientation_source.py /path/to/components_dynprec_orientation.go > /tmp/orientation-source-reference.json
cmp /tmp/orientation-source-reference.json src/theater/fixtures/components-v41/orientation-source-reference.json
```

The exporter optionally accepts another fixture path and traces only
FUN_142e29bac at its recorded orientation offset for the engine-1 teleport cases.
That validates the orientation subgrammar, not the entire composed event body.


## Equipment counts and conditional handles

An additional 366 original-libfilmshell traces cover four shared equipment
aliases: 128 tracked-stack layouts, 42 activation layouts, 160 control-signal
layouts and 36 command-tick layouts. Each has delta and snapshot coverage.
Stacks cover all counts 0–15, absent/present/alternating entries, index extrema,
all generations and selected registry versions 0/1/2/4. Activations cover all
eight scalar alternatives and absent/present category-4 handles; control signals
cover all 16 values and handle generations. Command ticks cover one/two entries
and every presence-mask combination. Existing tests apply every-bit truncation,
raw-field/source boundaries and following-component DELTA/keyframe checks.
All previous 1,157 authored case and trace objects remain unchanged.

The stack correction is independently supported by pinned LevelUp
[`equipement_porte_lecteurs.go`](https://github.com/JGtm/LevelUp/blob/413971bcc485be53a2fa8cf229e170539a5146f8/apps/go-api/internal/games/halo_infinite/film/internal/grammar/equipement_porte_lecteurs.go),
Git blob `cc09fe6b2d984591366004954d3fb643eb05cad5`. Its FUN_140f72dec analysis
explains that the pointer advances before comparison with +0x544 + (n+1)*8;
the loop therefore consumes n entries. The four-bit zero count has no handle
body. The old inclusive loop demonstrably consumed the following component's
first bit. This suite compares unchanged upstream readers, rather than using
local parser output as the expected boundary. It does not establish new real-film
occurrences or complete coverage of all component aliases.


## Unit targets, action positions and weapon-state branches

`unit_weapon_cases.py`, called by the main author, adds 930 original-reader
traces across 23 previously untraced aliases in this suite. The cases cover
unit equipment/grenade counts, low-frequency handle lists, property gates and
row versions, camouflage branches, control/aim fields, weapon ammunition and
type-info, plus simple equipment values. The 52 weapon type-info cases include
absent/present identity, single/list magazines, empty/full lists, one/64 frame
flags and conditional axis fields. All previous 1,523 case and trace objects
are unchanged. The complete suite now has 2,453 cases / 180 aliases.

The 122 actor-state cases exercise each of five target slots, both object/full
gates, absent/present optional fields, default/indexed delta positions, raw
snapshot positions and empty lists. The 200 actor-control cases include all
selectors, row versions 0/1/2/4, compressed directions, optional words, category-1
handles of both index widths and 72 action bodies. Actions cover vector modes
0–3, selected per-hand gates and optional aim values. Every new case participates
in the existing primitive-boundary, exact raw-field, serialization, every-bit
truncation and DELTA/keyframe following-component checks. Shared point tests now
include both actor readers: raw snapshots and default boxes work without map
context; indexed points refuse immediately after the recorded gate.

Actor-state's old 16-bit `quaternion` was a dated compatibility exception, not
the canonical game layout. Pinned LevelUp `unit_control.go` (Git blob
`804af70d536d2e420ad8624e5919c53f8d763770`) and
`lecteur_position_exceptions.go` (`699712aa7591344f6b60831506ce1e74e8c14f78`)
identify calls 1422cddc1 / 1422cde0e to FUN_14076e494: 0x10 is a precision level,
not a bit count. The canonical reader now uses the point grammar on both
non-object branches. Upstream traces exercise row version 4; the existing local
8/10/11-bit state-field widths for rows 1/2/3 remain as supported by LevelUp's
threshold table, rather than adopting libfilmshell's unconditional 12 bits.
Those lower actor-state rows are not independently traced by this suite.

Actor-control's action vector mode 0 also calls FUN_14076e494, documented in
`bloc_action.go` (`26a8d2f04d6b80c92b75b33c3c9c276e0192503b`). The enclosing
snapshot mode now selects three raw float words there. Existing explicit
full-precision context for non-snapshot action reads is retained. No other action
gates or tail grammars changed. Two pre-fix regression runs demonstrate wrong
following-component boundaries: actor-state default position (260 vs 277), and
snapshot actor-action position (269 vs 323). The corrected reads match the
unchanged original libfilmshell traces. These synthetic checks establish these
branches, not all legal combinations or complete component coverage.


## Tacmap positions and source-only aliases

`tacmap_cases.py` adds 270 original-libfilmshell traces across seven aliases:
waypoint state, co-op tether area, LOD thresholds, dungeon state, light state,
queued replay mission and back-menu override. The two position readers cover
raw snapshots, default/indexed delta points, zero/full/mixed words and waypoint
row versions 0/1/2/4/7. LOD covers every encoded count 0–31; the 32-player lists
cover all absent, all present and alternating handles. All previous 2,453 case
and trace objects remain unchanged. The upstream suite totals 2,723 cases /
187 aliases.

`tacmap-source.json` contains 81 additional cases for display assets, areas of
interest and POI icons, which are absent from libfilmshell's dispatch. Their
fixed enclosing fields follow the pinned LevelUp instruction-addressed writers
in `lecteur_position_exceptions.go` (commit
`413971bcc485be53a2fa8cf229e170539a5146f8`, Git blob
`699712aa7591344f6b60831506ce1e74e8c14f78`). `export_tacmap_source.py` checks that
blob, extracts those three functions unchanged, and executes their raw snapshot
branches for 27 full-body traces. The legacy delta helper stubs panic; they
cannot silently validate canonical delta cases. The remaining 54 composed delta
bodies follow the documented game call sites, not the original compatibility
reader. Tests compare their actual position bytes and primitive boundaries with
the original-libfilmshell tether traces at the recorded offsets. This proves the
point subgrammar independently; it does not claim upstream full-body agreement
for aliases upstream does not recognize.

All 81 additional cases also participate in exact source fields/serialization,
every-bit truncation, DELTA/keyframe continuation and missing-context tests.
Default boxes and raw snapshots need no map context; indexed points stop after
the gate when context is absent, retaining any preceding fields. All five tacmap
position sites now use precision level 16's point grammar instead of the legacy
six-bit traversal/registry-width guesses. Waypoints retain the additional bit
for registry rows above 1. The independent sources explicitly call the old
readers dated compatibility exceptions; closure counts do not override those
recorded game call sites.

```sh
GO_BIN=/path/to/go python3 src/theater/fixtures/components-v41/export_tacmap_source.py /path/to/lecteur_position_exceptions.go > /tmp/tacmap-source-reference.json
cmp /tmp/tacmap-source-reference.json src/theater/fixtures/components-v41/tacmap-source-reference.json
```

The snapshot continuation test now compares every field with its own authored
source, rather than requiring every fixture's second float word to be the same
NaN. Earlier NaN fixtures are unchanged; new zero/negative-zero/NaN/all-ones
patterns are preserved bit for bit.


## Device components and captured record boundaries

`device_cases.py` adds 452 original-libfilmshell traces across 23 device aliases,
with both snapshot and delta forms. They cover zero/ones/mixed fixed words,
optional group indices, both category-1 handle widths and all generations,
monitor counts 0–8 and rejection at 9/255, absent/all/alternating/single animation
layers, zero/nonzero weights, every two-slot dispenser presence combination,
and timer masks. Selected group/layer cases also use row versions 0/2/4.
All previous 2,723 authored case and trace objects remain unchanged.

Every new case uses the existing primitive/source/serialization, every-bit
truncation and following-component DELTA/keyframe checks. The snapshot test now
recognizes an invalid monitor count as an unsupported component with a retained
eight-bit prefix; it cannot consume the following byte as an entry. These device
readers already agree with upstream, so no production change was needed here.
The suite totals 3,175 cases / 210 aliases; 76 upstream aliases have no cases in
this suite, which does not imply they lack readers or other tests.

A separate [captured device prefix](../device-record-v41/README.md) checks 90
consecutive NEW records against original upstream entry/component reads under
the recorded level context. It stops at an unresolved DELTA instead of inferring
a schema. This does not establish full packet decoding or device semantics.


## Rigid-body points, velocity and projectile branches

`physics_projectile_cases.py` adds 594 original-libfilmshell cases for 15 shared
aliases. The 432 rigid-body cases cover empty/all/alternating/single-body masks,
row versions 0/1/4, default/nondefault orientations, default/indexed/mixed delta
points, and raw snapshots containing zero, negative zero and a NaN payload.
The 162 remaining cases cover compressed/raw velocities, forward/up orientation,
physics-state optional words, projectile rest/tick/bit components, ignored-player
references, playback state and animated-mesh fields. All prior 3,175 case/trace
objects are unchanged. The suite totals 3,769 cases / 225 aliases; 61 upstream
aliases remain outside this fixture set.

The previous rigid-body reader consumed positions using traversal widths and
required context even for an empty mask. It now uses the shared precision-16
point grammar. A new all-body default-box case demonstrates the old boundary
error (312 rather than 616). Every new case participates in primitive/raw-field,
serialization, every-bit truncation and DELTA/keyframe following-component
checks. Missing-context tests include rigid bodies: empty masks, snapshots and
default boxes are self-contained; indexed boxes refuse immediately after their
recorded gate, retaining earlier bodies and orientation fields.

Pinned LevelUp `lecteur_position_exceptions.go`, Git blob
`699712aa7591344f6b60831506ce1e74e8c14f78`, independently identifies the call
at 142f03837 from FUN_142f036f0 to FUN_14076e494 at precision 0x10. Its snapshot
form reads raw R(96). The file labels its traversal-width delta reader as a
dated compatibility exception, retained for historical corpus closure. That
exception is not adopted as canonical format. The source also describes a
non-wire orientation validity check (FUN_140501798) not implemented by these
readers; the fixtures establish structural bit grammar, not that every authored
orientation represents a physically valid transform.


## Shared object-state branches and damage-section refusal

`object_state_cases.py` adds 2,462 original-libfilmshell cases across 13 aliases.
The suite totals 6,231 cases / 238 aliases; all prior 3,769 case and trace objects
are unchanged. Forty-eight upstream aliases remain outside this fixture set.
The new cases cover damage counts 0–32 with absent/all/alternating entries and
every count 33–63 refused; legal region counts 0/1/31/32 with optional values;
all constraint counts and maximum-vitality flags; all dissolver states; body/
shield values and every regeneration gate; default/animated scale; parent-state
row versions 0/1/2/3/4 and archetypes 35/40/43; low-frequency head/count/mask and
tail branches; one/64 frame flags with isolated axis gates; and dead-state
archetype, optional fields, velocity flags and biped-tail differences.

The damage reader now refuses counts >32 immediately after the six-bit count,
as original libfilmshell does, instead of attempting entry gates. Count 33
previously yielded Truncated on the independent six-bit fixture; it now yields
Unsupported at bit 6. DELTA and keyframe integration assert that the following
component is not consumed. All new bodies participate in primitive/raw-field,
serialization and every-bit truncation checks as well as those integration
checks. The other tested shared object-state layouts already agree.

The 32-slot damage cap is pinned libfilmshell admission policy; LevelUp's
`components_object_state.go` (Git blob
`a2779aef639040593435d32694e114753051ffd6`) independently confirms the damage
field widths but omits that cap. This is not an independent game-disassembly
proof of the exact rejection instruction. Region-state admission is a distinct
open issue: that file explicitly notes the game's >32 validity failure, while
both reference readers consume the larger array. These fixtures exercise only
legal region counts and do not certify the unresolved invalid-region policy.
Source structure and validity checks remain separate evidence requirements.


## Non-predicted ability: complete anchor grammar

`ability_anchor_cases.py` adds 920 original-libfilmshell cases: 896 tag-3 bodies
and 24 inactive forms. All eight encoded labels, row versions 0/1/2/4,
optional category-1 references with both index widths and all generations,
parent-local positions, raw snapshots, default/indexed points, six-bit flags,
category-0/5 handles, vector gates and variant tails are covered. The suite
now totals 7,151 cases / 239 aliases; all prior 6,231 case/trace objects remain
unchanged. Forty-seven upstream aliases remain outside this suite.

The old reader's three zero-only bits actually combined the reference gate,
point gate and region index. Its seven-bit field combined six flags with a
handle gate. It recognized only two encoded labels. The new body follows
FUN_142f25e90 and its writer: encoded label + 1, reference and place, six flags,
then the label-specific body. Labels 7/8 have no extra body, and label 6 has
no nine-bit tail. The outer three-bit tail remains governed by row version >1.
The previous code refused the new label-1 default-box case; the corrected
reader agrees with the original trace and following-component boundary.

The pinned LevelUp source independently documents these calls and the earlier
misinterpretation: components_biped_anchor.go (Git blob
52551f8ad7a87851892d2d795ad37a098a8d59c6), components_biped_spartan.go
(973e443021d6048ce4569e11c33924cb23a68162), and default_state_ti41.go
(9bc106223f18a53ade4cac18fd589d9abe1806b4), all at
413971bcc485be53a2fa8cf229e170539a5146f8. The attached place remains two bits,
three 13-bit words and an optional 16-bit word even in snapshots. Unattached
places and the additional destinations use the common precision-16 point reader.

Every new case participates in primitive/raw-field/serialization, every-bit
truncation and DELTA/keyframe following-component tests. Missing-context tests
cover both prefix positions and later destinations. Authored observation data
in each new case independently specifies reference values, field values and
position extents. A separate test checks all 920 internal publications with
and without context. Complete attached prefixes survive a refused indexed
destination; missing positions/flags stay unknown. Internal observation positions
now use source ranges rather than mislabeling all forms as map coordinates;
the old combined field is removed. These callbacks have no current runtime
consumer, so this upgrade does not claim a new grappling trajectory feature.

Predicted ability's source disagreement is deliberately separate: the pinned
initialization analysis contradicts extra target fields read by libfilmshell.
That branch is not changed here. Broader biped/context/validity and runtime
coverage remains under audit.

## Biped points and predicted-ability source conflict

`biped_cases.py` adds 1,708 original-libfilmshell cases. The main suite now has
8,859 cases / 254 aliases, with all earlier 7,151 case/trace objects unchanged.
Thirty-two upstream aliases remain outside this fixture set. Added branches
cover mobility, all posture tags/subtags, control context, map-editor/EMP fields,
desired sets, low-frequency/malleable properties, energy masks and slide.

Mobility snapshots previously read quantized positions: the first new raw
fixture ended at bit 450 instead of 525. Both mobility positions now honor
snapshot precision. Mobility, posture and predicted ability retain self-contained
raw/default-box positions without map context; only an indexed point requires
external widths, after retaining its recorded gate. Explicit full-precision
and baseline switches remain supported. Control context is different: only
the full-precision switch expands its word from two to four bits, independently
of snapshot/baseline mode. Regression tests cover these distinctions.

The main upstream suite includes predicted tags 0/1/2 and compatible tag-3
cases. A separate 140-case `predicted-ability-source.json` covers tag 3 at row
versions 0/1/2/4, absent/present six-bit values (including low bit and bit 4),
and absent/raw/default/indexed points. These are deliberately not certified by
libfilmshell: its extra-target and version branches conflict with the pinned
instruction-addressed initializer analysis. FUN_140f03dfc clears dst[2]/dst[3]
before the branch tests; the optional six-bit value does not enable target
fields. Both row-version paths consume the same body.

`export_predicted_ability_source.py` checks and extracts unchanged functions
from LevelUp commit `413971bcc485be53a2fa8cf229e170539a5146f8`:

- `components_biped_spartan.go`, blob `973e443021d6048ce4569e11c33924cb23a68162`:
  `consumeBipedSpartanAbility`, `consumeSpartanAbilityTag3`.
- `lecteur_position.go`, blob `6033b0366c28cf899883ab35681ba4eb1f5f8b16`:
  `lireE494`, `lireE494Sur`, `lireE524Sur` and their output/table types.

Its bounded reader traces each primitive. Fixed external context selects raw
snapshot points or precision-16 default 22/22/22 and index-0 15/15/17 widths
with a one-bit index. This is an independent branch oracle, not an independent
level-catalogue oracle or an execution of the game binary. Source observations
are disabled; no branch grammar is substituted. The exporter rejects any
authored boundary mismatch. Other biped source references are
`components_biped_ability.go` (`9d77590d181ffec17452f1e68b83b67344991f08`)
and `components_biped_posture.go` (`02ec4e70e144dd6c107e80a0144afde44cc11048`).

All new cases participate in raw-field/serialization, every-bit truncation,
DELTA/keyframe continuation and applicable missing-context checks. Reproduce:

```sh
python3 src/theater/fixtures/components-v41/wire_cases.py
GO_BIN=/path/to/go python3 src/theater/fixtures/components-v41/export_predicted_ability_source.py /path/to/pinned/grammar > /tmp/predicted-ability-source-reference.json
cmp /tmp/predicted-ability-source-reference.json src/theater/fixtures/components-v41/predicted-ability-source-reference.json
```

## Manager, player, stat, text and TLV audit

`managed_cases.py` adds 3,152 original-libfilmshell cases, bringing the main
suite to 12,011 cases /284 aliases. All preceding 8,859 cases/traces remain
unchanged. Only managed-engine timers and game-engine alliances are outside
this main suite; both have independent source traces below. All 286 upstream
aliases now have some fixture coverage across these suites. This is an alias
inventory, not proof of every branch, valid build, runtime context or semantic
interpretation. Unit-command-tick's extra branch remains outside libfilmshell's
known body; the new original-reader cases cover its supported branch only.

New main cases exercise player malleable/frame/aim-assist state, current and
finalized stats, desired masks, team mapping, respawn blocks, current rounds,
RTPC and personal-AI sentinels, music variables, flock danger, navpoint filters,
formatted text and prompts. Counts include 255 navpoint strings, 64 player-frame
flags and all stat value widths. Unknown filter tag 15 retains its prefix and
refuses the following component in DELTA and snapshot records.

`managed-source.json` adds 4,024 cases: 120 alliance, 128 music, 192 timers,
1,536 splash and 2,048 TLV cases. `export_managed_source.py` verifies five Git
blobs at LevelUp commit `413971bcc485be53a2fa8cf229e170539a5146f8`, extracts the
original functions/constants plus two unchanged dispatch arms, and supplies
only bounded primitive bit I/O. The source cases are not certified by
libfilmshell. Reasons for separate source traces:

- Timer and alliance masks are single R64/R32 primitives in the pinned source.
  Libfilmshell splits those masks into R32/R1 primitives. Local source fields
  retain the source grouping; the main trace test is not weakened to hide it.
- Four-field timers use R16, R16, R5, R16. Local code previously placed R5
  last, retaining the total width while mislabeling values. The source test
  failed at primitive end 103; independently authored per-slot values now
  also verify the correction and increasing slot order.
- Music contains two R32-plus-flag pairs. Libfilmshell interprets each flag as
  gating another R32; the pinned dispatch does not. Local grammar is retained.
- Splash's pinned body starts with count3; its later tag3 gates a word.
  Libfilmshell instead reads a preceding kind4 and uses that kind at the tail.
  The existing local grammar agrees with the extracted source.
- Multiplayer properties always consume the mode-2 header and typed fields
  after an absent=0 gate, including variant zero. The pinned source traverses
  fields to terminator 0/1; libfilmshell treats the header as a byte length and
  omits the variant-zero body. The local source-supported traversal is retained.

Source identities:

| File | Git blob |
| --- | --- |
| components_moteur_de_partie.go | 4b6f127a8941c9f16097c7d512c2ec08b49e16c5 |
| lecteur_minuteur.go | dc382adaa1bbfe38e4f8004f094aaf04da636bd6 |
| dispatch_player.go | 3e58e6eb74179f6f69c96e20592b1dfb2c98d2dc |
| components_batch7.go | 73d02925cc5051252605236dfb3290af79c37a84 |
| tlv_mode2.go | 3709d03b18693a463257d899d9b34804ab418b64 |

The TLV tests cover all five-bit wire types, both terminators, extended tags,
integer/body/list/map forms and variants 0/1/5/31. Unknown wire types retain
the source's zero-body traversal; this does not establish their semantics or
prove that every authored variant is game-valid. The exporter executes the
pinned analysis, not the game binary. Cases stay within its resource limits.

Local limits now refuse instead of silently clamping lengths/counts, skipping
deep nonempty scopes, or returning success without a message terminator.
Ten-byte overflowing/nonterminating u64 varints also refuse after the retained
byte. Twenty independently authored DELTA/snapshot refusal cases verify the
exact prefix, every-bit truncation, following-component refusal and resistance
to fallback stubs. Additional tests cover exhausted message fields, legal
maximum counts/u64 values and an empty list at the depth limit. These are
explicit parser resource/representation limits, not inferred game validity
rules. The ordinary source traces do not certify these malformed-input cases.

All main and source cases participate in raw-field/serialization, every-bit
truncation and following-component checks. Reproduce the separate source export:

```sh
python3 src/theater/fixtures/components-v41/wire_cases.py
GO_BIN=/path/to/go python3 src/theater/fixtures/components-v41/export_managed_source.py /path/to/pinned/grammar > /tmp/managed-source-reference.json
cmp /tmp/managed-source-reference.json src/theater/fixtures/components-v41/managed-source-reference.json
```
