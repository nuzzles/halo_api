# Upstream refresh: f050371

A fresh remote HEAD query resolved to
[`f050371a6cda83b9887eeb83a1884336fce28e53`](https://github.com/OpenSpartan/libfilmshell/tree/f050371a6cda83b9887eeb83a1884336fce28e53)
(2026-11-10 18:27:19 UTC). It was checked out separately from the earlier
8f0f8d0 source tree. The change spans 72 files and 15 commits, including merges.
Historical wire oracles retain their original revision. The item, medal, damage,
sandbox and options catalogues, and the named variant Bond schema, now use
f050371. The current original readers return identical results for all 62
authored variant cases, 272 script cases and 20 stat-text cases from the older
fixtures; 18 named paths and ten flag mappings are unchanged. This validates
those surfaces, not every possible malformed Bond tree or writer build.

The original `components.rs::Reader::block` was also re-executed at f050371 on
all 12,011 existing authored component cases. Every end bit, completion result
and primitive read/skip trace matched the retained older oracle; the 128 shared
point cases (desired respawn, crew order, flock position/destination and
simulation state) matched as a focused subset. This establishes **no reader
drift on those inputs**, not coverage of newly changed branches: the upstream
diff adds or revises position, parent, control, ability and command-tick paths,
some outside the authored combinations. The retained oracle is historical
validation evidence, not a second selectable implementation revision.

## Wire readers

The ten newly handled upstream event types are 2, 8, 12, 13, 14, 80, 104, 105,
106 and 117. Local code already handles 117. The other nine now use conditional
readers in the existing `frame/events` modules; code 12 replaces an unsupported
fixed 94-bit skip. Their original-source fixture is `events-f050371-v41`:
162 authored lists, each followed by a zoom and terminator, with exact primitive
traces from the new revision. All existing reference-kind rows still agree.

| Code | Source body |
| --- | --- |
| 2 | Width selector and 2/5-bit section value |
| 8 | Six-bit vehicle seat |
| 12 | Two optional players, optional tag, precision-12 point, 19-bit direction |
| 13 | Width selector, 2/7-bit interruption code, flag |
| 14 | 32-bit effect word |
| 80 | Optional tag, two-bit form, zero/one/three precision-16 points |
| 104 | Zero-vector gate, otherwise 19-bit direction and 10-bit magnitude |
| 105 | Optional tag |
| 106 | Precision-12 point, 10-bit direction and nine-bit index word |

Each scalar stays in the canonical source fields. Point readers reuse existing
precision descriptors and fixed default-box widths, with explicit refusal for
missing indexed context. Runtime `actions` exposes the nine bodies as typed
source fields; bounded wire decoding does not invent semantic actors, damage
amounts or gameplay roles.

Additional component changes require their own differential follow-up:

- Source position now distinguishes raw/attached/indexed/default branches and
  closes nonfinite raw positions without the finite tail. Local position readers
  already have broader native-source coverage; compare branch-by-branch rather
  than replacing them with the revised upstream reader.
- Biped command-tick's second form now reads both optional ticks and final flags.
  The local reader already implements the full form.
- Command-state changes the leading handle domain and corrects attached position
  and target suffix conditions. These now use the corrected secondary reader and
  a separately pinned 336-case control oracle, detailed below.
- Grapple object target width is corrected to 13+2 bits. Audit the corresponding
  local branch and source-supported disagreements before changing it.
- Orientation now selects the high form when both raw and high flags are set.
  Compare exact branch evidence and downstream boundaries.
- Multiplayer-properties length arithmetic moves to 64 bits. Local TLV overflow
  and resource-limit behavior must remain explicit.
- Managed-player creation publishes its six-bit participant ID (under 32) for
  contextual identity joins. This is separate from the outer entity ID and from
  proving lifetime/holder attribution.

### Follow-up source comparison after resource-budget implementation

- `orientation::dynamic` already gives the high flag precedence over raw, with
  the separate 32-case `components-v41/orientation-source` oracle. Upstream now
  agrees with that branch; the older oracle pin remains historical evidence.
- `components/unit.rs` already reads the full command-tick form, including the
  same flag, both conditional ticks and final flags.
- The three control-state corrections are implemented in
  `frame/controls/secondary.rs`: the second optional reference uses category 1;
  the local-position bit selects three 14-bit words instead of a world point;
  and the 11-bit target word follows a present handle, with neither field after
  absence. `controls-f050371-v41` contains 336 authored views and original-reader
  traces, each followed by a kind-2 record and terminator. The old 17-case control
  fixture retains all bytes/traces, explicitly marking its three older full-body
  expectations as superseded. The other 14 remain current tests. Both local and
  default-box positions now decode without map context; missing reference domains
  and indexed-point context still refuse after their established prefix.
- Predicted grapple remains a source conflict. The 140-case
  `predicted-ability-source` oracle traces the initializer-aware LevelUp reader:
  its six-bit word does not enable the target branches now adjusted upstream.
  A width correction inside that conflicting branch does not establish that the
  canonical parser should consume the branch. New game/build evidence is needed.
- Raw-position closure needs focused work. The retained LevelUp
  `components_position_i0.go` explicitly describes a finite-value predicate and
  missing two-bit closing fields on some raw/delta paths. Our raw/predicted branch
  likewise returns after its handle tail. Upstream's new finite-raw branch is
  evidence to investigate; do not infer state-dependent delta finitude or replace
  the existing wider position grammar wholesale.
  The native source's `consumeAbsoluSousLaPortee`, `consumeQueueDePoignee` and
  `flottantsFinis` provide a concrete next oracle: the scope form reads its parent
  tail before the predicate, tests the byte-reversed raw words' exponent bits,
  and reports a nonfinite-position stop without the closing two bits. Upstream
  also omits those two bits for nonfinite raw words but returns a completed body;
  that stop-policy disagreement must remain explicit. The same native file labels
  its older inline handle-tail form out of scope, so it does not independently
  certify every raw/delta path's current local layout.
  A direct local call-site review additionally found:
  - `position::component` takes a separate snapshot branch: after two clear
    discriminants it reads `snapshot.flag`, three raw words and unconditionally
    two closing bits. The native scope reader interprets that third bit as the
    parent-tail gate, consumes the tail before checking finiteness, and refuses
    nonfinite words. Upstream's snapshot arm still skips `1 + 96 + 2` unchanged;
    its corrected ordinary raw arm cannot validate this snapshot branch.
  - `object::read_primary` already consumes the category-0 handle and optional
    region for raw `object-position-component`, but also unconditionally reads
    the closing two bits. The same native scope predicate applies there.
  - Snapshot runtime object/vehicle projections deliberately refuse a set
    `snapshot.flag`. Preserve that source field and parent-coordinate refusal
    when correcting the wire reader; decoding a tail does not prove a world pose.
  The next implementation needs an independently traced scope oracle covering
  both parent gates, all generations, the 11-bit region, finite/subnormal/signed
  zero/NaN/infinity words in stream byte order, source truncation, missing global
  reference domains and following component boundaries. Nonfinite refusal must
  survive the component dispatch layer rather than becoming a repairable stub.
  The byte-reversed exponent predicate is now applied to three established raw
  branches: the dynamic snapshot reader with a clear parent gate, its explicit
  writer-absolute/full-scope form with a clear parent gate, and the raw
  `object-position-component` reader with a clear parent gate. Infinity and NaN
  leave the final two bits unread and produce a nonrepairable component refusal;
  finite raw words still consume the closing two bits. Raw fields retain exact
  source bits. The old component oracle's seven synthetic snapshot positions
  deliberately contain a NaN; their former complete outcomes are now marked
  historical, without rewriting the original fixture.
  The proposed refusal at every set parent gate was tested and reverted. A
  retained snapshot at bit 226696 of `snapshot-refusal-v41/packet.bin` has a
  set gate and three finite raw words, yet its independently pinned upstream
  component ends at 226797, after a two-bit footer. Refusing at 226795 also
  invalidated downstream source and runtime tests. The existing snapshot parent
  layouts therefore remain in place. The native full-scope tail note may cover
  a different call site or engine mode; reconciling that distinction requires a
  branch-specific captured oracle and film-derived engine context. The older
  inline tail is also unresolved.
  The explicitly configured **non-snapshot full-scope** path now reads the
  separate native `FUN_14076e3e4` handle tail: one optional category-0 handle,
  then an optional 11-bit region, before checking finiteness and the final two
  bits. A set tail refuses after the raw words while engine type 1 remains
  possible, because its category-0 width may grow. A complete recorded variant
  engine type other than 1 removes that uncertainty; a present handle still
  requires the global reference domain. Authored branch tests cover absent and
  present handles, region/footer/successor boundaries and both context refusals.
  This does not change the disputed snapshot branch or establish its writer
  context on captured films. Both feature configurations pass 563 unit tests
  and ten doctests (14 unit tests intentionally ignored); strict all-target
  Clippy, formatting and whitespace checks pass. The captured-film source audit
  was not repeated because the ordinary production position profile does not
  select this explicit full-scope path.
- Local multiplayer-properties parses full bounded TLV/Bond-like fields using
  a checked 64-bit varint and explicit body/count/depth refusal. This is broader
  than upstream's length skip; existing tests include over-budget bodies and
  64-bit varint overflow. Upstream's arithmetic fix does not require reducing
  local coverage.
- Managed-player creation already retained the six-bit `player_index` word in
  type-9 default-state fields, including values above 31. Complete NEW and
  keyframe records now expose `managed_player_participant()` only for values
  0–31; raw values 32–63 remain inspectable. Replay uses that typed accessor
  when registering a managed team controller. The older type-9 creation fixture
  uses raw value 51, which previously could become an invalid engine-player
  association. A new f050371 original-function oracle covers all 64 values,
  both version forms and eight bit alignments (1,024 cases), plus record
  integration and successor boundaries. This does not by itself join a managed
  object to a life or prove continuous player ownership.

### Captured dynamic-position prefix conflict

The latest upstream reader relabels ordinary dynamic-position prefix bits as
`raw` and `relative`, stopping on the second bit when set. The pinned native
`components_position_i0.go` reads them as `bUsePred` and `bDelta`; its
keep-baseline path consumes the third handle gate and 96 raw source bits under
either second-bit value. A reproducible local 32-film inventory reaches 50
non-`00` ordinary prefixes, all in the raid film: 21 complete/one truncated
`01`, 14 complete `10`, and 14 complete `11`. Two hash-verified captured
snippets have independently traced f050371 outcomes: `11` stops upstream at
2 bits while the local native-source reader reaches 99; `10` reads 101 bits
upstream as a raw position while the local keep-baseline read reaches 99. See
[the fixture](../fixtures/position-prefix-f050371-v41/README.md).
Neither endpoint alone proves later packet boundaries. The local prediction
paths remain in place; copying upstream's new branch would remove native-source
coverage and can relabel a baseline copy as a new world coordinate.

## Newly added robustness policies

Upstream now limits a packet walk to 65,536 packets, inflated/raw chunks to
256 MiB and the entire inflated film to 1 GiB. It bounds reported missing chunk
indices, nested variant depth and caller-supplied coordinate magnitudes. The local decompressor
already limited inflated output. The local parser retains the 256 MiB decoded
chunk ceiling and 65,536-packet ceiling. Packet exhaustion returns a typed
`PacketLimit`, and chunk-local inflation exhaustion returns `InflateError::Limit`
with the manifest index. A later simplification removed aggregate film-byte
accounting and its `ParseBudget` state. These limits are resource policies,
not format maxima or total heap-size guarantees. No marker fallback is introduced.

Local chunk identity is retained in supplied order and never enumerates missing
index ranges. Upstream's bounded missing-index diagnostic does not need a local
analogue. Upstream's new variant readers use depth 32 and checked byte extents;
our shared registry Bond parser already uses checked lengths, per-structure
boundaries, depth 64 and a 100,000-value budget, with explicit partial diagnostics
and overflow/depth/count tests. The policy ceilings differ; neither denotes
the wire-format maximum.

The hint changes validate finite ordered level bounds with spans below 1e6,
filter spawn-pad coordinates outside +/-1e6, and guard calibration grid arithmetic
above 1e9. They do not introduce a general hint-count or recovery-attempt budget.
`MapGlossary::resolve` now rejects caller-supplied calibration spans of 1e6 or
more, so contextual replay cannot admit one. The canonical axis-width calculation
remains unchanged: original-function oracle cases include wider synthetic boxes
whose quantization widths are still source evidence. Built-in map bounds and the
strict admission threshold have tests. The canonical
parser has no spawn-pad fitting input; inferred calibration remains outside
canonical map context. Existing contextual schema recovery remains bounded to at
most 50 candidate archetypes per attempt and is never used by canonical parsing;
this comparison is not a whole-runtime CPU/memory guarantee.

## Broader capability changes still to compare

Broader team/managed-player joining, bot-only cold-start/life handling, scoreboard
round totals, vehicle-part rides, additional mode rules (Attrition, Elimination,
Extraction and Escalation), remaining public playback lookups and catalogue/name additions
are separate runtime/data work. Their semantic values and source assumptions
must be audited against local views; public API count alone does not prove
missing wire data. The reference catalogues now use one f050371 selection;
historical fixtures remain evidence.

One bot-only cold-start packet fixture now proves participant registration from
type 12 without human bootstrap identities. Exact controller ownership remains
unknown until an explicit empty type-8 listing arrives; this preserves the
unobserved-human-population uncertainty. A subsequent controller clear ends an
ownership-started bot life without inventing a death. Complete bot-life and mode coverage
still need their own evidence.

`PlayerReplaySet::vehicle_at(full_entity_id, time)` now supplies a supported
runtime-track lookup, with an exclusive end and no keyframe-namespace match.
Authored native packets and segmented gap/replacement tracks cover its identity
and support rules. This does not implement vehicle-part or seated-pose assembly.

The latest public `life_rides` query now has source-bounded counterparts:
`ride_observations_in_life` retains same-clock diagnostic order and
`riding_support_in_life` yields direct Riding evidence spans with exclusive
life/gap/state-change ends. It checks full entity IDs and ownership starts,
without adopting upstream inferred exits or durations. Native packets exercise
seats, transfers and a gap after positive riding. The retained Octagon and
Recharge replay audit passes, but both captures have zero established Riding
states, so they do not independently prove positive real-film ride attribution.

The f050371 level catalogue adds optional English display names and 33 Halo
service `MatchInfo.LevelId` values. `maps::levels::by_api_id` resolves that
namespace explicitly. The 33 level tags and 65 BSP region calibrations are
unchanged from the prior catalogue oracle; service level IDs do not substitute
for a recorded film level tag or map-variant asset ID.

The current `StatDisplay` mapping and `StatDefinition::shown` arithmetic now
have source-linked runtime counterparts. The recorded sort, display and
precision Bond values remain intact, with missing or malformed fields kept
unresolved. A separate f050371 oracle checks seven display codes and 63 exact
shown-value cases at the precision and i32 boundaries. This supplies the
numeric presentation convention, without inferring score aggregation or
display formatting for time and percentage values.

The scoreboard comparison found that upstream `model/scoreboard.rs::total` sums
last current values over round boundaries inferred from spawn-count clears, with
a match-clock fallback and a three-zero row-clear heuristic. Local statborg
records instead retain each finalized-round mask, independent absent side and
signed 64-bit source word. `ScoreboardSnapshot::finalized_value` now exposes a
selected side and its exact canonical source without collapsing absence into
zero. Existing gap and duplicate-owner rules apply. This is a direct round
observation, not whole-match total parity: absent rounds, owner changes, unread
source and the unresolved timer roles prevent a supported aggregate. Source
records and event boundaries were not changed by this runtime-only addition.
The finalized-value fixture checks selected numeric and absent sides, unset
mask bits, invalid indices, duplicate owners and gap invalidation. The complete
all-feature and no-default-feature suites each pass 558 unit tests and ten
doctests (14 unit tests are intentionally ignored); strict all-target Clippy,
formatting and whitespace checks pass.

`ScoreboardSnapshot::finalized_sum` now adds an explicitly selected inclusive
range from one snapshot. It requires every finalized-round mask bit and numeric
side, reports missing/ambiguous owners and malformed or absent values distinctly,
and returns an `i128` sum with each source-linked word. It does not infer round
coverage, continuity or a whole-match total. After this addition, both feature
configurations pass 562 unit tests and ten doctests (14 unit tests intentionally
ignored); strict all-target Clippy, formatting and whitespace checks pass.

Mode filename inference now includes upstream's Attrition/Elimination,
Extraction and Escalation branches, after the four older arena modes and before
Slayer. Its `reference_choice` and retained competing matches agree with an
unchanged original f050371 classifier over all 2,048 presence combinations of
eleven trigger filenames. The bounded script-data source is still a filename
candidate, not a decoded script load or canonical engine-mode field. The sole
supported option catalogue now uses f050371: 98 hash/name entries, 12 mode-script
groups, and 247 rules. The older fixture remains as historical
validation evidence, not a selectable parser catalogue.

## Unchanged source limitations

The snapshot diff only adds Debug derives. Vehicle `+0x818` construction context
still comes from a guessed branch selected by the next plausible slot; no new
asset physics catalogue or canonical selector was supplied. Types 6/10 still
lack body readers. Recovery still contains scans and candidate-type selection.
The new sources do not justify weakening those canonical refusals or carrying
runtime state across unread bodies.

Relevant exact source blobs:

| File | Blob |
| --- | --- |
| src/events.rs | 4fd94a1480341fa5faf52fd6f481672bb833740f |
| src/components.rs | 9e7f58f027e8ed38c3aa61ebaedf97871fd3d18a |
| src/container.rs | 068885d8c5fdc790ea8efd86e5e4e82873ab97f4 |
| src/film.rs | f07a8dee50f5a8da2eaa9d9e19d1705019b93053 |
| src/creation.rs | b400fd0b8cee51c57cac89145639d266ff3cdcb9 |
| src/snapshot.rs | 94c02bfe1f0c5dfa629d56e6ce9bb7f6f8eef1e3 |

The next audit priorities are component corrections,
then the new identity/mode/data projections. Full canonical decoding and complete
capability parity remain unproven. See VALIDATION.md for event and resource-budget checkpoint results.

## Sequential packet envelopes

Shared transport framing now follows each complete 16-byte header and declared
payload size through zero-size envelopes of any type and past type 7. Unsupported
bodies remain opaque; a malformed or short suffix remains an opaque region.
The latest upstream container also accepts recordings without the optional
terminal highlights chunk. `Film.summary` now records that absence as `None`;
runtime emits no summary events or reports while retaining replication packet
events and source lookup. Inputs without a replication chunk refuse, and duplicate or
nonterminal summaries remain errors. All 32 local corpus films have summaries,
so the absent-section path is backed by upstream source and synthetic integration
tests rather than a captured summary-free film.
Runtime marks a post-type-7 successor as a source-continuity gap even when the
envelopes are byte-adjacent. Synthetic cases establish these framing and
continuity mechanics. The captured 32-film inventory contains only empty,
terminal type-7 packets and no other zero-size envelopes, so it cannot establish
writer legality or state carryover for those new cases.

## Recorded vehicle-part carrier paths

Contextual replay now exposes `ride_carrier` for rides whose direct class-40
parent has a source-backed tag. An explicit attachment path and a compatible
carrier tag are required to resolve a vehicle part to its carrier; direct
vehicle rides need only their supported runtime track and catalogue type.
The synthetic native-packet test injects catalogue tags into the resulting
tracks solely to isolate this join, and covers a part link, a missing link and
a mismatched carrier. It is not evidence that the captured films record those
tags or part attachments. Upstream's snapshot-distance association remains an
inference and is not imported as a canonical relationship.

## Captured snapshot vehicle-context inventory

The current 32-film corpus contains 207 reached vehicle-class snapshot bodies,
including 38 f050371 catalogue part tags. Every body stops later at the
unrecorded `vehicle+0x818` flag, and no decoded table contains a second vehicle.
The inventory counted canonical Film fields without scanning for successor
records. This shows why the upstream
snapshot-proximity association cannot yet be reproduced from complete local
tables. Its later-slot probe is a recovery heuristic, not a recorded flag.
The next parser-level step needs independent asset/writer context or a
source-proven branch selector before a vehicle snapshot can continue canonically.
