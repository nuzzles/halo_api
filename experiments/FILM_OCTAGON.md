# Two-player Octagon replay

Run reproduction commands from `experiments/`.

## Full first-to-50 gameplay

Match `9a875c4e-03bc-4fff-a688-e21216d1618b` is available as
**Octagon gameplay → Octagon · First to 50**. The film is 435.745 seconds long;
its summary records timesknightt winning **50–49** against Nuzzles. Nuzzles has
50 deaths and timesknightt 49. The decoder binds all 100 lives (50 per player).
Nuzzles spawns at film second 3.39; timesknightt first appears at 146.19.

The native export includes 24,283 position samples, 21,310 aim samples, 570
firing events, two melee events, 100 scope observations, 11,957 shield samples,
and eight body-health samples. Body health is especially sparse; gaps remain
unknown. This does not claim a complete shot count, inventory, or damage stream.

Firing now supplies Bandit EVO/S7 weapon-window observations independently of
ammo fields. The export includes 153 weapon-set invalidations and 379 guarded
victim hit notifications. No magazine count, starting weapon, infinite-ammo
display, or damage-driven descope is inferred. The 100 scope rows are unchanged;
the UI labels them as samples. See [weapon evidence](FILM_WEAPONS.md),
[scope limits](FILM_ZOOM.md), and [retrieved variant settings](FILM_SETTINGS.md).

All 100 spawns use an additional 63-bit coordinate prefix at relative bit 192:

```text
110000010000110010011111000000011011000011111001101110001001000
```

It differs from the older controls at relative bit 210 and matches the prefix
in the idle Aquarius film. The Octagon suffix is at 310, giving the established
47-bit **15/15/17** coordinate layout. Across 18,705 independently tick-checked
position prefixes covering every life, 15/15/17 is the only partition with
unit-resolution changes and no jumps of 128 units or more when comparing samples
less than 100 ms apart within a life. The observed maxima are `[21,20,28]`.

Only the prefix whitelist needed extending; no new coordinate variant, match-ID
special case, or gameplay-specific parser was added. Aquarius's 36-bit window is now independently supported as 13/12/11
using map-bound evidence; see FILM_COORDINATES.md. Evidence is retained in
`films/analysis/octagon-gameplay/coordinate-evidence.json` and captured spawn/first
update regression cases in `../src/theater/fixtures/octagon_gameplay_records.json`.

```sh
HALO_PROBE_FILM=octagon/03-first-to-50 cargo run --example download_film_chunks
cargo run --release --example decode_theater_film -- films/octagon/03-first-to-50 --compact
node examples/build_decoded_replay.cjs --corpus films
```

## Movement freshness and terminal command boundaries

The replay marks position stale after 100 ms without a supported sample. Firing,
health and input are separate observation streams, so a shot does not refresh
position. The long gaps seen during combat were primarily decoder rejections,
not an absence of recorded motion.

Two boundary corrections on 2026-09-13 recover the recorded data:

- Pawn component-25 prefixes now accept the shared, fully checked terminal-input
  grammar after End. The old short guard only recognized roster 0 / tag 13;
  Octagon also uses tag 14 and roster 1. Position, aim, velocity and shields in
  otherwise valid pawn prefixes were discarded together.
- Paired commands establish ten-bit nonempty crouch/jump tails. The former
  eleven-bit form included the final zero terminator. Consuming that bit as part
  of the command rejected a following player's header. Terminal nonempty tails
  still require the zero terminator and exact byte padding.

| Octagon observation | Before | After |
| --- | ---: | ---: |
| Position samples | 19,185 | 24,283 |
| Aim samples | 16,533 | 21,310 |
| Shield samples | 9,516 | 11,957 |
| Crouch-input samples | 28,251 | 32,029 |
| Same-life position gaps >=100 ms | 172 | 27 |
| Same-life gaps >=400 ms containing firing | 67 | 0 |

Nuzzles's 257.287–259.089 s interval formerly had no interior position samples
despite five shots. It now contains 93 positions including the endpoints, with
a maximum gap of 49.920 ms. New neighboring observations remain continuous:
maximum per-axis changes within 100 ms are `[12,17,18]` for Nuzzles and
`[13,9,22]` for timesknightt.

The 100 ms replay threshold is unchanged. There are still sparse intervals,
including the long opening, and short unsupported frames. Motion decoding still
requires its strict leading frame clock; observed non-leading clock forms are a
separate lead. Neither activity nor a last-known velocity creates new positions.

Evidence: `films/analysis/octagon-gameplay/motion-gap-evidence.json` compares
same-life intervals before/after. Nine full FRAME payloads and raw position/aim
values are retained in `../src/theater/fixtures/motion_input_boundary_records.json`.
Tests reject corrupt ticks, End, tags, buttons, padding, truncation, appended
bytes and wrong coordinate layouts.

## Earlier controlled recordings (historical Python replay)

The sections below describe the archived presentation. The current native replay
does not use reported initial facing or stationarity to create observations.

**Updated 2026-09-10.** These two controlled clips remain in the viewer alongside
the eight-player Ranked match. The Ranked replay has now been visually confirmed
by the recorder and adds repeated death/spawn handling; see
[FILM_BANDIT.md](FILM_BANDIT.md). Stationary behavior and initial facing in the
Octagon scenes still come from the recording description, not decoded aim fields.

The two controlled Octagon recordings now play together in the 3D toy, with
named player markers, separate look directions, player selection, and a death
marker that follows the timeline in either direction.

| Clip | Match ID |
| --- | --- |
| Idle: Nuzzles and Yet face each other | `10cb8ac3-f0c7-4d71-ae4a-103e4fa5d2b8` |
| Nuzzles kills Yet with an assault rifle | `2b4e00fe-69c9-452e-b1b2-df3a9a0a6b3a` |

Both are v41, seven chunks, and 5,433 FRAME packets. Film durations are 90.976 s
and 90.903 s. Names and XUIDs are recovered from the film roster. The existing
five-bit roster-index extraction yields Nuzzles = 0 and Yet = 1 in both files.

## Initial positions: a new extraction window

**Correction from the Ranked match:** the first field is a changing spawn serial,
not a repeated player index. Serial and roster index coincide for these two
initial spawns. [FILM_BANDIT.md](FILM_BANDIT.md) validates the distinction across
127 spawns and connects each serial to its ordinary delta header.

These stationary captures have no supported ordinary motion updates, so their
positions come from initialization. A recurring frame signature contains these fields:

| Frame-relative bit offset | Width | Observation |
| ---: | ---: | --- |
| 0 | 13 | `1000100010000` |
| 11 | 7 | Spawn serial: 0 or 1 in these two initial spawns |
| 18 | 51 | `011000111000011011000110000111011010111101101000000` |
| 69 | 5 | Persistent roster/player index |
| 192 | 63 | `110000010000110010111111000000011011000011111001101110001001000` |
| 263 | 15 | X window |
| 278 | 15 | Y window |
| 293 | 17 | Z window |
| 310 | 30 | `000011101010010111001110000001` |

These are MSB-first, zero-based **frame offsets**, not a newly decoded general
entity header or a claim about the complete spawn-record length. The 13-bit
prefix and spawn serial must not be interpreted as a complete 14-bit delta
header. The initial serials happen to agree with the roster indices; the first
player's decoded aim during the isolated Nuzzles-shoots experiment provides a
behavioral cross-check on that assignment.

The exact same coordinates occur in both Octagon matches:

| Player | X raw | Y raw | Z raw |
| --- | ---: | ---: | ---: |
| Nuzzles | 16053 | 16410 | 109863 |
| Yet | 16669 | 16410 | 109863 |

Separation is **616 raw units along X**, with no Y or height difference. These
remain uncalibrated units, not meters. The viewer uses Nuzzles' spawn as the shared
horizontal origin and the lowest decoded Z as ground, so it displays `(0,0,0)`
and `(616,0,0)`.

Independent position-copy records in the same frames reproduce all three values:

- Nuzzles: record at frame bit **2103**, observed header `(333,3)`, coordinate
  window starts at 2133.
- Yet: record at frame bit **2704**, observed header `(334,1)`, coordinate window
  starts at 2734.

Each has the previously established 16-bit body prefix `0100001000000000`,
15/15/17 coordinate windows, `00` suffix, and following `000` chain End. The scene
builder checks those fixed signatures, offsets, values, and boundaries. It does
not locate them by subtracting padding from a frame length or scanning for a
convenient coordinate match. The semantic role of the copy entities is unknown.

At the pre-Ranked, 31-film stage, the spawn signature matched **33 frames**.
Every original keyboard film yields the same initial `(16361,16410,109862)` that
was independently established by walking/jumping. The controller and isolated
aiming films yield `(16053,16410,109863)`. The scene exporter currently uses the
extra copy-record checks specific to the two Octagon captures.

## Summary events independently validated

The existing summary-event decoder recovers:

| Film time | Player | Event |
| ---: | --- | --- |
| 25.344 s | Yet | Death |
| 25.344 s | Nuzzles | Kill |
| 77.279 s | Nuzzles | Medal, film code 32 |

The idle film has no events. The match-statistics API agrees with **every player's
kill, death, and medal totals in both films**: one kill and one medal for Nuzzles,
one death for Yet, and all zeroes in the idle match. This is the first nonempty
summary-event validation in this controlled corpus. It validates these examples,
not every possible event layout or medal identity.

The death timestamp also falls between nearby replication transitions at
25.339265 s and 25.353711 s. These are film times, including startup, not the
on-screen countdown. The replay places an elimination marker at 25.344 s.

## Aiming with a second player present

The kill clip yields **135 supported Nuzzles aim updates** with the same 68-bit
layout from `FILM_AIM.md`. Many now precede another player's update:

```text
clock                       [0,37)
Nuzzles aim                 [37,105)
second-player update        [105,161)
End                         [161,164)
first movement input begins 164
```

The added 56-bit layout has header `10001000000001`, followed by the 12-bit prefix
`010000100010` and 30 uninterpreted bits. It occurs 112 times after an aim record
and three times directly after the clock. In all 115 instances the next boundary
reaches End followed by the known movement-input prefix. The
[vitality follow-up](FILM_VITALITY.md) now identifies this as component 5,
shield vitality: its last prefix bit belongs to the component index, followed
by a 29-bit payload containing shield amount, delay countdown, and state windows.
The broader scan also finds eight component-4+5 body/shield updates after shields
reach zero. It checks 150 updates in total, including event-first frames that
this older clock-first parser skipped.

The clock-first parser also encounters consecutive 51-bit auxiliary records:
header `(324,2)` followed by `(325,0)`, each using the existing 19-bit prefix and
18-bit payload. The probe now walks both. Input data has room for two players in
these clips, but the motion probe still exports only the first recognized input
block; it does not claim to decode both players' input streams.

All prior exports are preserved row-for-row: 4,605 positions, 1,482 aim samples,
and 106,728 movement-input rows. The expanded probe exports 1,617 aim samples and
116,058 first-player movement-input pairs across 31 films.

## What the replay represents

- Spawn coordinates are decoded and cross-checked. They are held because the
  recorder explicitly reported that both players stand still.
- Initial look directions are derived from the recorder's **facing each other**
  description and the decoded positions. They are labeled **reported**, not
  decoded aim. Nuzzles switches to decoded aim when samples become available.
- Yet has no supported aim updates in these captures. The visualization does not
  invent a decoded aim trace for Yet.
- A death marker replaces Yet's standing avatar at the verified event time, at
  the last known spawn position. This is not a recovered corpse trajectory or
  ragdoll animation. These two controlled scenes have no decoded subsequent
  respawn; repeated life transitions are handled separately in the Ranked scene.
- The floor and guns are schematic. Octagon map geometry, bullet trajectories,
  damage values, and weapon firing animation have not been decoded. The gold ray
  indicates look direction; it is not a bullet or a recovered hit location.
- [Firing-event extraction](FILM_FIRING.md) now supplies 30 Nuzzles firing pulses
  in the AR clip and none in the idle clip. The displayed muzzle flash is schematic,
  driven by event timing rather than a decoded animation.
- Pitch/world-heading conventions remain the provisional display model described
  in the viewer README. Reported facing, stale aim, missing observations, and
  selected-player coordinates are labeled separately.

## Reproduce

```sh
HALO_TIMEOUT_SECS=120 \
  HALO_PROBE_CATEGORY="Octagon controls" \
  cargo run --example download_film_chunks

HALO_VALIDATE_STATS=1 HALO_TIMEOUT_SECS=120 HALO_ROSTER_OUTPUT=films/analysis/octagon \
  cargo run --example film_roster_probe

python3 examples/build_octagon_scene.py
python3 examples/build_theater_viewer.py
open films/analysis/theater_viewer.html
```

The Octagon builder also needs `films/analysis/all-films/aim.csv`. Generate it
using the [controlled export recipe](examples/theater_viewer/README.md#rebuild-controlled-film-csvs)
if missing. An unfiltered Rust motion-probe run over the current corpus would
also encounter the unsupported Ranked coordinate layout.

The roster probe reads cached chunks; `HALO_VALIDATE_STATS=1` is the only network
step after download. Its offline mode can inspect roster/events without API
access, but the Octagon scene builder requires the validated output. Local files:

- `films/analysis/octagon/rosters.json`: roster, events, per-player API comparison.
- `films/analysis/octagon/spawn_evidence.json`: offsets, timestamps, coordinate
  values, and independent copy-record identities.
- `films/analysis/octagon/scenes.json`: two replay scenes with separate players.
- `films/analysis/theater_viewer.html`: standalone offline replay, opening the
  Ranked match first when that scene exists. Select **AR kill** or
  **Two players idle** for these controlled experiments. If the Ranked scene is
  absent, the AR kill clip opens first.

Validation includes captured-byte Rust fixtures, the cross-corpus spawn audit,
exact preservation of prior CSV rows, the API event comparison, and browser checks
for multi-player rendering, player selection, death timing and reverse scrubbing,
mobile layout, and the existing movement/aim/jump clips.
