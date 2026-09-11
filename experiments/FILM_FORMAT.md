# Halo Infinite Theater format: current status and research notes

Pawn component 1 now yields 3D velocity direction and a raw nonlinear magnitude
code, with an explicit short stationary form. Speed conversion is unresolved.
See [FILM_VELOCITY.md](FILM_VELOCITY.md) for bit fields, controls and replay behavior.

> Active lab: 32 replayable films are retained (natural-end and later controls,
> Octagon gameplay, map/appearance/posture controls, two Arena games and the raid).
> Forced-end recordings and standalone probes are
> archived under `archive/cleanup-2026-09-10/` at the experiments root. Historical
> 36-film/30-clip counts below describe earlier validation runs. See the main
> experiments README for current commands. Python field readers used by the
> byte inspector now live under `examples/legacy/`.


**Updated 2026-09-10 · film major version 41.** The current replay shows all eight
players in the Ranked Arena Bandit EVO match, including simultaneous movement and
aiming, deaths, and respawns. The recorder reviewed it and said, “This looks very
good and correct!” This is qualitative visual confirmation alongside the byte
checks and API aggregate validation described below. The replay also includes
the three-round Oddball match, with all eight players, 264 spawns, and 248 deaths;
its visual confirmation is still pending.

The goal is to decode the Theater format completely. We have reproducible partial
extraction and a useful replay; the complete record grammar, map/weapon geometry,
complete shot accounting, damage, and numeric calibration remain open. Supported
parsing is consolidated in [`halo_api::theater`](../src/theater/mod.rs). The
[typed decoder guide](THEATER_DECODER.md) covers the API, native JSON replay,
wasm checks, corpus parity, and performance. Superseded probes are archived;
`examples/` retains the native workflow and inspector's historical field readers.

## Current findings

| Area | Supported result | Remaining limit |
| --- | --- | --- |
| Packet/time structure | 16-byte packet headers, microsecond timestamps, 37-bit clock record with wrapping 8-bit counter | Clock constants and some packet kinds are unknown |
| Position | X/Y/Z widths 13/12/11 (Aquarius), 15/15/17, 17/17/16 (Bazaar), and 18/18/15; external map bounds predict widths | Bounds are not extracted from films; exact Forge/Recharge bounds and other precision levels remain open |
| Velocity | Pawn and projectile direction plus logarithmic magnitude in world units/s; exact zero form and raw codes retained | Other dynamic-precision forms remain unsupported; no position integration |
| Movement input | Two 6-bit axes, neutral 31; every level 0–62 observed with a controller; terminal roster-0/1 inputs bind across Octagon respawns | Physical stick response, other command headers and larger rosters remain open |
| Crouch input | Held/released commands in controls and both Octagon gameplay players, including combined jump/crouch forms | Physical posture and slide remain unresolved; absent samples stay unknown |
| Aim | Cyclic yaw12 and upward-increasing pitch11; decoded alongside movement for all eight Ranked players | Exact angular quantizers, camera/weapon transforms, and exact scoped FOV are unknown |
| Armor | Recorded model/visor/coating identifiers, five attachment TagIds and mythic FxIds; one selected-player equipment panel with offline metadata names | Helmet attachments, armor effects, kits, emblems and some geometry variants remain unknown; schematic body does not render armor assets |
| Pawn component list | Observed generation tag + `00`, count3 at body bit 4, then `n` sorted 6-bit indices at bit 7 | Unknown components still block complete chain walking |
| Player identity | Spawn wire ID maps to roster identity; two record flags precede the 16-bit `8704 + wire_id` field and two-bit generation tag | Raid checks tags `01/10/11/00`; identical wire/tag reuse after a full cycle remains unresolved |
| Lives/events | Bandit: 127 spawns/122 deaths; Oddball: 264 spawns/248 deaths and round resets; per-player event totals match the API | Aggregate validation does not decode shots, damage, or every medal identity |
| Firing | Guarded spawn/roster bindings; 1,579 Bandit and 3,360 Oddball indicators; ID generation and sequence wraps checked | API shot totals differ slightly; exact bullet accounting and hit locations remain open |
| Melee | Event plus companion identity; 105 events across both controls and both Ranked films | Partial activity coverage; hit outcomes and animation duration unresolved |
| Grenades | Guarded throw events and spawn/owner/generation-bound projectile paths in both Ranked games and the control; recorded projectile velocity and rest flags | Partial paths; grenade type, explosions, blast damage and unsupported forms remain open |
| Scope | Stage 0 unscoped, 1 first zoom, 2 second zoom; five BR/S7 transitions and 1,409 Ranked observations | Partial coverage; magnification, FOV, scope-out cause, and other record forms remain open |
| Weapons | Recorded reload starts; 19,723 magazine observations, both carried-slot selections, and seven calibrated 32-bit weapon fingerprints | Partial magazine coverage; starting inventory, manual/automatic cause, energy ammo, reserves, other selection forms, and Shock firing confirmation remain open |
| Vitality | Registry components 4/5 identify body health/shields; checked amount windows and 300-tick regeneration delay | Partial export; scalar calibration, initial/terminal forms, and complete damage-frame coverage remain open |
| Replay | Eight players, aim, trails, selection/focus, death/spawn events, backward scrubbing, persistent combat badges and compact health/shield meters | Gaps are labeled; vitality has partial coverage and provisional scales; avatars, floor, and guns are schematic |

Key corrections to earlier interpretations:

- A firing weapon window combines a carried slot, two guards and a 32-bit
  fingerprint. The same weapon can occupy either slot. The two checked
  component-42 selection forms differ at bit 5, not in their first three bits.
  See [weapon evidence](FILM_WEAPONS.md).

- [Bazaar coordinate evidence](FILM_COORDINATES.md) adds a third width tuple.
  Its spawn prefix matches the original controls, so that prefix cannot identify
  coordinate layout by itself. `Controlled`/`Ranked` enum names have been replaced
  by explicit bit-width names.

- Data after the entity-chain End contains movement input; it is not simply padding.
- Movement inputs have intermediate axis levels, rather than only keyboard endpoints.
- Coordinate widths differ between the controlled map and the Ranked capture.
- Spawn serial and roster index coincided on initial spawns but diverge on respawn.
- The earlier sparse-list parser used the wrong count offset. Sparse pawn lists
  are now checked; the blanket rejection and interleaved-presence hypothesis do
  not apply to these pawn records.

## Documentation map

| Document | Scope |
| --- | --- |
| [Replay guide](examples/theater_viewer/README.md) | Downloads, safe rebuild commands, controls, data sources, and gaps |
| [Recording inspector](examples/theater_inspector/README.md) | Linked byte/text/bit and decoded views, field provenance, exact unknown/opaque coverage |
| [Motion](FILM_MOTION.md) | Original 26-film position/input evidence and checked record boundaries |
| [Controller](FILM_CONTROLLER.md) | Intermediate input levels, stick-calibration limits, position-copy records |
| [Crouch/posture](FILM_POSTURE.md) | Controlled holds, terminal multiplayer commands, Octagon crouch toggles during jumps, and unresolved physical slide state |
| [Aiming](FILM_AIM.md) | Isolated yaw/pitch experiments, recoil cross-checks, angle interpretation |
| [Octagon](FILM_OCTAGON.md) | Initial spawn positions, two-player scenes, first nonempty event validation |
| [Raid](FILM_RAID.md) | Hour-long 20-player replay, continuation spawn flags, alternate suffix, and four generation tags |
| [Ranked/Bandit](FILM_BANDIT.md) | Eight-player sparse components, changing spawn identity, evidence and limits |
| [Ranked/Oddball](FILM_ODDBALL.md) | Three-round replay, changing clock headers, widened and reused spawn IDs |
| [Firing](FILM_FIRING.md) | Firing-event windows, controlled checks, API shot comparison, replay pulses |
| [Melee](FILM_MELEE.md) | Guarded event and companion roster windows, controlled comparisons, replay activity |
| [Grenades](FILM_GRENADES.md) | Throw and clock windows, negative weapon-switch control, recorded projectile paths |
| [Scope](FILM_ZOOM.md) | BR/S7 stages, pawn identity, guarded Ranked observations, and replay state |
| [Weapons](FILM_WEAPONS.md) | Reload-start event, magazine/zero forms, selected slot, weapon-window naming, and replay |
| [Vitality](FILM_VITALITY.md) | Body/shield component names, damage/recovery evidence, raw windows, regeneration countdown |

## Current corpus and artifacts

The [CSV catalog](films.csv) tracks **32 active, replayable films**, including both decoded Ranked
Arena gameplay scenes: Bandit EVO and Oddball.
Run all reproduction commands from `experiments/`.

| Category | Films | Purpose |
| --- | ---: | --- |
| Keyboard controls - natural end | 13 | Original actions; one-minute matches |
| Controller movement | 1 | Left-stick circles with varying tilt, fixed camera |
| Aiming controls | 2 | Isolated pitch up and rightward yaw |
| Octagon controls | 2 | Nuzzles/Yet stationary baseline and AR elimination |
| Weapon controls | 5 | Bandit-to-pistol switch, reload comparison, reported BR/S7 scope stages, BR75/Shock switches, and AR/Stalker firing |
| Ranked Arena gameplay | 2 | Checked Bandit EVO and three-round Oddball captures |
| Octagon gameplay | 1 | Full first-to-50 match |
| Map controls | 2 | Bazaar and Aquarius |
| Armor customization | 2 | Cadet Blue / Cadet Brick comparison |
| Posture controls | 1 | Crouch holds and sprint/slide |
| Raid gameplay | 1 | Hour-long, 20-player raid |

The 13 forced-end controls are archived outside the active catalog. Historical
measurements below retain their original corpus scope.

The Oddball match is `4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73`, stored under
`films/ranked-arena/02-oddball` when downloaded. The existing Ranked capture keeps
its `bandit/01-evo` storage key. Categories are independent of those stable keys.
The frozen original TSV manifests are in `archive/manifests/`.

The controlled-film CSVs contain **4,605 positions, 1,798 aim samples, and 130,112
first-player movement-input pairs**. The separate Bandit scene contains
**157,737 positions** (including 127 spawns) and **125,362 aim samples**, recovered
from 160,809 checked delta prefixes. Oddball adds **296,138 positions** and
**234,952 aim samples** from 300,430 checked prefixes. The composed viewer has
**30 clips, 458,491 player positions, and 362,112 aim samples** in the historical export; some films had no displayable motion/aim
track, and scene files replace matching CSV-derived clips rather than duplicating
them. These counts describe exported observations, not complete frame coverage.

Files under `films/` are gitignored. Important outputs are:

- `films/analysis/all-films/{positions,aim,inputs}.csv`: controlled observations.
- `films/analysis/octagon/`: validated rosters, spawn evidence, and scenes.
- `films/analysis/bandit/`: validated rosters, spawn/life checks, per-record evidence,
  and the Bandit scene.
- `films/analysis/oddball/`: validated roster/events, clock/spawn evidence, per-record
  evidence, and the three-round Oddball scene.
- `films/analysis/theater_viewer.html`: standalone replay with all current scenes.
- `films/analysis/firing/`: event overlays and source offsets; 4,986 displayed firing
  pulses across both Ranked matches, Octagon, the two single-shot controls, and
  the new 15-shot reload comparison.
- `films/analysis/melee/`: 105 checked melee events, companion identity evidence,
  and controlled scenes preserving the original aim observations.
- `films/analysis/grenades/`: checked throw events, unaccepted candidates, source
  offsets, and the two controlled projectile paths.
- `films/analysis/weapons/`: 344 reload starts, 23 magazine and five selection
  observations from the historical probe, byte evidence, and two weapon-control scenes.
- `films/analysis/ranked-ammo/`: native corpus audit for the multiplayer magazine
  extension; the replay reads current per-film `decoded-film.json` files.
- `films/analysis/zoom/`: 1,414 checked scope observations, candidate/source
  evidence, and the BR/S7 control scene.
- `films/analysis/vitality/`: 178,472 shield and 2,010 body observations, registry
  evidence, and countdown checks; supplies the provisional replay bars.

Open the existing artifact with `open films/analysis/theater_viewer.html`, or
follow the [rebuild guide](examples/theater_viewer/README.md). The old Rust motion
probe cannot decode the Ranked coordinate layout. The guide isolates the 34
controlled films before exporting CSVs and uses `build_bandit_scene.py` and
`build_oddball_scene.py` separately.
Do not run an unfiltered old probe over the full corpus into existing CSV outputs.

## Evidence and validation

1. **Bytes:** component/position guards, repeated clock counters, continuation
   boundaries, independent position copies, spawn-to-first-position agreement,
   and no conflicting accepted observations. The Ranked scanner decodes checked
   prefixes; it does not claim to walk every record in the chain.
2. **API:** every player's kill/death/medal totals agree for the two Octagon films
   and both Ranked matches. This checks aggregate summary events, not position or aim.
3. **Browser:** all 391 Ranked spawns and 370 deaths, all displayed firing/melee/grenade/reload
   events, controlled projectile paths, sampled vitality, clearing old-life state, backward scrubbing, compact
   health layout, and earlier clips are checked by `check_theater_replay.cjs`.
4. **Recorder:** isolated aiming and the combined Ranked replay were visually
   confirmed. This supports the displayed interpretation without measuring exact
   world units, angular endpoints, or camera transforms.

Run the committed decoder fixtures with:

```sh
cargo test --offline --example film_motion_probe
python3 -m unittest discover -s examples -p 'test_*.py'
node examples/check_theater_replay.cjs
```

The committed browser driver uses the generated replay and a local Chrome
installation. Reproduction commands and evidence offsets are in the linked guides.

## Original 26-film investigation

The following inventory and historical measurements refer to the original
keyboard corpus, unless an update explicitly says otherwise. Counts such as
101,587 frames or 102 replication chunks are not totals for the current 36-film catalog.
The retracted sections are retained to prevent repeating those inference errors.

26 custom-game clips recorded by a collaborator on a bare Forge map (empty floor,
player spawned at the centre), identical mode and armour, **exactly one player action
different per clip**. Two groups of 13:

- `force-end` — match ended by hand at ~10 s of gameplay, action performed at 5 s.
- `natural-end` — 1-minute mode time limit expired, action performed at 10 s.
  (A 10-second limit turned out not to be settable.)

Match IDs live in [films.csv](films.csv); the original setup notes are preserved
in `archive/manifests/theater_probe_films.tsv`. The corpus is ~101 MB and
lands in `films/`, which is gitignored.

The controlled design helps isolate each action. Incidental motion, timing, and
runtime state also vary, so byte differences alone do not prove semantics. The
registry component-name schema matches across these 26 clips; compare names and
checked boundaries rather than per-session runtime bytes.

## Original exploration tools

`download_film_chunks` and `common::load_corpus` preserve real chunk metadata.
The current download/export workflow is in the [replay guide](examples/theater_viewer/README.md).
The original `film_corpus_diff`, `film_structure_probe`, `film_frame_probe`,
`film_new_record_probe`, `film_entity_probe`, and `film_timeline_probe` remain
useful exploratory examples within their capture assumptions. `film_chain_probe`
is a historical byte inventory whose frame-size-derived lengths are invalid;
its outputs must not override the checked boundaries in the newer decoders.

---

## Tier 1 — High confidence

These measurements concern the original 26-film corpus. Header grammar beyond
the checked Delta/End cases remains provisional, as noted below.

### Chunks

| chunk type | role |
| ---------- | ---- |
| 1 | ECS component registry (schema bootstrap) |
| 2 | replication packet stream |
| 3 | summary events (kills / deaths / medals) |

### Registry (chunk type 1)

- 118 archetypes × 16640 bytes; each archetype is 64 slots × 260 bytes.
- Component name is a NUL-terminated ASCII string at slot offset **+8**.
- **The component schema is identical across all 26 clips** — 118 archetypes, 1067
  total components, same order. Only ~11 bytes per registry differ, at fixed in-slot
  offsets (+76, +77, +97, +194…+201, mostly an 8-byte field at +194), which look like
  per-match runtime IDs.
- Named examples: archetype 3 = `low-frequency` (2 components), 4 = `high-frequency`
  (1 component), 5 = `player-waypoint-component` (27), 7 = `forge-sim-generic-component`
  (64); 0/1/2 lead with `game-engine-team-mapping-component`.

### Packet stream (chunk type 2)

16-byte packet header: LE `u16` type, 2 bytes of unknown meaning, LE
`u32` payload size, LE `u64` timestamp in **microseconds**.
Bytes 2–3 are nonzero in 2,435 of 203,840 packets; the earlier zero-only claim was
incorrect.

**Every byte of every replication chunk parses with zero unparsed tail** across all
102 type-2 chunks. The packet walk is exact.

| packet type | per chunk | size | role |
| ----------- | --------- | ---- | ---- |
| 1 | 1 | 343019 B (fixed) | large mostly-empty buffer |
| 2 | 1 | ~128–130 KB | large, unexplained |
| 6 | 1 | 4 B | — |
| 8 | ~1 | 3437 B (fixed) | short header then zeros |
| 12 | ~1 | 4 B | — |
| 10 | 1 per frame | 5 B | pre-frame marker |
| 0 | 1 per frame | 1–2975 B | FRAME (entity records) |
| 7 | 1 | 0 B | chunk terminator |

- Types 0 and 10 occur in exactly equal numbers (101587 each corpus-wide) and
  strictly alternate `10, 0, 10, 0, …`.
- **Type 10 varies.** `82 00 00 00 00` occurs 101,175 times, `f8 00 00 00 00`
  occurs 331 times, and five other values occur. Its meaning remains unknown;
  the earlier claim that it could safely be ignored was unsupported.
- Frames arrive at ~60 Hz (≈16.7 ms apart).

### Record header model

The library models headers as bit-packed, MSB-first: 1 bit (1 ⇒ `Delta`), else 2 more bits (0 `End`, 1 `New`,
2 `Delete`, 3 `Delta`); then an 11-bit **slot** and a 2-bit **tag**.
`full_id = (tag << 30) | slot`. `(slot, tag)` together are the entity identity —
slots 148 and 1632 each host two distinct entities in the original inventory.
The 14-bit Delta and three-bit End cases have structural checks. The full
`New`/`Delete` grammar has not been established by a complete chain walk. Ranked
spawn extraction uses separate guarded windows rather than this whole model.

### The clock entity's observed counter

`(slot 519, tag 2)` is the match clock and is **exactly 37 bits**.

- Confirmed by the **6-byte frames** (3658 corpus-wide): they hold only this record,
  and `37 + 3` bits of `End` = 40 bits = 5 bytes, with a single `00` byte after. A
  record longer than 45 bits could not fit an `End` inside 48 bits at all.
- Observed layout: 14-bit header, one bit set to 1, then **14 constant bits**
  (value 10304 = `0x2840`, identical within and across films), then an **8-bit
  counter that wraps every 256 ticks**, LSB at bit 36. The constant fields'
  meanings and the clock's archetype binding remain unknown.
- Wrap counts match the prediction exactly: 9 wraps over 2324 records (force-end),
  21 over 5203 (natural-end).
- It is the dominant entity record while the player stands still. The frame also
  contains input data after the entity chain, even when this is the only entity.

### RETRACTED: bytes after the entity chain are padding

The 6-, 9- and 10-byte clock-first frames share an entity chain, but the remaining
bytes belong to an additional serialized section. The common `80 6b ef 80` suffix
contains two 6-bit movement inputs, `(31,31)` for neutral. Walking changes these
to `(62,31)`, `(0,31)`, `(31,62)`, or `(31,0)` according to direction. The input
block shifts to different bit offsets when preceding records change.

See [FILM_MOTION.md](FILM_MOTION.md) for the exact extraction and cross-film
checks. Decoding this section as an entity was a category error; an unfamiliar
slot was never sufficient evidence of padding. The `payload_size` field has not
been shown to over-report meaningful content, and padding has not been shown to
be unbounded. Final alignment and fields after the movement axes remain open.

Do not infer entity-record lengths by subtracting an assumed suffix from the
frame size. Walk independently checked fields and continuations instead.

### Film timeline

The replication stream covers **100% of `film_length`** — load, gameplay and
post-game are all present in the data.

| phase | force-end (40.2 s film) | natural-end (90.1 s film) |
| ----- | ----------------------- | ------------------------- |
| load / startup | 0 – ~16 s | 0 – ~16 s |
| countdown, entities settling | ~16 – 20 s | ~16 – 20 s |
| steady gameplay (556 B/s) | 20 – 26 s | 20 – 76 s |
| **match end marker** | **26 s** | **76 s** |
| post-game tail (540 B/s) | 26 – 40 s | 76 – 90 s |

Pre-game and post-game phases are **identical between the two groups** — same
durations, same per-second entity sets, even the same byte counts (a 767-byte spike
at t=16 s, `1632:3` appearing at t=18 s). Only the steady-gameplay stretch differs:
6 s versus 56 s. The post-game tail is **14 s** in both = ~10 s unskippable Victory
screen + ~4 s transition.

**Gameplay t=0 is film t≈16 s**, confirmed three independent ways:

1. Force-end matches ended by hand at ~10 s of gameplay; marker at film t=26 s.
2. Natural-end matches auto-end at exactly 60 s; marker at film t=76 s.
3. Force-end walk clips (5 s of walking from gameplay 5 s) diverge over film
   t≈21.0–26.0 s — right width, right offset.

### Match end has a detectable signature

At the match-end second: a **byte spike** (888 B force-end, 938 B natural-end, versus
a 556 B steady state), record-1 entities `519:2 1680:0`, and `1632:2` appearing the
following second. Usable as a marker without external metadata.

### Recurring entities in the original controlled recordings

Scanning record 1 of every frame in all 26 films finds **18 distinct `(slot, tag)`
entities**; **13 appear in all 26 films and none appears in only one**. Independent
recordings of the same map and mode produce the same entity set, so cross-film
comparison is valid at entity level.

Record 2 adds at least two more that a record-1-only scan misses: **(324, 2)** with
1001 records and **(256, 0)** with 187. The player **(128, 0)** has 4578 records in
record-2 position versus 26 in record-1. **The 18-entity figure is a floor, not a
total** — a complete inventory needs the chain walked to the end.

### Record bodies have recurring signatures, not one universal player prefix

The player has multiple body signatures. Motion layouts include
`010001000000001100100000…` and `010001100000000000101100100000…`;
the earlier `01000010110011…` observation applies to other layouts and is not
universal. `(324,2)` has a 19-bit constant body prefix followed by 18 varying bits
in the 1,005 records checked by `film_motion_probe`.

### Per-action byte signatures

Frame-byte delta versus the group's do-nothing baseline (force-end):

| action | total Δ | shape |
| ------ | ------- | ----- |
| walk fwd/back/left/right | +4.4k…+6.6k | **+12 B/frame sustained for 5 s** |
| throw grenade | +4919 | +557…+586 per 500 ms bucket, ~1.5 s — new projectile entity |
| jump | +1291 | transient ~1.2 s |
| ping | +407 | small, 3 buckets |
| melee | +354 | brief spike |
| shoot once | +233 | small, ~1 bucket |
| crouch | **+29** | almost nothing — likely one bit in an already-replicated component |

### Summary-event validation

The type-3 chunk is **36 bytes in all 26 original clips**, with an empty event
list. That initial corpus could not validate nonempty summary events. Later, the
Octagon and Ranked captures supplied real kills, deaths, and medals, and all
players' totals matched the API; see [FILM_OCTAGON.md](FILM_OCTAGON.md) and
[FILM_BANDIT.md](FILM_BANDIT.md).

---

## Tier 2 — Probable, single line of evidence

### Interleaved presence bits (historical hypothesis)

**Update:** the Ranked pawn records instead have a sparse list: fixed body prefix
`0100`, a three-bit count at body offset 4, then that many six-bit indices at
offset 7. See `FILM_BANDIT.md` for counter and adjacent-record checks. This does
not establish how other entity types encode their components.

The historical proposal was **one presence bit followed inline by component
data**, with no upfront list. The original supporting arguments were:

- The clock record fits exactly: `14 header + 1 presence bit + 22 data = 37 bits`,
  and archetype 4 (`high-frequency`) has **exactly one component** — one presence bit
  is all it could carry.
- It explains the 22-bit body whose low 8 bits are the wrapping counter.
- It explains the `(1632, *)` result that killed the mask-mode hypothesis: the bit at
  body position 0 varies per record for the same entity, which is impossible for an
  archetype-determined mode flag but *expected* for component 0's presence bit.

This proposal never established a general component grammar. The checked sparse
pawn list is the current basis for further decoding; the clock's own component
encoding and archetype binding remain open.

### Packet type 1 is a pre-allocated, near-empty state buffer

343019 bytes, identical size in every chunk and film. Its 79-byte stride (99.86% byte
equality) is **not** a record size — bit-level equality at period 79 is 99.972%, so
the content is a sparse *repeating bit pattern* whose byte period is
`lcm(79, 8) / 8 = 79`. Split into 79-byte rows: 4342 rows, only **38 distinct**, the
dominant row is **all zeros**, largest run 2791 consecutive zero rows. A 3-byte motif
(`0a 82 80`) slides 2 bytes per row through rows 1024–1049.

"Not an entity table" is high confidence. "Pre-allocated full-state buffer" is the
interpretation.

### Slot bands

Observed slots fall in bands: 128–148, 256, 324–336, 512–595, 1068, 1100–1144,
1318–1340, 1632–1680. Consistent with slot ranges being partitioned by archetype, but
**unproven** — and the one route we tried to test it with is closed (see Tier 4).

---

## Tier 3 — Shaky or retracted (do not build on these)

### RETRACTED: "record-2 lengths are quantized per frame size"

An earlier revision published a table (12 B→51 bits, 14 B→65, 21 B→125, …) and treated
it as ground truth. **Those are frame-size artifacts, not measurements.** The method
searched for an `End` within 8 bits of the frame end, but the entity chain is
followed by a separate input section, so it reduced to `frame_size − constant`;
the "distinct lengths" were distinct frame sizes.

The tell: bucketing the player's records by measured length and printing the body bits
shows short bodies are **strict prefixes** of long ones —

```text
L=28  01000010110011
L=29  010000101100111
L=30  0100001011001111
L=32  010000101100111101
```

— one record truncated at different offsets, not a population of different lengths.
The apparent cross-check (player length 84 pairing with clock-chain 121, both 37
apart) was **circular**: both numbers were measured to the same assumed chain end.

### RETRACTED: specific player record lengths

"Player record = 96 bits" and "= 125 bits" both came from subtracting an assumed tail
from `payload_size`. So did "a 12-byte frame implies record 2 is 24 bits". All void.

### INVALID: component indices from the original mask parser

The following old lists used an unsupported parser and do not establish component
identities. This does not invalidate the separately checked Ranked sparse list:

- "the walking player record has 4 components — indexes 3, 7, 8, 16"
- "`(1632, 2)` = {0,3,25,28,30,44,56}, `(1632, 3)` = {14,19,50,52,55,61}"
- **all archetype lower bounds derived from max index** (e.g. "`(1632,3)` needs ≥62
  components, only 4 of 118 qualify"). That inference chain is broken at its root.

### WEAK: the "coherent kinds at bit 37" argument

Decoding record 2 at bit 37 across 2388 frames gave 2266 `End`, 57 `Delta`, 1
`Delete`, none undecodable, and this was presented as independent confirmation of the
37-bit clock length. On its own it is **weak** — a constantly-zero region inside a
longer record reads as `End` too. The 6-byte-frame argument (Tier 1) is the real
support; the 37-bit length itself is solid, but not for this reason.

### Historical measurement of `(324, 2)` was unreliable

All 221 of its records produced the same measured value, but that measurement used the
retracted method, and it mostly reflects `(324, 2)` always appearing at the same frame
size. What survives: **`(324, 2)` looks like a fixed-length record**, which makes it a
good validation target. **Update:** 51 bits is now independently supported by
1,005 record walks and their following input blocks; see `FILM_MOTION.md`.

---

## Historical refutations requiring reassessment

These are the previous investigation's conclusions, not grounds for permanently
excluding these encodings. The `New` search tested only selected slots and
payloads. Several mask arguments depended on the now-invalid record lengths or
on unproven archetype bindings. The new structural boundaries make retesting
possible. The existing mask decoder remains unsupported on these films.

The Ranked capture now establishes one sparse-list form with the count at a
different offset than the earlier parser used. The following blanket rejection
is historical, not a current conclusion about pawn delta records.

- **Unsupported historical conclusion: `New` records do not exist.** Exhaustive search for `001` + 11-bit
  slot + 2-bit tag for both known slots, across the opening snapshot frame and packet
  types 1, 2 and 8: **zero** hits for the clock slot anywhere. The hits that do appear
  (17 and 51 in the 1.03 Mbit type-2 packet) are *below* the ~63 expected by chance for
  a 16-bit pattern at that length. A body-width sweep (4…32 bits) from two verified
  start offsets never produced even two consecutive `New` records. `FilmWorld::bind`
  had no source from that search. The later guarded spawn extraction supplies
  pawn-to-roster bindings without claiming a complete `New` parser.
- **Packet type 1 is not an entity or binding table** (see Tier 2 for what it is).
- **Superseded conclusion: all upfront lists were refuted.** Over the player's
  4578 records, every variant (`count = field`, `count = field + 1`, index widths
  5/6/7) yields **negative body sizes** (records shorter than their own mask), **262
  distinct component sets for a single entity**, and indexes up to 63 on an entity that
  had no proven archetype binding. The Ranked count at body bit 4 corrects this
  parser shape and validates a sparse list for supported pawn records.
- **"No count field, one 6-bit index" is refuted**: only 4 distinct index sets across
  5766 records, every one mapping to multiple record lengths.
- **Slot → archetype via the mask-mode bit is refuted.** `(1632, 2)` and `(1632, 3)`
  each use *both* "dense" and "sparse" values of that bit across their 52 records, so
  it cannot be an archetype property. (Under the interleaved hypothesis this is simply
  component 0's presence bit — which is evidence *for* Tier 2.)

---

## Open questions, ranked

1. **Generalize coordinate widths.** The Ranked capture uses 18/18/15 versus
   15/15/17 on the controlled map. Recover these from metadata/registry and
   calibrate numeric encoding and world units.
2. **Complete the sparse pawn components.** Position, aiming, the repeated counter,
   and observed body/shield vitality forms are checked; unknown earlier components
   and trailing fields still prevent complete chain walking. See `FILM_BANDIT.md`
   and [FILM_VITALITY.md](FILM_VITALITY.md).
3. **Generalize spawn bindings.** Oddball extends pawn identity checks to 264
   spawns, including wire-ID reuse with a changed tag and an alternate coordinate
   offset. The complete `New` grammar, later tag generations/wrap, and slot →
   archetype bindings remain open. See [FILM_ODDBALL.md](FILM_ODDBALL.md).
4. **Finish the input section.** The extra data after the entity chain includes
   movement axes; decode its prefix, button fields, and final alignment.
5. **Complete shot accounting and damage.** [FILM_FIRING.md](FILM_FIRING.md) now
   establishes guarded firing activity and player attribution. Resolve the small
   differences from API shot totals, late events, sequence gaps, weapon-specific
   behavior, hits, and damage. Kill/death/medal totals remain independently validated.
   Body/shield components and regeneration delay are now identified; initial and
   terminal vitality forms, amount calibration, and event-first damage-frame
   coverage remain open in [FILM_VITALITY.md](FILM_VITALITY.md).
6. **Extend the consolidated library decoder.** `../src/theater/` now implements
   all supported observation families and native replay exports cover 36 films.
   The compatibility `decode_first_delta_header` helper remains provisional;
   `Film::try_from_chunks` uses the checked sparse layout instead.

## Historical next-experiment proposal (not a proof of boundaries)

Record lengths must come from **structure**. The original proposal used a prefix
diff that produced the clock's 37 bits: take the longest common
bit prefix between an idle frame (`clock + End`) and a moving frame
(`clock + player + End`) — the prefix ends where the clock record ends, because what
follows differs.

Apply it one record deeper:

1. Collect frames whose chain starts `clock + player`.
2. Bucket them by the player body's leading bits (stable, per Tier 1).
3. Within a bucket, find pairs whose *later* bits diverge. A maximum common prefix
   is only a candidate boundary: identical leading bits do not establish identical
   complete state, and matching bits can extend into the next field.
4. With one real length in hand, the interleaved model becomes testable:
   `L − 14 − n = Σ present widths`, and differences between records of the same entity
   yield individual component widths.

The new motion probe instead checks repeated counters, subsequent entity records,
and the movement-input block at independently predicted offsets.

## Methodology warnings for whoever resumes

Hard-won, each from an actual mistake in this project:

- **Never infer entity-record length by assuming a fixed frame suffix.** The
  post-chain data includes input fields, not just padding. The previous claim of
  unbounded padding was itself an inference error.
- **Don't hash raw registry bytes to test schema identity** — per-match runtime IDs
  make every film look different. Hash the decoded component names.
  `film_corpus_diff` originally got this wrong and reported all 26 films as mutually
  incompatible, which would have invalidated the whole method.
- **`../examples/position_probe.rs` fabricates chunk timestamps** as
  `(index - 1) * 20_000`. Any timing analysis must use `common::load_corpus`.
- **A single-element list is vacuously "ascending".** An index-ordering test cannot
  discriminate a one-index reading.
- **Beware metrics that reward garbage.** "Same index set implies same record length"
  looked encouraging for every count-based reading (93–97%), but wider index reads
  manufacture more unique sets and a set seen once passes trivially.
- **Force-end matches end at film t≈26 s**, so `film_corpus_diff` divergences there
  for that group are end-of-match jitter, not the scripted action. Natural-end clips
  place their scripted action at t≈26.5 s — same number, unrelated cause.
- Treat zero-component results as a diagnostic, not automatic proof of invalidity.
  The current pawn extractor requires component 25 and cannot establish how
  zero-component records behave in the general format.

Velocity now includes world-speed conversion; see [FILM_VELOCITY.md](FILM_VELOCITY.md).
Projectile field boundaries and ownership evidence are in [FILM_PROJECTILE_MOTION.md](FILM_PROJECTILE_MOTION.md).
