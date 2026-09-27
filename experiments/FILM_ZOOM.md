# Scope stages: BR and S7 control

## Current native replay

The full Octagon film adds 100 scope samples: 86 first-stage and 14 unscoped.
No separate forced-descope record has been established. Victim hit notifications
are decoded independently and **never** create scope-zero samples. Custom-game
descope settings are unresolved; see [FILM_SETTINGS.md](FILM_SETTINGS.md).

Readouts now say **Scope sample 0/1/2**, with timestamp and age in the tooltip.
They hold the last recorded stage, which can become outdated when transitions
are unsupported. Death/new life and observed weapon-set updates clear the held
sample to unknown; they do not produce a stage-zero observation. The Octagon
example at 160.858 s therefore retains the 159.823 s scope sample after a hit.
The native replay preserves all existing scope rows exactly.

The sections below retain the original controlled-study counts and historical
Python reproduction commands; use THEATER_DECODER.md for the active workflow.

**2026-09-10 · film major version 41.** Match
`0131715c-568e-49d8-af6a-90e63c5c8156` establishes a two-bit scope-stage value:
**0 unscoped, 1 first zoom, 2 second zoom**. The record also contains the pawn's
wire identity and generation, independently matching existing life bindings in
both Ranked films. This is a zoom stage, not a magnification multiplier or FOV.

The capture is cataloged as `weapons/03-br-sniper-zoom` in **Weapon controls**.
Its download has seven chunks, 4,645,042 decompressed bytes, and 90.721 s of film.
The control was promoted from `pending` after v41 packet, clock, spawn, and
controlled motion-probe checks. Weapon names below come from the recorder;
there are no firing events here to bind BR/S7 names to opaque firing windows.

## Controlled observations

| Reported remaining time | Action | Film time | Decoded stage | Chunk / payload byte / bit |
| --- | --- | ---: | ---: | --- |
| 0:55 | BR scope in | 22.036707 s | 1 | 2 / 487955 / 0 |
| 0:50 | BR scope out | 26.674662 s | 0 | 2 / 500818 / 0 |
| 0:46–47 | Switch to S7 | 29.394270 s | Slot-1 selection, not a scope record | 2 / 508359 / 37 |
| 0:45 | S7 scope in | 32.080040 s | 1 | 2 / 515818 / 0 |
| 0:40 | S7 scope in again | 36.801706 s | 2 | 2 / 528911 / 0 |
| 0:35 | S7 scope out | 41.889813 s | 0 | 3 / 487420 / 0 |

Film timestamps include pregame time. The recorder's countdown times are
approximate. The weapon switch uses the previously established component 42
payload `0010011`. Scope changes are a separate record family; neither aim
changes nor the weapon-selection component are used to synthesize scope values.
There are no accepted firing, melee, grenade, or reload events in this film.
The existing motion probe exports zero position deltas, two aim samples, and
4,688 input pairs. The replay adds the independently checked spawn position;
it does not invent continuous movement or initial aim.

The five stage values are exactly **1 → 0 → 1 → 2 → 0**. There are zero accepted
scope records in the other 33 controlled films, including movement, aiming,
shooting/reload, grenade, melee, and weapon-switch controls. Recorder visual
confirmation of the new scope overlay remains pending; the input description
and byte checks are the evidence for this interpretation.

## Checked 25-bit window

Offsets are zero-based bits from the candidate start, MSB-first. The final stage
bit overlaps the following record's leading bit. These windows do not establish
the complete entity/event grammar or a registry component index for zoom.

| Relative bit | Width | Interpretation |
| ---: | ---: | --- |
| 0 | 1 | Uninterpreted leading bit |
| 1 | 10 | Signature `1001010110` |
| 11 | 8 | Pawn wire identity |
| 19 | 2 | Generation `01` / `10` |
| 21 | 2 | Checked guard `00` |
| 23 | 2 | Zoom stage `00`, `01`, or `10`; `11` remains unsupported |

Serial is `wire + 256 × (generation − 1)`. Unlike the firing/grenade windows,
this record has no separately decoded roster field. The existing spawn export
maps serial to roster player. Records must fall inside that pawn's life and
before its recorded death. Simultaneous conflicting stages are rejected.

For the first four transitions the following clock-shaped record begins at bit
24. The final scope-out includes the same checked 64-bit auxiliary form found
in the manual reload control, followed by a clock at bit 88. The auxiliary
bits 1:32 are `0100000000000100100001001100000`; its meaning stays opaque.
The clock guard checks bits 3:29 against one of the three known round signatures
and reads its eight-bit counter at 29:37. Leading `001`, `101`, or `111` is
accepted without interpreting those three bits globally.

Full original payloads:

```text
BR in:    cac008a07b4200a8806bef80
BR out:   cac008207b420158806bef80
S7 first: cac008a07b420378806bef80
S7 next:  cac009207b420450806bef80
S7 out:   cac008200242600000026ce07b4205d8806bef80
```

Ranked also establishes consecutive 25-bit scope windows at 24-bit strides.
The parser accepts up to two following windows before the checked clock or
auxiliary-plus-clock continuation. Each displayed observation still needs its
own life binding. Other continuations are withheld; there is no guessed skip.

## Corpus results and limits

The 36-film probe exports **1,414 scope observations**:

| Film | Accepted | Unscoped | First zoom | Second zoom |
| --- | ---: | ---: | ---: | ---: |
| BR/S7 control | 5 | 2 | 2 | 1 |
| Ranked Bandit | 538 | 206 | 332 | 0 |
| Ranked Oddball | 871 | 367 | 504 | 0 |
| Other 33 films | 0 | 0 | 0 | 0 |

Both Ranked exports contain observations for all eight players. Oddball checks
scope identities after generation reuse as well. These are partial observation
counts, not verified totals of button presses or complete state transitions.
Repeated values are retained; the format's repetition and all causes of scope
changes are not established. A scope-out value does not distinguish a manual
input, damage-induced descope, or another cause.

The evidence retains **529 life-bound raw candidates** with unchecked
continuations (155 Bandit, 374 Oddball). Some can be arbitrary payload matches;
they must not all be counted as missed scope changes. Unbound matches are counted
separately. No unsupported candidate is displayed or used to infer a transition.
There is no independent match-statistics API scope count.

The stage value does not establish the BR/S7 magnification, a second-stage
multiplier of 2×, scope overlay geometry, camera FOV, or animation timing.
Those remain separate decoding/calibration tasks. Weapon switching uses its own
partial parser and does not imply complete weapon inventory support in Ranked.

## Replay and inspector

The replay shows **UNSCOPED / SCOPE 1 / SCOPE 2**, or `?` when no observation is
available. Scope readouts appear above players, in the roster, and beside aim,
with previous/next observation buttons and gold timeline marks. The displayed
value is held from the last accepted observation; a dashed underline and tooltip
show that it is held. Partial Ranked coverage may leave a displayed observation
outdated. The selected-player readout always says **Last sample**.

Death, new life, and a subsequently observed weapon selection clear old scope
state. Scrubbing recomputes the value from timestamps, independently of playback
direction. Scope does not change the viewer camera's FOV or draw a guessed cone.
The inspector highlights the original stage bits and source fields, preserving
unparsed leading bits and opaque auxiliary content.

Open <http://127.0.0.1:8766/replay?clip=weapons/03-br-sniper-zoom> with Theater Lab
running, or select **Weapon controls → BR / S7 scope steps** in the offline toy.

## Reproduction and tests

Run from `experiments/`. The [replay guide](examples/theater_viewer/README.md)
explains aggregate CSV and multiplayer scene generation.

```sh
HALO_TIMEOUT_SECS=120 HALO_PROBE_FILM=weapons/03-br-sniper-zoom \
  cargo run --offline --example download_film_chunks
HALO_PROBE_FILM=weapons/03-br-sniper-zoom \
  HALO_MOTION_OUTPUT="$PWD/films/analysis/zoom-control" \
  cargo run --offline --example film_motion_probe
# Rebuild aggregate controlled CSVs per the replay guide, then:
python3 examples/film_firing_probe.py
python3 examples/film_weapon_probe.py
python3 examples/film_zoom_probe.py
python3 examples/build_theater_viewer.py
python3 -m unittest discover -s examples -p 'test_*.py'
node examples/check_theater_replay.cjs
python3 examples/theater_lab.py
# In another terminal:
node examples/check_theater_inspector.cjs
```

`analysis/zoom/{events,evidence,scenes}.json` preserves event rows, source offsets,
rejected-candidate evidence, audit counts, and the controlled scene.
`--zoom-json PATH` selects an alternate export for the HTML builder.
Rows are `[film_seconds, spawn_serial, zoom_stage]`. Match/player/life, time order,
and stage range are validated before merging. No record means unknown, including
before the first scope observation in each life.

`examples/fixtures/zoom_records.json` contains seven captured packets: all five
control transitions, a reused identity, and consecutive scope windows, plus an
unsupported-continuation negative. Tests cover exact bit values, stage/generation
limits, corruption/truncation, field coverage, and invalid life/order/identity.
Browser checks inspect all 1,414 displayed observations, five control steps,
zero/second-stage values, held state, selection/death/new-life resets, reverse
seeking, mobile layout, and all five inspector byte selections.
