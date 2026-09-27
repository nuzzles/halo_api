# Ranked Arena Oddball replay

Match **`4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73`**, stored as
`ranked-arena/02-oddball`, now has a decoded eight-player scene in the replay.
Select **Ranked Arena gameplay → Ranked Arena · Oddball**. It opens first,
with Nuzzles selected. Position, aim, deaths, and respawns cover all three rounds.
The scene now also includes 3,360 firing events, 90 melee events, and partial
health/shield observations for all eight players.
This capture has byte-level checks and API event validation; recorder visual
confirmation is still pending.

## Capture and checks

- Film v41, 63 chunks, 54,840,096 decompressed bytes, duration **1,204.046 s**.
- **72,145 FRAME packets**, **300,430 checked sparse delta prefixes**.
- **296,138 position samples** including 264 spawns; **234,952 aim samples**.
- **248 deaths**, each assigned to one life; all players' kill/death/medal totals
  independently match the API. The film has 734 summary events.
- Every spawn has a checked movement continuation. Maximum distance between
  spawn and first delta is **7.280110 raw units**.
- **233,929** complete supported prefixes immediately precede another checked
  player record. **13,881** prefixes contain later components left undecoded.
- Ordinary respawn delays range **10.019627–10.057633 s**, excluding round resets.

| Roster | Player | Kills | Deaths | Medals | Lives | Positions | Aim |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | Yet | 27 | 27 | 13 | 30 | 38,403 | 32,035 |
| 1 | KOTHA 21st | 29 | 28 | 9 | 31 | 38,972 | 35,185 |
| 2 | Peachcs | 34 | 37 | 14 | 38 | 34,548 | 26,096 |
| 3 | Nuzzles | 33 | 26 | 18 | 28 | 39,731 | 33,794 |
| 4 | GuardedKitten95 | 19 | 35 | 12 | 36 | 35,130 | 21,685 |
| 5 | Nighted Zion | 30 | 32 | 21 | 34 | 36,271 | 28,144 |
| 6 | Vague v1 | 47 | 31 | 27 | 32 | 37,385 | 29,524 |
| 7 | TEED MOBEAST | 29 | 32 | 18 | 35 | 35,698 | 28,489 |

## Clock changes between rounds

The clock remains 37 bits, with its repeated eight-bit counter at bits 29–36.
Its 29-bit prefix is not constant throughout this film:

| First observed film time | Prefix | Source chunk / payload byte |
| ---: | --- | --- |
| 1.198925 | `10100000011110110100001000000` | 1 / 543864 |
| 489.688743 | `10100000011111000100001000000` | 25 / 633967 |
| 840.316209 | `10100000011111010100001000000` | 43 / 510961 |

The later changes precede eight-player spawn bursts beginning at 489.756630 s
and 840.384660 s. These correspond to scene life IDs 97–104 and 182–189.
The initial burst is IDs 0–7 at 3.181620–3.298854 s. Nine surviving-player lives
end at a round reset without a death event; the scene preserves `death: null`
and clears position trails and aim at the new spawn. It does not invent kills.
The first clock appears about two seconds before initial player loading.

## Spawn IDs extend and wrap

The seven-bit serial window documented for Bandit was only the observed range
of that shorter capture. In Oddball, relative to spawn reference offset `s`:

| Relative bits | Observed meaning |
| --- | --- |
| 1–17 | Integer `8704 + wire_id`, with wire IDs 0–255 |
| 18–19 | Tag `01` for the first 256 spawns, `10` when wire IDs are reused |
| 20–68 | Remaining 49 bits of the earlier spawn-body signature |
| 69–73 | Persistent roster index |

The ordinary delta has `8704 + wire_id` at relative bits 0–13 and the same
2-bit tag at bits 14–15. The next two bits remain `00`; count3 and sorted
6-bit component IDs retain the offsets used by the Bandit sparse reader.
Thus the earlier fixed `0100` check becomes `1000` after reuse.

At **1,156.937615 s**, wire ID 0 reappears with tag `10`, now bound to **Peachcs**;
it originally belonged to **Yet** with tag `01`. This spawn is at chunk 58,
payload byte 797761, bit 87. IDs 0–7 are reused before the film ends.
The decoder keys records by `(wire_id, tag)` and exports unique chronological
life IDs **0–263**. Using only the old seven-bit window would alias after 128;
using only an eight-bit ID would bind late movement to the wrong player.

“Generation” describes this observed tag transition. Wrap of the tag itself,
further generations, and a general entity allocation grammar remain unproven.
The previous slot/tag interpretation of the 14-bit header alone does not include
these additional identity bits.

## Two spawn coordinate layouts

Coordinates retain the Ranked **18/18/15** widths. Seven players use the original
coordinate guard at bit 192, X/Y/Z at 263/281/299, and suffix at 314. All 32 lives
of **Vague v1** have an extra **32 bits before the coordinate guard**, shifting
those offsets to 224, 295/313/331, and 346. Roster and identity offsets do not move.
Both layouts use the same exact guards, and every spawn is checked against its
first movement sample. The extra field's meaning is not decoded.

## Reproduce

Run from `experiments/`. If cached chunks and validated rosters already exist,
the scene builder and combat exports can run offline:

```sh
HALO_TIMEOUT_SECS=120 HALO_PROBE_FILM=ranked-arena/02-oddball \
  cargo run --example download_film_chunks
HALO_TIMEOUT_SECS=120 HALO_VALIDATE_STATS=1 HALO_PROBE_FILM=ranked-arena/02-oddball \
  HALO_ROSTER_OUTPUT=films/analysis/oddball cargo run --example film_roster_probe
python3 examples/build_oddball_scene.py
python3 examples/film_firing_probe.py
python3 examples/film_melee_probe.py
python3 examples/film_vitality_probe.py
python3 examples/build_theater_viewer.py
open films/analysis/theater_viewer.html
```

`films/analysis/oddball/` contains `rosters.json`, `scenes.json`,
`decode_evidence.json`, and `delta_evidence.csv`. Each delta retains its source
chunk, payload byte, bit range, wire ID, generation, life, and component list.
Small original-bit fixtures in `examples/fixtures/oddball_records.json` cover
both spawn layouts, all three clocks, IDs above 127, and post-wrap records.

Browser checks exercise all 264 spawns and 248 deaths, movement in every life,
reverse seeking across rounds and ID reuse, late-match playback, and mobile layout.
The existing Bandit, Octagon, motion, aiming, firing, and vitality checks remain.

## Combat and health follow-up

The firing probe now checks eight-bit wire identities, generation reuse, and
sequence wrap. Of 3,374 raw events, seven duplicate copies and seven post-death
events are excluded, leaving **3,360 displayed firing events**. The separate
melee event plus matching companion roster window identifies **90 melee events**.
Both streams cover every player; neither count claims complete hit/bullet accounting.

The vitality probe revalidates the checked sparse prefixes under all three clocks
and both generations. It exports **115,978 shield observations** and **1,442 body
observations**, with **93,393** countdown/clock checks and no state/clock failures.
These are partial observed amounts; immediate damage frames, initial values,
and terminal forms remain incomplete. See [firing](FILM_FIRING.md),
[melee](FILM_MELEE.md), and [vitality](FILM_VITALITY.md) for exact windows.

## Limits

This is a capture-specific partial scanner, not a complete chain decoder.
Unknown components and unchecked forms leave gaps; the viewer labels stale
observations and never connects trails across lives or long gaps. Team membership,
Oddball carrier/scoring and map geometry are not exported for this capture.
Combat and health use separately checked partial exports. Missing health samples
stay unknown or marked as held; the viewer does not reconstruct a complete HUD.
