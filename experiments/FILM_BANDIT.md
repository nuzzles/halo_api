# Eight-player Ranked replay: spawn identity and sparse pawn components

Run reproduction commands from `experiments/`.

**Updated 2026-09-10.** After inspecting the generated replay, the recorder
confirmed: **“This looks very good and correct!”** This is qualitative validation
of the combined visualization. It supplements the structural and API checks below
without establishing exact coordinate units, angular endpoints, or complete
format coverage. [FILM_FORMAT.md](FILM_FORMAT.md) indexes all current findings;
the [replay guide](examples/theater_viewer/README.md) covers the full rebuild.

Match `ec02ed9d-346e-4eb0-a491-77e6b557847c` is a real eight-player Ranked Arena
game with Bandit EVO starts, movement, and respawning. The replay now includes
all eight players, their look directions, and repeated death/spawn transitions.

The major format finding is that **spawn identity and player identity are
different fields**. Binding the changing spawn serial to the persistent roster
index makes it possible to follow players through the whole match. A checked
sparse component list then exposes movement and aiming for each live pawn.

## Capture and independent checks

- Film v41; 34 downloaded chunks; 29,998,559 decompressed bytes.
- Duration **622.954 seconds**; **37,316 FRAME packets**.
- **127 spawns**, serials 0 through 126, including eight initial spawns.
- **122 deaths**, all assigned to exactly one decoded life.
- **345 summary events**. Each player's kill, death, and medal totals match the
  match-statistics API independently of the motion decoder.

| Roster index | Player | Kills | Deaths | Medals | Lives | Position samples including spawn | Aim samples |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | diarrhea698343 | 11 | 18 | 3 | 18 | 17,848 | 15,845 |
| 1 | NON STOP BRS | 20 | 11 | 13 | 12 | 22,155 | 13,690 |
| 2 | Yet | 11 | 15 | 5 | 16 | 19,923 | 16,312 |
| 3 | Live4rmda504 | 14 | 19 | 7 | 19 | 17,819 | 9,950 |
| 4 | Miiindful | 19 | 14 | 9 | 15 | 19,972 | 17,540 |
| 5 | Nuzzles | 16 | 14 | 6 | 14 | 20,533 | 17,322 |
| 6 | Relic Future | 11 | 16 | 5 | 17 | 19,010 | 16,497 |
| 7 | ELITExBOSH | 20 | 15 | 7 | 16 | 20,477 | 18,206 |

Totals: **157,737 positions** (157,610 delta positions plus 127 spawns),
**125,362 aiming samples**. These are supported observations, not a claim of
complete coverage of every player at every frame. Colors distinguish players;
team assignments are not inferred from roster order.

> Update: [Oddball](FILM_ODDBALL.md) extends the observed serial range to 255
> and establishes a changing two-bit tag on wire-ID reuse. The seven-bit window
> and fixed `0100` below describe this Bandit capture only.

## Spawn serial → roster identity

Locate the observed 51-bit body signature:

```text
011000111000011011000110000111011010111101101000000
```

If it begins at bit `p`, use `s = p - 18` as the reference offset. All fields
below are MSB-first. Some records are embedded inside larger frame payloads.

| Relative bit | Width | Field |
| ---: | ---: | --- |
| 11 | 7 | Observed spawn serial window, 0…126 |
| 18 | 51 | Body signature above |
| 69 | 5 | Persistent roster index, 0…7 |
| 192 | 63 | Guard: `110000010000110010111111000000000011000011111001101110001001000` |
| 263 | 18 | X |
| 281 | 18 | Y |
| 299 | 15 | Z |
| 314 | 30 | Guard: `000011101010010111001110000001` |

The corresponding ordinary delta header is:

```text
14-bit value = 8704 + spawn_serial
slot = 128 + spawn_serial // 4
tag  = spawn_serial % 4
```

For example, ELITExBOSH initially has serial 7 and roster index 7. After death at
33.820 s, the spawn at 43.844269 s has **serial 8, roster index 7**. Yet's death at
35.356 s is followed by **serial 11, roster index 2** at 45.379412 s.

All 119 respawns follow that player's preceding death by **10.023238–10.056597 s**.
The decoder uses actual spawn timestamps; it does not manufacture a ten-second
respawn timer. Every life has ordinary position deltas, and the first is within
**11.402 raw units** of its independently extracted spawn position. The eight
initial coordinates are separated across the map, strengthening this check.

This corrects the earlier Octagon interpretation of bit windows `[13,18)` and `[69,74)` as
“repeated player indices.” They coincided for the initial two spawns. The seven
bits starting at 11 expose the changing serial in this capture. The complete
spawn header, serial wrap/reuse beyond this range, and general `New` grammar
remain undecoded.

## Coordinate widths vary between maps

The controlled map used X/Y/Z widths **15/15/17**. This capture requires
**18/18/15**, four more bits in total. Evidence includes the four-bit shift in
the following frame counter, the spawn suffix moving from relative bit 310 to
314, continuity when partitioning the 51 coordinate bits, and agreement between
spawn and first ordinary delta coordinates.

Example first-pawn updates:

```text
25.456421  [99111, 131306, 16528]
25.524022  [99109, 131304, 16528]
25.540354  [99108, 131303, 16528]
25.557260  [99107, 131302, 16528]
```

Widths are currently specified for this capture, not decoded from a registry or
map bounds. Coordinates remain unsigned raw values displayed relative to a shared
origin. Their physical scale and absolute world origin are uncalibrated.

## Sparse component list

After the 14-bit pawn delta header, body offsets are:

| Body bit | Width | Meaning |
| ---: | ---: | --- |
| 0 | 4 | Observed fixed prefix `0100` |
| 4 | 3 | Number of component indices, `n` |
| 7 | `6*n` | Strictly increasing six-bit component indices |
| `7 + 6*n` | variable | Component payloads in index order |

The old mask attempt read its count at the wrong offset. Its failure did not
refute sparse lists in general. This layout accounts for the earlier exact pawn
signatures and works across changing player identities and component sets here.

| Component | Observed payload | Interpretation |
| ---: | --- | --- |
| 0 | `00000` + X18 + Y18 + Z15 + `00` (58 bits) | Position |
| 1 | `00` form: 31 bits total; `01` form: 2 bits total | Registry names translational velocity; numeric encoding is unparsed |
| 4 | Observed 11-bit form | Body vitality; amount and state windows in the vitality follow-up |
| 5 | Observed 29-bit form, leading bit 0 | Shield vitality, regeneration-delay countdown, and state windows |
| 21 | `1` + yaw12 + pitch11 + `0` (25 bits) | Aim, using the controlled aiming experiments |
| 25 | `1` + frame counter8 + `0` (10 bits) | Repeated clock counter |

[The vitality follow-up](FILM_VITALITY.md) identifies components 4 and 5 using
their registry names, controlled damage, and Ranked recovery. The earlier zero-bit
guess for component 4 was wrong; the checked observed forms occupy 11 + 29 bits.
Other components and alternate vitality forms remain unparsed. Width alone does
not establish a component's semantics.

## Extraction and evidence limits

`examples/build_bandit_scene.py` finds guarded spawn records, builds lives using
the independently validated events, and scans clock-bearing frames for candidate
pawn headers. Candidates require an active spawn binding, the fixed body prefix,
an ordered sparse list containing component 25, supported component payloads, and
an exact match to that frame's eight-bit clock counter with both surrounding bits.

- **160,809 checked delta prefixes**, with no duplicate player/frame observation
  and no overlapping accepted prefixes.
- **27,872** begin immediately after the 37-bit clock; **132,937** are embedded.
- **152,813** have no components beyond 25. Each ends at the independently
  checked next delta marker; **125,327** end exactly at another player's
  independently checked header and decoded prefix.
- **7,996** contain later components. Only the known prefix through component 25
  is used; its end is not claimed as the end of that record.
- **518 candidates rejected**: 354 unsupported position flags, 90 containing
  component 2, and 74 containing component 15. None of the supported layouts
  failed their aim flags or repeated counter checks.

This is partial extraction by signatures and structural checks, not a complete
chain walk or a false-positive probability proof. It deliberately excludes deltas
after a life's death. Some longer observation gaps remain, including sparse
startup updates and unsupported gameplay states. No extrapolation fills them.

Artifacts in `films/analysis/bandit/`:

- `rosters.json`: film names/events and independent API totals.
- `decode_evidence.json`: all spawn offsets, lives, counts, gaps, and checks.
- `delta_evidence.csv`: each accepted record's chunk, payload byte, starting bit,
  checked ending bit, component indices, position, and aim.
- `scenes.json`: life-tagged tracks and events for the viewer.

## Rebuild and validate

The film is cached under `films/bandit/01-evo` and catalogued in [films.csv](films.csv)
under **Ranked Arena gameplay**. To fetch it in a fresh workspace:

```sh
HALO_TIMEOUT_SECS=120 HALO_PROBE_FILM=bandit/01-evo \
  cargo run --example download_film_chunks
```

Then validate the roster/events and build the scene. If the API-validated
`rosters.json` already exists, the Python builder can use it offline:

```sh
HALO_TIMEOUT_SECS=120 HALO_VALIDATE_STATS=1 HALO_PROBE_FILM=bandit/01-evo \
  HALO_ROSTER_OUTPUT=films/analysis/bandit cargo run --example film_roster_probe
python3 examples/build_bandit_scene.py
python3 examples/build_theater_viewer.py
python3 -m unittest discover -s examples -p 'test_bandit_scene.py'
open films/analysis/theater_viewer.html
```

The HTML builder also reads the controlled CSVs under
`films/analysis/all-films`. See the [controlled export recipe](examples/theater_viewer/README.md#rebuild-controlled-film-csvs)
when rebuilding from scratch. `HALO_MOTION_OUTPUT` applies to the older Rust
probe, which does not support this Ranked layout; it is not a switch for this
Python builder. Use `--corpus` or `--analysis` for the Ranked builder's paths.

The Python tests preserve captured records from multiple identities/component
sets and reject counter, aim-flag, position-flag, and truncation corruption. Browser
checks covered all 127 spawns and 122 deaths, backward scrubbing, clearing old-life
aim/trails, stale positions, the 1 ms kill/death timestamp difference, mobile layout,
and the earlier Octagon/controller/aim/jump scenes. The original checks used a
temporary driver; `node examples/check_theater_replay.cjs` now reproduces them
and adds combat/health checks across both Ranked films.

The viewer opens this match with Nuzzles selected. It has recent trails, player
selection, camera focus, event jumping, and per-player coverage. Spawn clears
previous-life aim; deaths replace the body with a marker at the last observation.
Kill/death display pairing accepts at most 1 ms difference and requires a unique
one-to-one timestamp neighborhood; ambiguous events remain plain deaths.

**Firing follow-up:** [FILM_FIRING.md](FILM_FIRING.md) adds 1,579 guarded firing
pulses for this match, with spawn/player attribution and an independent API shot
comparison. Exact bullet accounting remains open. Map geometry, a Bandit EVO
weapon mesh, bullet paths, and damage are not decoded. The floor, avatar, and gun are schematic. Aim uses the existing
provisional angle scale from `FILM_AIM.md`; look rays do not identify hit points.

**Combat/health follow-up:** [FILM_MELEE.md](FILM_MELEE.md) adds 13 guarded melee
events across all eight players. [FILM_VITALITY.md](FILM_VITALITY.md) documents
partial body/shield observations. The replay shows separate shooting/melee badges
and compact health meters for every player, preserving unknown and older values.
