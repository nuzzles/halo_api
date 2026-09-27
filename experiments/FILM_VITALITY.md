# Body health, shields, and regeneration delay

Run reproduction commands from `experiments/`.

The existing captures identify pawn **component 4 as body vitality (health)** and
**component 5 as shield vitality**. Both contain changing amounts. The shield
payload also contains a countdown that resets on damage and expires before
regeneration. A new experiment is not required to locate these components.

This is a partial extraction of observed forms, not a complete health timeline or
a calibrated conversion to Halo damage points. Initial values, terminal/death
forms, maximum-vitality overrides, and every flag are still unresolved.

## Registry evidence

The bootstrap registry in all four checked films contains these names at
archetype index 35:

| Component | Registry name |
| ---: | --- |
| 0 | `object-position-dynamic-precision-component` |
| 1 | `object-translational-velocity-dynamic-precision-component` |
| 4 | `object-body-vitality-component` |
| 5 | `object-shield-vitality-component` |
| 21 | `unit-desired-aiming-vector-component` |
| 25 | `unit-command-tick-component` |

The already checked position, aim, and repeated counter align with this ordering.
Health and shield behavior provide additional independent support. This names
the observed pawn components; it does not decode the general `New` record's
archetype binding. Related registry names include component 11
`object-dead-state-component` and component 13
`object-maximum-vitalities-component`, whose payloads are not decoded here.

## Controlled AR kill

In `octagon/02-ar-kill`, all **150 accepted vitality deltas belong to Yet**.
None belong to the shooter, Nuzzles. The stationary idle comparison yields zero
matching updates; missing deltas do not mean zero health.

The shield amount drops in steps, remains unchanged between hits, and reaches
zero at **24.086002 s**. At that same timestamp, component 4 first appears. Its
amount then decreases while the shield amount stays zero:

| Film time (s) | Shield raw | Body raw, if updated |
| ---: | ---: | ---: |
| 22.834719 | 59 | — |
| 22.917991 | 53 | — |
| 23.001381 | 48 | — |
| 23.418664 | 43 | — |
| 23.585849 | 32 | — |
| 23.835640 | 16 | — |
| 24.002546 | 5 | — |
| 24.086002 | 0 | 124 |
| 24.169435 | 0 | 107 |
| 24.336098 | 0 | 91 |
| 24.419501 | 0 | 74 |
| 24.503090 | 0 | 58 |
| 24.669851 | 0 | 41 |
| 25.086859 | 0 | 24 |
| 25.253662 | 0 | 8 |

These times include film startup. The independently decoded death is at
25.344 s. The export does **not** synthesize a body-zero sample at that time.
The table is a selection of observations, not a count of fired bullets or hits.

The earlier [Octagon analysis](FILM_OCTAGON.md) recognized 115 continuations
through its clock-first parser. Scanning all kind-0 packets and checking the
same End/input boundary finds **142 component-5-only records** and **eight
component-4+5 records**. Many damage frames begin with event records rather than
the usual clock, explaining why clock-first extraction missed them.

## Checked payload windows

Offsets below are MSB-first, zero-based **within the component payload**. They
are not fixed offsets in a packet: the pawn's sparse component list and the
earlier payloads determine where each component starts.

Component 4 has this observed 11-bit form:

| Bit | Width | Observation |
| ---: | ---: | --- |
| 0 | 1 | Observed `1`; role/alternate forms unresolved |
| 1 | 7 | Body amount window, exported as `body_raw` |
| 8 | 3 | State window: `000` during the AR damage, `110` during recovery, `100` at its observed endpoint |

Component 5 has this observed 29-bit form:

| Bit | Width | Observation |
| ---: | ---: | --- |
| 0 | 8 | Shield amount window, exported as `shield_raw`; observed 0–64, high bit always zero |
| 8 | 8 | Observed zero window; semantic role unresolved |
| 16 | 9 | Regeneration-delay countdown, observed 0–300 ticks |
| 25 | 4 | State window: `0000` ordinary/delay, `0001` depleted, `1100` recovering |

The amount windows are measured integer projections. Do not assume the constant
bits prove the complete scalar encoding, or convert body values to percentages
without calibrating its endpoints and alternate forms. Shields repeatedly
recover to raw **64**; body recovery in the accepted Ranked forms reaches raw
**126** and changes state. Neither observation establishes absolute hit points,
custom-game maximums, or overshield behavior.

## Ranked recovery and clock checks

The independently checked Bandit prefixes provide **62,344 shield
observations**, including **560 body observations**, across all eight players.
Source byte/bit locations are reread and checked against the sparse list, known
position/aim guards, repeated clock counter, and spawn/death boundaries.

Oddball adds **115,978 shield observations** and **1,442 body observations**,
with body raw 6–126 and shield raw 0–64. The same registry names and field guards
pass across all three round clocks and both observed generations. There are
**93,393** additional countdown/clock checks, **13,802** shield recovery-state
observations, and no rejected windows, countdown mismatches, or state failures.
Across the four films the export contains **178,472 shield observations** and
**2,010 body observations**. Body updates share rows with shield updates.

For ELITExBOSH's life with **spawn serial 8, roster index 7**:

- At 51.717165 s, shield raw is 8 and the delay is 299 ticks.
- At 56.705489 s, the delay reaches zero while shield raw remains 8.
- At 56.722078 s, shield raw increases to 9 and state becomes `1100`.
- At 58.456698 s, it reaches 64; at 58.473418 s, state returns to `0000`.
- Later in that life, at 66.148208 and 66.165274 s, body raw increases from 125
  to 126 while shield raw increases from 1 to 2. Body state changes `110` → `100`.

The AR hit frames independently show the delay reset to **300**. In **50,560
Bandit** and **87 Octagon** short, non-reset intervals, its decrease exactly
equals the frame-counter advance, including wrap and skipped observations.
There are no mismatches. The established approximately 60 Hz cadence makes this
a roughly **five-second regeneration delay**.

All accepted `0001` shield states coincide with raw zero, and all `1100` states
have delay zero. These are observed state patterns, not a complete bit-flag
dictionary. Bandit has 7,775 shield recovery-state observations. Recovery can be
interrupted by new damage; the export does not fill gaps or regenerate values
using a guessed formula.

## Reproduce and limits

After building the Octagon and Ranked scenes with the
[replay guide](examples/theater_viewer/README.md):

```sh
python3 examples/film_vitality_probe.py
python3 -m unittest discover -s examples -p 'test_film_vitality.py'
```

The probe writes `films/analysis/vitality/samples.csv` and `evidence.json`.
The CSV preserves raw payload bits, component offsets, source chunk/byte,
spawn serial, player name, time, clock, and the independent boundary check.
Blank body columns mean that component was absent, not that body health was zero.
The evidence includes registry names, per-player counts, state checks, and the
countdown/clock audit. All reproduction is offline.

Octagon accepts only complete `[5]` and `[4,5]` deltas followed by the known
End/input guard. Ranked deliberately reuses the existing prefixes through
component 25. As a result, **most immediate Ranked damage frames are absent**:
they often start with an event instead of the usual clock. The Ranked body
observations in this export describe recovery, not every health loss. Neither
the decoder nor the replay should claim a complete health timeline from this
partial export.

The replay now offers a **provisional visualization** of these observations:
compact blue shield and coral body-health bars in every persistent roster card,
above players, and in the selected-player card, plus arrows to jump between amount
changes. Tooltips retain sample ages and recorded recovery/delay details. Shield
64 and body 126 are display scales, not calibrated hit points. Unknown values
show `?`, old observations use dim dashed bars, and death clears values to `—`.
The meters stay visible; respawn returns to unknown until a new sample arrives. Health and shield timestamps remain independent. See the
[preview guide](examples/theater_viewer/README.md#health-and-shield-preview).

Fixtures cover an AR shield hit, shield depletion with body damage, the last
accepted body hit, Ranked recovery frames, Oddball round/generation changes, counter disagreement, truncated
records, corrupt component/continuation guards, and unsupported scalar forms.

A useful next controlled experiment would keep both players still, damage only
shields and allow full recovery, then deplete shields and damage body health
without killing the target, followed by full recovery and a final lethal burst.
That would clarify scalar endpoints, body/shield flags, and terminal forms.
