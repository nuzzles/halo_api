# Theater v41: position fields, record boundaries, and movement inputs

Run reproduction commands from `experiments/`.

**Updated 2026-09-10.** This document preserves measurements from the original
26-film keyboard experiment. The current catalog and supported-capture status are in
[FILM_FORMAT.md](FILM_FORMAT.md). [FILM_BANDIT.md](FILM_BANDIT.md) extends position
and aim extraction to all eight Ranked players, with 18/18/15 coordinate widths,
sparse pawn components, and identities that survive respawning. The recorder
visually confirmed that combined replay; raw-unit calibration remains open.

**Aiming follow-up:** [FILM_AIM.md](FILM_AIM.md) adds a checked 68-bit aim
record, cyclic yaw, upward pitch, and independent recoil evidence from both
shooting clips. The probe now also exports `aim.csv`.

**Controller follow-up:** [FILM_CONTROLLER.md](FILM_CONTROLLER.md) confirms all
63 observed levels (0–62) on each movement axis and adds a 79-bit record whose
position fields match the player's. Counts below describe the original 26-film
keyboard experiment.

The original 26-film measurements were reproduced on 2026-09-09. To inspect one
of those captures in the current mixed corpus:

```sh
HALO_PROBE_FILM=force-end/04-walk-forward \
  HALO_MOTION_OUTPUT=films/analysis/motion-example \
  cargo run --offline --example film_motion_probe
```

The probe writes `positions.csv`, `aim.csv`, and `inputs.csv` to its output
directory (default `films/analysis`). `HALO_FILM_CORPUS` changes the input root;
`HALO_MOTION_OUTPUT` changes the output directory. It makes no network requests.
Use the [controlled-corpus rebuild](examples/theater_viewer/README.md#rebuild-controlled-film-csvs)
for all 31 supported controlled films. To repeat only this document's 26-film
counts, use only `theater_probe_films` in that recipe and a separate output
directory. An unfiltered run over the full corpus encounters the unsupported
Ranked layout; the Ranked Python builder is a separate decoding path.

## Findings from the keyboard experiment

1. **Three raw position fields are recoverable in three player-record layouts.**
   There are 3,043 samples across the eight walk clips and two jump clips.
2. **The supposed padding contains movement inputs.** Two 6-bit fields distinguish
   neutral, forward, backward, left, and right. The probe extracts 92,553 input
   pairs, including idle frames.
3. **Specific record lengths now have structural evidence.** The recognized player
   layouts occupy 97, 134, or 105 bits. Entity `(324, 2)` has a 51-bit layout.
   These offsets are checked by walking to subsequent records and the input block;
   they are not derived from the packet size.

This is a partial decoder for observed signatures, not a general component decoder.
Position scale, world origin, and the complete field encodings are still unknown.

## Position evidence

Names `x`, `y`, and `z` below identify the extracted axes; world orientation has not
been calibrated. The stationary values are `(16361, 16410, 109862)`.

| Action | Force-end samples | Natural-end samples | Observed behavior |
| --- | ---: | ---: | --- |
| Forward | 355 | 453 | y decreases; x and z stay fixed |
| Backward | 302 | 431 | y increases; x and z stay fixed |
| Left | 299 | 298 | x increases; y and z stay fixed |
| Right | 299 | 470 | x decreases; y and z stay fixed |
| Jump | 69 | 67 | z rises and falls; x and y stay fixed |

Both jumps ascend monotonically from the first airborne sample to a peak of
**109962**, then descend monotonically to **109862**: a 100-unit excursion.
The extracted portions last 1.168 s and 1.155 s. The horizontal values agree exactly
between the two jumps and with the stationary axes in the walk clips.

The forward/backward and left/right pairs identify independent fields and their
directions. Jumping independently identifies the vertical field and its return to
ground. Values continue coherently as the player changes record layout. Records
also contain an 8-bit value that agrees with the independently extracted clock
counter in **all 3,043 samples**, including wraparound.

The offsets and input values were developed using force-end recordings; the same
rules were then applied to natural-end recordings without changing the signatures.
The Rust probe applies the same checks to both groups and fails on counter,
boundary, or action inconsistencies rather than dropping disagreeing samples.

### Exact extraction windows

All offsets are zero-based, MSB-first, relative to the start of the player record.
This probe enters it at frame bit 37, after the recognized clock record.

The player header is the 14-bit pattern `10001000000000`, interpreted by the
existing header model as `(slot 128, tag 0)`.

The Ranked follow-up identifies this as spawn serial 0. Later spawns have
different headers; `(128,0)` is not a permanent identifier for a player.

| Field | Layout A | Layout B | Layout C |
| --- | --- | --- | --- |
| Signature after the header | `010001000000001100100000` | `010001100000000000101100100000` | same as B |
| x extraction | bit 38, 15 bits | bit 44, 15 bits | bit 44, 15 bits |
| y extraction | bit 53, 15 bits | bit 59, 15 bits | bit 59, 15 bits |
| z extraction | bit 68, 17 bits | bit 74, 17 bits | bit 74, 17 bits |
| Additional selector | — | bits 91–94 = `0000` | bits 91–94 = `0001` |
| Repeated clock counter | bit 88, 8 bits | bit 125, 8 bits | bit 96, 8 bits |
| Following bit | bit 96 = `0` | bit 133 = `0` | bit 104 = `0` |
| Record length | **97 bits** | **134 bits** | **105 bits** |

These are observed integer **windows**, not a proof that every coordinate's complete
encoding has exactly that width. Additional constant upper bits, range parameters,
or a different underlying numeric representation have not been ruled out. In
particular, do not convert these values to meters or reuse the offsets on a record
whose signature and coordinate layout do not match.

The later sparse-list finding explains these lengths: A has components `[0,25]`,
and B/C have `[0,1,25]`. With this map's widths, position component 0 occupies
54 bits. Component 1 occupies 31 bits in B and 2 bits in C, accounting for their
29-bit difference; its semantic role is still unknown. The counter is component
25. The Ranked position component is 58 bits, so these absolute offsets and
record lengths cannot be reused there.

Example, force-end forward walking, consecutive frames:

```text
frame payload: a0 7b 42 07 ec 40 08 80 64 0f fa 5f fb eb 49 8f e8 40 37 e7 c0
raw position:  (16361, 16375, 109862), counter 253, input (62,31)

frame payload: a0 7b 42 07 f4 40 08 80 64 0f fa 5f fa 6b 49 8f f0 40 37 e7 c0
raw position:  (16361, 16372, 109862), counter 254, input (62,31)
```

## The input block disproves the padding interpretation

After the recognized entity chain's `000` terminator, the common input block starts
with the 13-bit pattern `1000000001101`, followed by two 6-bit fields:

| Action | First field (forward/backward) | Second field (left/right) |
| --- | ---: | ---: |
| Neutral | 31 | 31 |
| Forward | 62 | 31 |
| Backward | 0 | 31 |
| Left | 31 | 62 |
| Right | 31 | 0 |

The table holds in both recording groups. During deceleration or stationary
updates, a movement clip can carry the neutral input; the probe permits that and
checks that no conflicting direction appears. Interior analog values and the
meanings of the 13 prefix bits have not been decoded in this keyboard experiment.
The controller follow-up confirms intermediate axis values; their mapping to
physical stick magnitude remains uncalibrated.

For a 9-byte idle frame the block starts at bit 40. Its first 25 bits are:

```text
1000000001101 011111 011111
prefix        31     31
```

This is the previously discarded `80 6b ef 80` region. In an ordinary 21-byte
walking frame, the block starts at **bit 137**, not on a byte boundary. For forward
movement it reads `1000000001101 111110 011111`, or `(62,31)`.

The suffix shifts with the preceding records, and its fields change with the
scripted direction. It is therefore meaningful serialized data, not uncleared
buffer padding. Earlier claims that frame content ends at the entity-chain `End`
were incorrect: **the entity chain is only one section of the frame**.

The probe recognizes 92,553 such blocks. Another 25 supported frames have a
different nonzero input prefix and are explicitly reported as unsupported. A zero
at the block start is observed in the early 6-byte frames. Bits after the two axes
remain undecoded; the jump's first airborne frame has additional nonzero bits
there. No rule about the amount of final alignment padding is assumed.

## Record lengths verified against continuations

For layout A, the chain walk is:

```text
clock [0,37)
player [37,134)          # 97 bits
End [134,137)
input block starts 137
```

This predicts the input location in differently sized frames without examining
their size. Four player frames have an additional record instead of `End` at the
predicted location:

| Film | Player's next boundary | Continuation | Input starts |
| --- | ---: | --- | ---: |
| natural-end/backward, ~76.458 s | 134 | `(324,2)` | 188 |
| natural-end/backward, ~76.709 s | 142 | `(324,2)` | 196 |
| natural-end/right, ~77.173 s | 134 | `(324,2)` | 188 |
| natural-end/right, ~77.407 s | 134 | `(324,2)` | 188 |

The auxiliary record has header `10010100010010`, then the constant 19-bit prefix
`0100001000001000001`, then 18 bits of unknown data: **51 bits total**.
After it, `000` is followed by the same neutral input block. Independently, all
**1,001** clock-first frames with `(324,2)` in record position 2 also walk correctly
with this 51-bit length, followed by `End` and the input section. Total checked
auxiliary records: **1,005**.

The old reported value 51 was unsupported because it came from frame-size
arithmetic. Its numerical agreement with the new measurement does not rehabilitate
the old method. The new support comes from matching records and input blocks at
predicted offsets, including nonterminal player records.

## Coverage and limits

The probe walks **96,545 of 101,587 frames** using the limited grammar:

```text
recognized clock + [recognized player] + [recognized auxiliary] + End
```

There are 98,297 frames matching the clock prefix. Other first records and player
layouts are skipped, not guessed. The CSVs are samples, not complete trajectories;
missing samples must not automatically be interpreted as no movement. The input
CSV likewise does not contain every recorded input update.

Timestamp offsets use the first nonzero packet timestamp, as in the existing
corpus probes. No fabricated chunk intervals are used.

This original probe does not recover slot-to-archetype bindings, calibrated
absolute world coordinates, or the remaining input/button fields. The later
Ranked builder decodes a sparse pawn component list, while a general component
grammar and complete chain walk remain open. Missing positions must not be filled
by integrating the input fields as though they were measured velocity.

## Independent packet audit

All 102 replication chunks still have complete packet coverage. Two older claims
are contradicted by the bytes:

- Header bytes 2–3 are nonzero in **2,435 of 203,840 packets**. Their meaning is
  unknown; they could include padding, but they must not be assumed zero.
- Type-10 payloads are `8200000000` ×101,175; `f800000000` ×331;
  `8400000000` ×31; `8000000000` ×15; `8600000000` ×15;
  `8800000000` ×13; and `8a00000000` ×7. Their meaning remains open.

The new probe prints both inventories on every run.
