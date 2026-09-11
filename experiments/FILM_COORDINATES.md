# Coordinate layouts across maps

The new Aquarius capture supports map-dependent position encoding, but does not
establish that every map has a unique layout or recover the selection rule.

| Captures | X/Y/Z widths | Total coordinate-window bits |
| --- | --- | ---: |
| Original Forge and Octagon controls | 15/15/17 | 47 |
| Full Octagon first-to-50 gameplay | 15/15/17, different spawn prefix | 47 |
| Bazaar | 17/17/16, empirically inferred | 50 |
| Two Arena captures | 18/18/15 | 51 |
| Aquarius, idle | Unresolved | 36, candidate window |

The registry calls pawn component 0 `object-position-dynamic-precision-component`.
Map bounds and precision settings are plausible causes of the differing widths.
Neither their metadata nor a formula relating them to bit widths is decoded.
Repeated captures of each map with movement and differing spawn locations would
help distinguish a stable map layout from other encoding choices.

## Aquarius: a distinct but unresolved encoding

Match `16d67b8c-09c2-4ab8-8b94-892f7edb6212` is a v41 film with a recorded end,
seven chunks, and 5,366 replication frames. The recorder confirmed staying still
throughout. It has one recognizable pawn-spawn candidate in chunk 1, payload
byte 505970, at payload bit 3470 (film second 3.036471).

The known spawn body and suffix are present. Relative to the spawn start:

- The 63-bit pre-coordinate prefix at 192 differs from Bazaar at bit **210**.
- Bits 255–262 are `01100000`, versus Bazaar's `11100000`.
- The known 30-bit suffix starts at **299**, versus 310/313/314 in the other captures.
- Keeping the previously established coordinate start at 263 gives a **36-bit
  candidate window**, `001111011110010011010010101011010111`.

The changed preceding flags make this a new structure to validate. Without a
position delta, the same start and overhead cannot be independently confirmed.
The subsequent [full Octagon gameplay capture](FILM_OCTAGON.md) shares this prefix
but has a 47-bit window and motion supporting 15/15/17. Its prefix is now accepted
by the decoder; Aquarius's unresolved 36-bit window still fails the supported
coordinate boundaries. This is another example of a prefix not identifying widths.
An all-offset scan found no component-0 update candidates for the initial pawn,
consistent with the recorder's idle description. Its candidate position window
appears only once, so there is no independent copy or motion continuity check.

Do **not** infer 12/12/12 merely because the total is 36. That split yields
`[990,1234,2775]`, while 13/13/10 yields `[1980,4938,727]`, and 12/13/11 yields
`[990,2469,727]`. A single uncalibrated position cannot distinguish them.

No new `CoordinateLayout` variant was added. The partial Film JSON retains its
registry, roster, clocks, and unsupported-spawn diagnostics; player position
remains unavailable. The replay builder reports and skips this film, and direct
import explains that no decoded positions are available instead of opening an
empty scene. The downloaded chunks remain available in the byte inspector.

Captured evidence is in `films/analysis/aquarius/coordinate-evidence.json` and
the tracked `../src/theater/fixtures/aquarius_unresolved_spawn.json`. A regression
test preserves the unknown result, guarding against accidental assignment to an
existing tuple. The next useful control is Aquarius with several seconds of
forward movement, sideways movement, and a jump, keeping the camera fixed.

## Bazaar: the third supported partition

Match `79bce30e-0d61-4c57-93dd-92d61124e80c`, reported as a one-minute idle
game on Bazaar, introduces a third empirically supported X/Y/Z encoding:
**17/17/16 bits**. This is a v41 film with a recorded end. Its manifest reports
48.945 seconds. The downloaded bytes also contain late movement/aim updates and
combat events; the recorder confirmed acting near the end. That clarification
is retained in `films.csv` alongside the original idle-control intent.

### Evidence and confidence

The spawn is in chunk 1, payload byte 539664, at payload bit 4138. Offsets in
this table are relative to that spawn start:

| Field | Offset | Width |
| --- | ---: | ---: |
| Previously observed coordinate prefix | 192 | 63 |
| X raw | 263 | 17 |
| Y raw | 280 | 17 |
| Z raw | 297 | 16 |
| Previously observed suffix | 313 | 30 |

The prefix exactly matches the original Forge controls. Their suffix starts at
310, while the two Arena captures use 314. Bazaar therefore has **50 coordinate
bits**, compared with 47 and 51. The prefix itself does not encode a unique
coordinate layout or gameplay category.

The 17/17/16 partition gives spawn `[113463,33570,10206]` and first ordinary
position `[113462,33570,10206]`. Across 394 initial-pawn position candidates,
all three axes change smoothly, with maximum consecutive changes `[16,10,13]`.
Each axis has unit-resolution changes (the gcd of its changes is one).

Enumerating all positive axis widths up to 32 summing to 50 leaves only
17/17/16 when requiring unit-resolution changes and no consecutive jump of
128 raw units or more. Alternatives such as 16/17/17 create large wraparound
jumps; 17/18/15 makes every Y change even; 18/18/14 makes X changes multiples
of two and Y changes multiples of four. These are empirical partition checks,
not a decoded engine formula. The 50-bit total also aligns subsequent velocity,
aim, and command-tick fields: 391 of those candidates match the frame tick;
one of those has an unsupported continuation and remains rejected by the
clocked-prefix reader.

The total length has independent structural checks. The per-axis split relies
on the observed motion/quantization behavior. We have **not** recovered map
bounds, the precision-selection metadata, physical units, or absolute origin.
Another map could use a different partition with the same total length; matching
a suffix alone cannot establish universal support for every such map.

## Decoder changes

- `CoordinateLayout::{X15Y15Z17,X17Y17Z16,X18Y18Z15}` names widths explicitly.
  `axis_bits()` supplies widths to all pawn position readers and skip lengths.
- Spawn-prefix validation and supported coordinate lengths are separate checks.
- Clocked pawn prefixes, complete input chains, vitality, and weapon updates
  are selected by their record structure, without a `ranked` boolean.
- Input chains and clocked prefixes share the same component reader. Position
  and aim may appear together, including on Bazaar. Supported End/input boundaries
  are recognized alongside the existing continuation form.
- A complete input chain owns its observations; the prefix scan does not emit
  a second position with the attached input missing.
- Old JSON layout strings `Controlled` and `Ranked` still deserialize; new
  exports use the explicit names. No match ID or map name selects a parser.

The motion-chain reader still requires the checked wire-0/generation-1, roster-0
chain. The later [terminal input decoder](FILM_POSTURE.md) separately binds
roster slots 0/1 across respawns in one/two-player films, independently of the
coordinate layout. Other command forms and larger rosters remain unresolved.
General projectile paths remain unsupported. This refactor does not imply
complete record grammar or map support.

## Reproduce and view

From `experiments/`:

```sh
HALO_PROBE_FILM=maps/01-bazaar-idle cargo run --example download_film_chunks
cargo run --release --example decode_theater_film -- films/maps/01-bazaar-idle --compact
node examples/build_decoded_replay.cjs --corpus films
```

Choose **Map controls → Bazaar idle** in the replay. The current decoder exports
one life, 405 position samples (including spawn), 295 aim samples, 2,087 input
samples, seven firing events, one grenade-throw event, and one reload event.
There are no accepted health/shield samples or projectile paths in this film.
These remain unknown. Position changes occur around film seconds 35.8–42.8;
the spawn remains the recorded position during the preceding idle interval.

`films/analysis/bazaar/coordinate-evidence.json` retains candidate bit windows,
source offsets, and the partition criteria. Small captured records are tracked
in `../src/theater/fixtures/bazaar_records.json`, covering spawn, both velocity
forms, combined position/aim, and continuation versus End/input boundaries.
Rust tests check the extracted fields, wrong-width/tick rejection, suffix
corruption, JSON compatibility, and input grammar with multiple width tuples.

Validation after adding Aquarius: 26 Theater tests, eight retained catalog/inspector
tests, and the 24-film browser replay check (including unresolved Aquarius import)
pass. The preceding parser refactor also passed clippy and wasm compilation. The original
23 films retain all 1,108,962 historical observations/lifetimes. Generalized
grammar accepts 1,251 additional input samples and three additional aim samples;
these are counted separately from historical evidence. No duplicate position or
aim samples were introduced.
