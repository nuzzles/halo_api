# Coordinate layouts across maps

Position widths depend on the map's quantization bounds. They are not playlist
IDs, and different maps can share a layout. At the observed precision level 16,
the external Infinite decoder reference gives, per axis:

```text
extent = maximum - minimum
bits = min(26, ceil(log2(ceil(60 * extent))))
world = minimum + (raw + 0.5) * extent / 2^bits
```

| Captures | X/Y/Z widths | Coordinate bits |
| --- | --- | ---: |
| Forge controls, Octagon gameplay, raid | 15/15/17 | 47 |
| Bazaar | 17/17/16 | 50 |
| Both Ranked Arena games (Recharge) | 18/18/15 | 51 |
| Aquarius | 13/12/11 | 36 |

The native `CoordinateBounds` API computes widths and dequantizes a raw position
only when the supplied bounds agree with its checked `CoordinateLayout`. Bounds
are supplied separately; the library performs no network or filesystem access.
Only the four captured layouts are accepted by spawn parsing. The bounds formula
has not been established for every precision level or every map/record form.

## Aquarius: recovered spawn

Match `16d67b8c-09c2-4ab8-8b94-892f7edb6212` contains one idle spawn at film
3.036471 s, chunk 1, payload byte 505970, bit 3470. Its coordinate window starts
at spawn+263 and ends at +299. Its prefix is also used on Octagon, so the prefix
alone cannot select widths.

The exact match-history reference identifies Aquarius (`ctf_aquarius`), level
`fc878857-e778-4daf-b0ef-5d826b922f3f`, map asset
`c395f3ac-4614-45f9-a83a-56f69e8ae962`, version
`dda9eaa7-441d-45ef-983b-bf9e89298be5`. Independently published BSP bounds predict
**13/12/11**. That split fits the recorded suffix and yields raw
**[1980, 2469, 727]**. The native decoder now emits this spawn and 4,753 input
observations. Replay can open Aquarius. It does not invent movement or initial aim.

The retained fixture `aquarius_unresolved_spawn.json` keeps its historical filename;
its current test verifies the bounds-supported partition. There is still no
movement-based validation on Aquarius because the recorder stayed still.

## Bounds provenance and limits

[reference/map-coordinate-bounds.json](reference/map-coordinate-bounds.json)
contains only Aquarius and Bazaar, with a pinned external source and independently
retrieved level IDs. The reference comes from JGtm/LevelUp commit
`cf333a3889771c6462dfce9e1bc287a897043a47`, not from a decoded map-bounds field in
our films. `download_film_maps` caches exact match-history and map asset revisions
under each film's `settings/`. It found 19 of the 32 films in the first 1,000
history results; 13 older natural-end controls were outside that search.

Bazaar validates both its predicted widths and the world-speed conversion against
recorded movement (325 comparisons; median speed error 0.0493 world units/s).
See [FILM_VELOCITY.md](FILM_VELOCITY.md).

The newer Str8 Octagon controls and Facility Aetheria raid identify the Forge canvas
`fo11_blank`. Both Ranked games identify Recharge (`sgh_blueprint`). Their exact BSP
bounds remain unavailable here. An exploratory use of Vagabond bounds for the
controls gave plausible numbers but was **not established** and is not used by
the decoder. Matching widths never establishes matching origins or scales.
The bootstrap registry is largely identical across these maps; it cannot by
itself supply their distinct bounds. Map `.mvar` metadata identifies the base map
but does not provide a decoded BSP bound table in our current work.

## Earlier Bazaar inference

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

Historical validation before the bounds-supported Aquarius partition: 26 Theater tests, eight retained catalog/inspector
tests, and the 24-film browser replay check (including unresolved Aquarius import)
pass. The preceding parser refactor also passed clippy and wasm compilation. The original
23 films retain all 1,108,962 historical observations/lifetimes. Generalized
grammar accepts 1,251 additional input samples and three additional aim samples;
these are counted separately from historical evidence. No duplicate position or
aim samples were introduced.
