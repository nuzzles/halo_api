# Controller follow-up: intermediate movement axes confirmed

Run reproduction commands from `experiments/`.

**Updated 2026-09-10.** The counts below describe the controller experiment and
the 27-film corpus at that stage. For current totals and the visually confirmed
eight-player replay, see [FILM_FORMAT.md](FILM_FORMAT.md) and
[FILM_BANDIT.md](FILM_BANDIT.md). The Ranked extension does not yet decode all
players' input streams or calibrate joystick pressure.

Match: `df0fadd0-5d20-4f4a-ae99-e1f62656c502`.

The recorder used **left-stick circles with varying tilt and a stationary camera**
from 0:55 to 0:30 remaining in a one-minute custom match. The downloaded film is
version 41, 90.754 seconds long, with 5,412 frames and seven chunks.

## Result

The same two 6-bit movement fields identified in the keyboard corpus each take
**every integer value from 0 through 62** in this clip. They are not boolean
direction flags. Neutral remains `(31,31)`.

The probe extracts 4,464 input pairs and 1,562 player position samples. Observed
nonneutral inputs span **film time 21.975–46.701 seconds**, consistent with the
reported gameplay interval plus startup. Using a 21–47 s analysis window captures
1,501 decoded input samples and **329 distinct pairs**. The decoder skips unknown
layouts, so this is not complete frame coverage.

Example consecutive inputs at movement onset:

| Film time (s) | Forward/backward | Left/right |
| ---: | ---: | ---: |
| 21.975 | 30 | 28 |
| 21.991 | 27 | 14 |
| 22.010 | 24 | 0 |
| 22.025 | 25 | 0 |
| 22.043 | 27 | 0 |
| 22.058 | 28 | 0 |
| 22.076 | 31 | 0 |
| 22.092 | 35 | 0 |

This confirms intermediate **encoded axis values**. It does not by itself
calibrate physical stick deflection: rotating a stick at full tilt also produces
intermediate values on each individual axis.

## Why light-pressure scaling remains open

Despite the reported varying tilt, most decoded sustained samples lie near the
outer boundary. Excluding the first and last 0.2 seconds of observed nonneutral
input leaves 1,408 samples:

- 1,265 have at least one axis at exactly 0 or 62.
- Their raw radius about `(31,31)` ranges from 30.594 to 43.139.
- Dividing each centered axis by 31 gives a convenient display scale, but its
  Euclidean magnitude can exceed 1. It is not established as the physical stick's
  radial deflection.

Intermediate axis values are encoded, but this capture does not establish which
value a light forward tilt produces. Dead zones, response curves, saturation, or a
different movement-vector representation remain possible explanations. These
fields may represent processed movement rather than raw controller hardware.

A discriminating follow-up would hold one direction at several **steady low and
medium deflections**, with neutral intervals between them and the camera fixed.
That separates magnitude changes from the axis changes caused by rotating a stick.

## An additional record with matching position fields

The original probe stopped when a recognized player record was followed by entity
`(333,1)`, which was absent from its supported grammar. This was an unsupported
continuation, not a failure of the player's measured length.

All 808 observed instances of this additional layout have:

| Field | Offset relative to record start | Width |
| --- | ---: | ---: |
| Header `10010100110101` | 0 | 14 |
| Constant prefix `0100001000000000` | 14 | 16 |
| x window | 30 | 15 |
| y window | 45 | 15 |
| z window | 60 | 17 |
| `00` | 77 | 2 |

Total: **79 bits**. At the predicted next boundary, `000` is followed by the
recognized movement-input prefix in every instance. In 777 instances it follows
a recognized player record, and all three extracted position values agree
**exactly** with the player's. The other 31 follow the clock directly. The role of
this entity is unknown; matching position does not establish that it is a weapon,
attachment, or any particular component.

These 15/15/17 coordinate widths apply to the controlled map. The Ranked capture
uses 18/18/15; neither this copy-record length nor its entity identity is a
general map-independent rule.

This provides an independent cross-check on the coordinate windows in a new film.
All 1,562 player counters still match the clock. The vertical window remains
constant at 109863; x and y vary through the circles.

## Reproduce and inspect

The catalog category selects the controller follow-up:

```sh
HALO_TIMEOUT_SECS=120 HALO_PROBE_CATEGORY="Controller movement" \
  cargo run --example download_film_chunks

HALO_PROBE_FILM=controller/01-stick-circles \
  HALO_MOTION_OUTPUT=films/analysis/controller-followup \
  cargo run --offline --example film_motion_probe

python3 examples/plot_controller_inputs.py
python3 examples/build_theater_viewer.py
open films/analysis/theater_viewer.html
```

Generated locally:

- `films/analysis/controller-followup/positions.csv`
- `films/analysis/controller-followup/inputs.csv`
- `films/analysis/controller-followup/analog_summary.json`
- `films/analysis/controller-followup/controller_trace.html` — standalone
  interactive input and position traces, with synchronized time selection.

The 3D builder uses the combined controlled CSVs, not `controller-followup`, by
default. If those combined exports are missing, follow the
[replay rebuild guide](examples/theater_viewer/README.md#rebuild-controlled-film-csvs)
first. For an isolated position-only viewer, use
`python3 examples/build_theater_viewer.py --csv films/analysis/controller-followup/positions.csv`
(discovered multiplayer scenes are still included). The separate controller
trace works directly from this section's local export.

The extended Rust probe checks the 79-bit record's prefix, matching position when
available, and following boundary. It does not locate it by scanning for an input
suffix. Across all 27 films it walks 101,545 of 106,999 frames, exports 4,605 player
positions and 97,017 input pairs, and preserves the original keyboard results.

## Type 10: a new difference, not yet an interpretation

The controller clip has `80 00 00 00 00` in 5,388 type-10 packets and
`f8 00 00 00 00` in the remaining 24. The original corpus predominantly used
`82 00 00 00 00`. This is worth testing, but a single new clip cannot establish
that this is an input-device flag; other session differences are uncontrolled.
