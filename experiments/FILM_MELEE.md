# Melee activity: guarded event and companion identity

Run reproduction commands from `experiments/`.

The replay displays **105 checked melee events**. The event occurs once in each
isolated melee experiment and nowhere in the other 30 controlled films. Its
companion roster index agrees with the independently checked spawn binding for
every accepted occurrence, including the two Ranked matches.

| Recording | Events | Controlled event time | Chunk / payload byte / event bit |
| --- | ---: | ---: | --- |
| `force-end/13-melee` | 1 | 21.017151 s | 2 / 479937 / 62 |
| `natural-end/13-melee` | 1 | 26.305848 s | 2 / 494599 / 62 |
| `bandit/01-evo` | 13 | — | See `analysis/melee/evidence.json` |
| `ranked-arena/02-oddball` | 90 | — | See `analysis/melee/evidence.json` |

Each Ranked match has accepted events for all eight players. No accepted melee
events arrive after death, duplicate an event, or disagree with spawn identity.
These checks support an activity indicator; they do not establish complete
coverage of every melee variant, hit outcomes, damage, or animation duration.

## Checked bit windows

Offsets are zero-based, MSB-first, relative to the matched event reference.
The **94-bit event window** is:

| Bit | Width | Observation |
| ---: | ---: | --- |
| 0 | 1 | Not interpreted as an event field |
| 1 | 9 | `101010001` |
| 10 | 8 | Wire identity |
| 18 | 2 | Generation tag: observed `01` / `10` |
| 20 | 6 | `000010` |
| 26 | 40 | Opaque weapon-related window |
| 66 | 28 | `0010110010010110011110011111`, also checked in firing events |

A required **32-bit companion window** begins at event offset **−31**. Its
last bit overlaps the event's uninterpreted leading bit:

| Companion bit | Width | Observation |
| ---: | ---: | --- |
| 0 | 1 | Uninterpreted |
| 1 | 9 | `101001101` |
| 10 | 8 | Same wire identity as the event |
| 18 | 2 | Same generation tag as the event |
| 20 | 7 | `0011000` |
| 27 | 5 | Persistent roster index; checked against the spawn |

These windows establish checked fields, **not complete record boundaries**.
The scene serial is `wire + 256 × (generation − 1)` for the observed generations.
The companion's generic state signature alone is insufficient: similar messages
also occur around reload/cancel behavior. Extraction requires the actual melee
event, suffix, matching companion, and a valid player life.

## Reproduce and inspect

Build the checked multiplayer scenes and controlled CSVs using the
[replay guide](examples/theater_viewer/README.md), then run:

```sh
python3 examples/film_firing_probe.py
python3 examples/film_melee_probe.py
python3 examples/build_theater_viewer.py
python3 -m unittest discover -s examples -p 'test_combat_records.py'
```

The melee probe reuses the firing probe's independently checked spawn/life
export. It writes `analysis/melee/events.json` with player rows
`[film_seconds, spawn_serial]`, `evidence.json` with original source offsets and
validation results, and `scenes.json` for the two controlled melee films. Those
scenes add the checked spawn while preserving all 12 original aim observations.
No stationary track or initial facing is inferred.

The builder validates match, roster, ordered finite timestamps, and life/death
boundaries. Theater Lab's inspector annotates both event and companion windows;
the weapon window remains opaque and surrounding unsupported bits unparsed.

## Replay behavior

Purple **MELEE** badges and a schematic ground arc pulse for **350 ms of film
time**. This duration is a visibility aid. The arc does not imply a decoded strike
direction. Previous/next melee buttons and purple timeline ticks use the actual
event times. Shooting and melee badges light independently during overlaps.

Combat badges and slim shield/health meters remain in every player's roster
card even when another player is selected or an avatar leaves the camera view.
The pulse clears at death or a new life; scrubbing backward restores the same
state. `window.theaterViewerState.players` exposes `melee`, `meleeTime`,
`meleeIndex`, `meleeSupported`, and `meleeRingVisible` for inspection.

Captured fixtures cover both controls, both Ranked matches, generation reuse,
corrupt companion identities, and truncated/event-guard failures. The committed
`examples/check_theater_replay.cjs` checks every displayed combat event, overlap,
life resets, and navigation using the actual generated replay in Chrome.
