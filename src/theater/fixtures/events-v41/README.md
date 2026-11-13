# Event reader differential fixtures

Reference: OpenSpartan/libfilmshell
`18067987ebc3628ad499c819c6aee814692df43b`.

`wire_cases.py` independently authors 1,494 event lists. These cover conditional
pickup, reload, melee, impact and dialogue bodies; newly supported map-independent
events; all possible early-NUL positions in a 16-byte string and an unterminated
full-length string; game-event scalar values/parameters; and teleport, respawn,
AI-jump and game-event positions; and all NetworkedActionRequest variants. Default-box and indexed-box forms are separate.
Raw respawn float words retain negative zero and a NaN payload. Fixtures contain
synthetic source fields only, with no real player data.

Every complete body is followed by a known zoom event and the list terminator.
Unsupported-body cases stop after their known reference fields or action prefix.
`export_reference.py` archives the pinned upstream commit into a disposable
checkout, instruments `events::Bits::bits` and the component reader’s primitive
reads/skips, and calls the original event walker. Event 102 reaches the original
shared `components::action_end`. Authored endpoints, completion and record counts must agree before
export. It also exports the unchanged `handle_kinds` table for all 123 event
codes. Neither expectations nor source bytes are generated from halo_api.

The 113 action bodies are shared with the independent component fixture author.
For variant 18’s refusal, upstream’s parent cursor stays before the nested flag,
while its trace records that flag. `reader_end` and `end` retain both facts; our
canonical endpoint includes the established prefix. No upstream branch or return
value is changed to obtain the trace.

The event tests check primitive boundaries, source fields and serde; truncation at
every bit with no usable byte padding; missing/invalid map context; runtime projection of default-box projectile positions; and actual
Film/frame integration with following entity and control views. Default-box
points need no map descriptor. Indexed points require an explicitly supplied
descriptor and matching region; the gate is retained before a missing-context
stop. Contextual frame decoding is exercised separately from canonical Film
parsing, whose default configuration supplies no external map widths.

The 32 projectile cases cover precision-15 detonation and precision-12 impact
points, including fixed default-box words (21 or 18 bits per axis), indexed
positions and optional prefix/tail fields. Default boxes decode without map
context and do not become map-region coordinates in runtime. The detonation
cases select the eight-bit tail branch shared by both readers; upstream's fixed
eight-bit tail does not validate the locally retained conditional 19-bit branch.

These traces establish agreement on authored branches, not full event coverage.
The conditional projectile tail is covered by original-source traces below.
Broader event-version and snapshot/full-precision admission remain under audit.
Multi-region catalogue fixtures live separately in `events/regions.rs`. Script's recorded context
is covered separately below; upstream's event-15 assumption alone does not
establish its conditional gate. The original LevelUp fixtures remain unchanged.

The 415 `fire_cases.py` bodies cover short/block/timestamp modes; all prefix
gates; shorthand/explicit counts including 0, 1 and 15 barrels and 0, 1, 2, 3
and 15 markers; marker kinds and out-of-list indices; both barrel vector widths;
last-present-barrel ordering; both target discriminants, handle forms and
optional marker values; all aim-vector gates; block selectors; both point sites;
and optional tails. Each is traced through the original upstream event walker.
Configuration 1 precedes the list explicitly; the continuation bit is not used
as a substitute. The truncation harness preserves configuration while shifting
the event start, and configuration 0/missing-v3 admission have separate negatives.
Fire layout admission is now distinct from its optional position descriptor.
All default-box and map-independent cases work with no map widths; indexed cases
refuse at the recorded gate. Canonical header-derived admission is tested in
`bootstrap-v41`.

Reproduce from the repository root:

```sh
python3 src/theater/fixtures/events-v41/wire_cases.py
python3 src/theater/fixtures/events-v41/export_reference.py /path/to/libfilmshell > /tmp/event-reference.json
cargo test --offline --all-features --lib events::tests
```

Review the independently exported traces before replacing `reference.json`.
The exporter uses the stable Rust toolchain and cached offline dependencies.

## Damage flags and remaining map-independent bodies

The additional 691 cases preserve all previous 803 case/trace objects exactly.
They cover all 13 upstream empty bodies in three reference forms; all presence
gate combinations for damage-section response (1), throw (39) and AI dialogue
(75); and 568 damage-aftermath (0) bodies. Damage cases exhaust the eight
independent optional gates with/without the f58 byte, retain every other f58
value, and toggle each of the first fourteen flags in both directions.

The original libfilmshell trace failed at the first flag boundary (bit51) before
the local damage reader was corrected to preserve each flag as a separate field.
The pinned original LevelUp `vue_a_charges.go::chargeDegatsApres` also specifies
fifteen separate bit reads; only the fifteenth gates the following word. This
does not change complete-body widths or event boundaries. Truncated packets now
retain each fully recorded flag instead of losing a partial fourteen-bit group.
The body grammar and field names used by the separate runtime damage projection
remain unchanged.

The main suite now traces 42 of the 44 upstream body codes. Script (15) has the
separate recorded-context fixtures below; kill (85) has historical and variant/
playback-context tests. This is a dispatch coverage statement, not proof of all
versions, branch combinations or game-valid records.

## Original-source projectile tails

`projectile_source.py` authors 144 additional event lists: 96 detonations and
48 impacts, including all prefix gates, default/indexed positions, three
reference forms and both 8/19-bit detonation direction tails. Every body has
a following zoom and terminator. `export_projectile_source.py` executes the
unchanged original `chargeDetonation`, `chargeImpact`, variant/tag helpers and
position readers at LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`.

The exporter checks these original Git blobs before extracting functions and
constants:

| Source | Blob |
| --- | --- |
| vue_a_charges_armes.go | 2a1a7be5519ca9e483247ad587781adc7bc33dd0 |
| lecteur_position.go | 6033b0366c28cf899883ab35681ba4eb1f5f8b16 |
| bit_leaf_readers.go | a0bbe36af9f333cc35d5b9f3cd42307097ccb0e2 |
| vue_a_charges.go | d022e4cd4b92c755c4210fed655423129ecc175e |

Only bounded primitive I/O, delta-mode/precision-table context and observation
sinks are supplied. Explicit table widths are not an independent map-catalogue
oracle. The source reader proves the recorded tail gate chooses 8 or 19 bits;
libfilmshell's fixed eight-bit tail cannot establish the latter. These separate
traces preserve that disagreement without modifying upstream branch grammar.
Tests check source words/serde, every-bit truncation, both tail values and
primitive boundaries, missing/invalid map context, map/default-box runtime
projection, and Film/event/entity/control continuation. They do not establish
snapshot/full-precision event admission or additional builds.

```sh
python3 src/theater/fixtures/events-v41/projectile_source.py
GOCACHE=/tmp/halo-go-cache GO_BIN=/path/to/go python3 src/theater/fixtures/events-v41/export_projectile_source.py /path/to/pinned/grammar > /tmp/projectile-source-reference.json
cmp /tmp/projectile-source-reference.json src/theater/fixtures/events-v41/projectile-source-reference.json
```

## Script writer context

`script_cases.py` authors 32 additional complete event lists in `script.json`:
both optional-prefix forms, absent/present references and payload lengths
0/1/7/8/31/64/511/1023. Each includes a following zoom event, terminator and empty
entity/control views. `export_script.py` uses the original event exporter to
trace the 16 prefix-present cases only. The 803 earlier event cases/trace bytes
remain unchanged; the shared exporter still runs those by default.

The gate comes from JGtm/LevelUp
[`vue_a_charges_execution.go`](https://github.com/JGtm/LevelUp/blob/413971bcc485be53a2fa8cf229e170539a5146f8/apps/go-api/internal/games/halo_infinite/film/internal/grammar/vue_a_charges_execution.go):
writer `FUN_142eec4d8` emits 15 bits when recorder simulation != 2, then 13 bits,
a 10-bit payload bit-count and the payload. The `vue_a_versions.go` native table
records version 1 for Script. The playback reader uses its own, different
simulation predicate; the canonical reader follows the recorded writer value.
Valid simulation values are 0–3. Unsupported versions, unavailable/invalid
simulation and incomplete canonical options refuse before the body.

The no-prefix branch is independently authored from the pinned writer analysis;
it is not validated by libfilmshell, which always reads the prefix. All 32 local
films record simulation 3 and script version 1; no captured simulation-2 claim
is made. Tests cover every-bit truncation, source scalar/opaque field ranges,
serde, original upstream boundaries where applicable, missing context and actual
Film/event/entity/control continuation. Replay uses the same recorded gate.

```sh
python3 src/theater/fixtures/events-v41/script_cases.py
python3 src/theater/fixtures/events-v41/export_script.py /path/to/libfilmshell > /tmp/script-reference.json
cmp src/theater/fixtures/events-v41/script-reference.json /tmp/script-reference.json
cargo test --offline --all-features --lib context::events
```


## Engine-1 teleport orientation

`teleport-engine-one.json` has 48 authored complete event lists: three reference
forms, both orientation gates, both independent flags, absent/default/indexed/
mixed point pairs, and a following zoom event. The pinned LevelUp
`vue_a_charges_execution.go` documents that FUN_142efa2a8 / FUN_141f86118 selects
FUN_142e29bac for recorded engine type 1. Its body is mapped and implemented in
`components_dynprec_orientation.go`: inverse-gated 30-bit direction, then 30-bit
roll. `teleport-engine-one-reference.json` executes that original decoder only,
with its exact source Git blob checked; it does not claim the original event
walker supports engine 1 (it refuses it).

Tests check every-bit truncation, orientation primitive boundaries, event/next-
event ends, both runtime-default settings, and missing-variant refusal before
orientation. A complete synthetic Film with recorded engine-1 variant verifies
continuation into the entity/control views without assumptions. No captured
engine-1 film is present in the retained corpus.

```sh
python3 src/theater/fixtures/events-v41/teleport_engine_one.py
GO_BIN=/path/to/go python3 src/theater/fixtures/components-v41/export_orientation_source.py /path/to/components_dynprec_orientation.go src/theater/fixtures/events-v41/teleport-engine-one.json > /tmp/teleport-reference.json
cmp /tmp/teleport-reference.json src/theater/fixtures/events-v41/teleport-engine-one-reference.json
```
# Direct semantic outputs

`semantics-reference.json` retains the unchanged libfilmshell event reader's
body/reference values for all 1,473 complete main fixtures plus the two
`semantic-extra.json` kill bodies (absent/present player gates and full-width
percentage extremes). The extra cases are synthetic and not gameplay samples.
`export_semantics.py` uses a Git archive of commit
`18067987ebc3628ad499c819c6aee814692df43b`, adding serialization derives and a
test harness only. It does not call halo_api or change the upstream readers.

```sh
python3 export_semantics.py /path/to/libfilmshell > /tmp/semantics-reference.json
cmp semantics-reference.json /tmp/semantics-reference.json
```

Runtime tests compare the common source subset across all 18 typed upstream
body families. Intentional representation differences remain explicit: source
relative reference indices, exact text length, all game parameters and original
point codes/widths are retained locally. The 144 original LevelUp projectile
cases and engine-1 teleport fixtures also exercise the direct view without
pretending libfilmshell's differing layouts are authoritative for those paths.
These tests establish direct field interpretation, not inferred actor ownership,
coordinate calibration, mode semantics or full match-event coverage.
