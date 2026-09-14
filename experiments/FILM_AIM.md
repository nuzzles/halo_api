# Theater v41: yaw and pitch extraction

Run reproduction commands from `experiments/`.

**Firing follow-up:** [FILM_FIRING.md](FILM_FIRING.md) identifies separate firing
events just before the single-shot recoil samples. The replay's firing indicators
use those records, not pitch changes or kill events.

**Updated 2026-09-10.** The isolated aiming measurements below established the
yaw/pitch interpretation. [FILM_BANDIT.md](FILM_BANDIT.md) now recovers aim
alongside movement for all eight Ranked players through sparse component 21,
across respawns. The recorder visually confirmed both the earlier look-direction
toy and the combined Ranked replay. This supports the displayed heading and
pitch conventions; exact angular quantization remains uncalibrated.

**Octagon follow-up:** [FILM_OCTAGON.md](FILM_OCTAGON.md) adds checked spawn
coordinates for two players, named replay tracks, and the first nonempty
kill/death summary validation against match statistics.

The two isolated aiming recordings identify a **68-bit player record** containing
an absolute cyclic yaw field and an upward-increasing pitch field. The extended
`film_motion_probe` exports `aim.csv` alongside positions and movement inputs.
That Rust probe supports the isolated layout and checked continuations. The
separate Ranked builder handles additional component sets; neither is a complete
decoder for every pawn record.

## Controlled recordings

| Recording | Match ID | Reported action |
| --- | --- | --- |
| `aim/01-pitch-up` | `128deecd-95ba-4887-9d50-660710358c77` | Slowly aim up from 0:55 to 0:45 remaining, reaching the upper pitch limit |
| `aim/02-yaw-right` | `a7861b2f-9435-47d6-924e-4f3bd9555b50` | Turn right with acceleration from 0:55 to 0:45 remaining, completing many circles |

Both films are version 41, with seven downloaded chunks each. There are 5,425
and 5,423 FRAME packets respectively. Film timestamps include startup and thus
are different from elapsed gameplay time or the on-screen countdown.

The decoder extracts these observations without changing signatures between clips:

| Measurement | Upward aiming | Rightward turning |
| --- | ---: | ---: |
| Supported aim samples | 590 | 569 |
| Film-time interval | 22.253074–32.763011 s | 21.668705–31.728975 s |
| Yaw raw range | 0–24 | 0–4095 |
| Pitch raw range | 1024–1510 | 999–1024 |
| Movement input in every aim sample | `(31,31)` neutral | `(31,31)` neutral |

Pitch never decreases in the upward clip. It reaches **1510 at 32.463419 s** and
remains there through the last supported sample, **0.300 s later**, even while yaw
continues changing slightly. This observed plateau is consistent with the
reported maximum upward pitch. Later idle frames do not repeat this aim record;
absence of a record must not be interpreted as a reset to level.

Yaw wraps and continues smoothly in the rightward clip. With shortest signed
12-bit differences, every change is nonpositive, and accumulated motion is
**−32,960 counts**, or approximately **8.046875 rightward revolutions** on a
4096-count-per-turn display scale. Median speed rises from **0.1159 turns/s** in
film seconds 22–23 to **0.9997 turns/s** in seconds 25–30. This is an angle that
accumulates motion, not a stick deflection or angular-speed input.

The small changes on the other axis are preserved, not filtered away: the upward
clip also changes yaw by 24 counts; the rightward clip changes pitch by up to 25
counts. The recordings are not perfectly isolated at the encoded-field level.

## Exact observed layout

Offsets are zero-based, MSB-first, relative to the player record's start.
The controlled-film Rust parser enters this record at frame bit **37**, after the recognized
clock. It only applies these offsets when both header and body signature match.

| Field | Relative bit offset | Width / value |
| --- | ---: | --- |
| Player header `(128,0)` | 0 | 14 bits: `10001000000000` |
| Observed body signature | 14 | 20 bits: `01000100101010110011` |
| Yaw window | 34 | 12 bits |
| Pitch window | 46 | 11 bits |
| Constant observed suffix | 57 | 2 bits: `01` |
| Repeated clock counter | 59 | 8 bits |
| Following bit | 67 | `0` |
| Next record boundary | 68 | End or another supported record |

For the ordinary aiming frames:

```text
clock       [0,37)
aim player  [37,105)
End         [105,108)
input       starts at 108: 1000000001101 011111 011111
```

The 68-bit boundary is supported by the repeated clock counter and the following
entity-chain End and known input section. It is not obtained by subtracting an
assumed amount of padding from the payload size. Every exported aim counter
matches the frame's leading clock counter.

The sparse-list interpretation identifies this record as components `[21,25]`:
14 header bits + 7 list-prefix/count bits + 12 index bits + 25 aim bits + 10
counter bits = **68 bits**. Larger Ranked records can include position and other
components before aim, so the table's absolute offsets do not apply to them.
Their yaw12/pitch11 payload is unchanged and is checked against component 25's
counter. Ordinary pawn headers vary with spawn serial rather than staying at
`(128,0)` for the whole match.

Captured frame at film time 30.010451 s in the upward clip:

```text
a0 7b 42 07 9c 40 08 95 66 00 94 e1 f3 08 06 be f8 00
yaw = 4, pitch = 1336, clock = 243, movement = (31,31)
```

Captured frame at film time 25.005520 s in the rightward clip:

```text
a0 7b 42 06 3c 40 08 95 67 3b 8f 9d c7 08 06 be f8 00
yaw = 2524, pitch = 999, clock = 199, movement = (31,31)
```

## Independent check: older shooting clips

At the 29-film stage, applying the same signature recovered **323 additional aim
samples**, for **1,482 total** across all 29 films. Two particularly useful checks
are the original single-shot recordings:

- Force-end shot, film 21.200559–21.400704 s: yaw stays **3072**, pitch rises to
  **1026**, then returns to **1024**.
- Natural-end shot, film 26.338777–26.489008 s: yaw stays **3072**, pitch begins at
  **1026** and returns to **1024**.

These small vertical excursions are consistent with firing recoil and recovery,
independently supporting aim/view orientation rather than raw hardware input.
Other recovered samples include small changes in the old “do nothing” recording
and post-game changes, so the experiment labels do not imply bitwise immobility.

The prior **4,605 position CSV rows are exactly unchanged**. All **97,017 prior
movement-input rows** are retained with identical values; newly supported chains
add input observations. Across the expanded corpus the probe walks 112,332 of
117,847 frames, exporting 106,728 movement pairs. Unknown layouts remain skipped.

## Angular interpretation and limits

- Yaw is cyclic and decreases for right turns. For closely spaced samples,
  `delta = ((yaw_now - yaw_previous + 2048) % 4096) - 2048` unwraps it.
  Multiplying by `360 / 4096` is a useful approximate degree scale.
- Pitch **1024** is the observed level reference. Increasing pitch means looking
  upward. **1510** is the limit observed in this recording, not the maximum value
  representable by an 11-bit field.
- Exact quantizer endpoints, rounding, and pitch-to-degree conversion remain
  unproven. A full-turn pitch model would put the observed excursion near 85.4°,
  but that is a hypothesis; the CSV and trace deliberately retain raw pitch.
- The old forward-walking direction (negative extracted Y) and shooting yaw 3072
  support yaw 0 along positive extracted X. The Ranked match now supplies combined
  movement/aiming with recorder visual confirmation. Absolute camera transforms
  and exact angular endpoints still need numeric calibration.
- The isolated aiming CSVs have no exported position updates, so those scenes
  still use a labeled schematic pivot. A later controlled spawn audit found their
  initialization coordinates, but the isolated scenes do not yet use them.
  The Ranked scene has separately decoded position and aim streams for each life.
- The role is aim/view orientation. A separate weapon-barrel transform, camera
  position and animation/recoil components are not decoded here. Scope stages
  are now decoded separately; see [scope evidence](FILM_ZOOM.md).
- 20 pitch-clip and 19 yaw-clip frames also contain the same candidate aim signature
  after a different preceding chain. They are excluded from CSV counts because
  that preceding chain is not supported. Searching for a signature is not used
  as a substitute for walking the chain in this Rust probe. The separate Ranked
  scanner accepts guarded prefixes with active spawn bindings, supported component
  fields, and counter checks; it explicitly does not claim complete chain walking.
- In arbitrary films, long gaps can hide entire turns. The controlled yaw clip's
  exported adjacent samples are at most 50.165 ms apart; shortest-path unwrapping
  is supported here by continuous motion and the reported turn direction.

## Reproduce

```sh
HALO_TIMEOUT_SECS=120 \
  HALO_PROBE_CATEGORY="Aiming controls" \
  cargo run --example download_film_chunks

python3 examples/plot_aim.py --directory films/analysis/all-films
open films/analysis/all-films/aim_trace.html

cargo test --offline --example film_motion_probe
```

Before plotting, generate the combined controlled CSVs using the
[replay guide](examples/theater_viewer/README.md#rebuild-controlled-film-csvs).
Do not run the old motion probe unfiltered over the full mixed corpus: it cannot
decode the Ranked coordinate widths. The combined export includes both isolated
aiming films, the original shooting checks, and the Octagon aim continuation.

The Python analysis checks monotonic pitch, the observed pitch plateau, rightward
unwrapped yaw, acceleration, neutral movement inputs, and recovery after shooting
in both older clips. The Rust fixture tests use captured frame bytes and check
that counter corruption, a broken next-record boundary, and truncation fail.

Outputs: `aim.csv`, `aim_summary.json`, and the standalone interactive
`aim_trace.html`. The trace plots cyclic yaw, raw pitch, and derived turning speed;
its cursor reports actual decoded samples and does not interpolate across gaps.
