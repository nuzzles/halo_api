# Pawn velocity

The version-41 pawn component **1**, named
`object-translational-velocity-dynamic-precision-component` in the registry,
contains a direction and a nonlinear magnitude. Direction is now decoded in
`halo_api::theater` and displayed in motion replay. The magnitude-to-world-speed
conversion remains unresolved.

## Wire forms

Bits are MSB-first, relative to the start of component 1:

| Bits | Meaning |
| --- | --- |
| 0–1: `01` | Short stationary form; no direction or magnitude follows |
| 0–1: `00` | Long form, 31 bits total |
| 2–20 | 19-bit unit-direction code |
| 21–30 | 10-bit nonlinear magnitude code |

The short form occurs at stops and in idle controls. Absence of component 1 is
not a stationary observation. A **long-form magnitude code of zero** occurs near
the jump apex and retains its recorded direction; it is not replaced by the
short form. Forms `10` and `11` remain unsupported.

## Direction quantization

For direction code `q`:

```text
face = q / 87381
remainder = q % 87381
u_code = remainder / 294
v_code = remainder % 294
u = 2 * (u_code + 0.5) / 293 - 1
v = 2 * (v_code + 0.5) / 293 - 1
```

Division for `face` and `u_code` is integer division. Valid faces are 0–5; valid
component codes are 0–292. The larger strides leave unused codes. These are
withheld, not clamped into plausible directions. Form a vector using this table,
then normalize it:

| Face | Vector | Cardinal code | Independent control |
| --- | --- | ---: | --- |
| 0 | `(1, u, v)` | 43070 | Left: +X |
| 1 | `(u, 1, v)` | 130451 | Backward: +Y |
| 2 | `(u, v, 1)` | 217832 | Jump rising: +Z |
| 3 | `(-1, u, v)` | 305213 | Right: −X |
| 4 | `(u, -1, v)` | 392594 | Forward: −Y |
| 5 | `(u, v, -1)` | 479975 | Jump falling: −Z |

Both grid coordinates are 146 for each cardinal. The midpoint formula therefore
recovers exact zero on the other two axes. Joystick circles exercise the grid
between these cardinals; this is not a set of boolean movement directions.

This independently inferred partition matches the public Halo 4 implementation
in Blam-Network/blf, pinned to commit
`ea02ce095f17e9fe10b00695b4641f0209423ef0`:

- [19-bit constants `0x15555`, `0x126`](https://github.com/Blam-Network/blf/blob/ea02ce095f17e9fe10b00695b4641f0209423ef0/blf_lib/src/blam/common/math/unit_vector_quanitzation.rs)
- [Six-face mapping, exact midpoint and normalization](https://github.com/Blam-Network/blf/blob/ea02ce095f17e9fe10b00695b4641f0209423ef0/blf_lib/src/blam/halo4/v20810_12_09_22_1647_main/math/real_math.rs)

This reference corroborates direction; it does not establish Infinite's speed
constants or position-coordinate scales.

## Independent movement checks

The initial investigation checked 288,071 long forms and 766 short forms attached
to accepted positions across ten recordings. Every long direction fit the valid
face/grid bounds. Horizontal direction was compared with displacement between
two nearby position samples on either side, within the same pawn life and a
150 ms window. Displacements below eight raw horizontal units were excluded to
reduce quantization noise. This is a comparison with recorded motion, not exact
instantaneous ground truth: acceleration, changing direction and packet batching
introduce differences.

| Recording | Comparisons | Median angular difference | 90th percentile |
| --- | ---: | ---: | ---: |
| Controller circles | 1,252 | 3.15° | 6.86° |
| Bazaar | 302 | 3.84° | 6.77° |
| Octagon first to 50 | 12,830 | 1.77° | 5.11° |
| Ranked Oddball | 214,642 | 1.55° | 5.65° |

All eligible horizontal directions in straight keyboard controls were exact;
all 62 eligible jump-axis sign comparisons agreed. Vertical angles cannot be
calibrated against raw position differences without knowing the per-axis world
scales. The same direction decoder applies to all three supported position
layouts; it does not use match IDs, map names or playlist categories.

## Magnitude limits

The initial sample set spans magnitude codes 0–484. They rise and fall with
movement speed independently of direction, but are **not linear speed**. Jump
acceleration and controller changes suggest a logarithmic-like companding curve.
An empirical fit alone does not establish the engine's quantizer or its constants.
Frame timestamps may bunch together, so naive `distance / timestamp delta` is
also unreliable. The wrapping command clock advances at nominal 60 Hz in these
controls. Raw position axes have uncalibrated, potentially different scales.

Consequently the API exports `magnitude_code`, not guessed metres per second,
and replay never integrates it to fill gaps. A useful next control would isolate
steady movement and jumps with known map/world distances; the unresolved code
range, quantization endpoints and any dynamic-precision alternatives still need
evidence.

## API, replay and inspector

`PlayerTrack.velocities` contains `Sample<Velocity>`, bound to recorded pawn lives.
`Velocity::Stationary` preserves the short form. `Velocity::Directed` preserves
both integer codes and a decoded `[f32; 3]` unit direction. The float precision
exceeds the wire grid resolution; the original integer code is retained exactly.
Older schema-1 exports deserialize with an empty velocity stream.

Clocked pawn prefixes, input chains and supported combined weapon records all
propagate velocity. Boundary/tick/life guards remain required. Invalid direction
codes retain the checked field length while withholding velocity, allowing
otherwise supported position/aim fields to survive. Truncated and unknown forms
are rejected. Each exported source span is exactly 2 or 31 bits.

The replay's **Velocity** toggle shows a blue, fixed-length direction arrow for
recent samples, separate from the gold aim ray. The selected-player readout shows
unit X/Y/Z, raw magnitude and sample time, with previous/next sample buttons.
After 100 ms without a sample it labels the value as a last observation and hides
the arrow. Missing positions, death and new lives also prevent a current arrow.
No position samples or existing position interpolation rules are changed.

`decode_theater_film` also writes `decoded-film.velocity.csv` beside its JSON
(or `<output-stem>.velocity.csv` with a custom output path). This is a seekable
inspector evidence index, not a replacement replay file. The inspector reads one
chunk's accepted source locations and rechecks their form/codes against the bytes.
It marks the direction's 19 bits and raw magnitude's 10 bits decoded, the long
form's two bits checked structure, and the short stationary form's two bits
decoded. It retains the explicit speed-scale caveat. Unsupported directions stay
opaque. Legacy controlled/ranked annotations use the same field display.

Coverage still measures the union of verified inspector annotations across **all
decompressed bytes**, including registry padding, snapshots and unknown records.
A field's whole-recording contribution can be small even when it occurs in many
movement frames. Native source windows are not blanket semantic coverage.

The 32-film refresh exports **1,318,149 observations**: 1,299,173 directed and
18,976 stationary. It preserves every pre-existing player stream, life, event,
projectile and top-level data field (diagnostics excluded from the comparison).
The raid contributes 880,097 velocity samples. The release decoder took about
16.2 seconds for the entire corpus, including 9.0 seconds for the raid, excluding
file loading, JSON export and the independent preservation audit.

With native and legacy velocity annotations, Oddball's whole-recording decoded
coverage rises from **12.55% to 14.34%**. The 63-chunk scan has no annotation errors
and accounts for all 438,720,768 bits:

| Category | Bits |
| --- | ---: |
| Decoded | 62,893,590 |
| Checked structure | 7,513,110 |
| Opaque | 2,454,411 |
| Unparsed | 365,859,657 |

Captured-byte tests live in `src/theater/fixtures/velocity_records.json`, covering
cardinals, diagonals, all supported coordinate layouts, both forms and long-form
zero. Tests also check all byte alignments, invalid/truncated codes, life binding,
source ranges, duplicate removal and backward-compatible JSON. Local corpus
validation and direction comparison totals are retained under ignored
`films/analysis/velocity/`.
