# Hour-long raid

Match **9d644d09-38c3-429a-bf3d-de0ea577815f** is cataloged as
**Raid gameplay → Raid · 1 hour**. The downloaded v41 film ends naturally and
lasts **3,953.667 seconds (1:05:53.667)**. Its 200 chunks contain 189,285,701
decompressed bytes. All 20 recorded human players are represented in the replay.

The existing `X15Y15Z17` coordinate layout applies. The native export contains
942 recognized lives, 958,688 position samples, 814,749 aim samples, 19,426
firing samples, 173 melee events, 564 grenade-throw events, 1,524 reload events,
5,817 scope samples, 199,882 shield samples and 1,729 body-health samples.
Decoding took 9.06 seconds in release mode; JSON export took 0.76 seconds.

## Generalized spawn and identity forms

The initial decode correctly rejected multiple deaths attached to one recognized
spawn. The cause was missing spawn forms, not a reason to associate later
actions with an old life. Three observed encodings now decode in the shared
Theater module, with no raid-specific or match-ID branches:

- **Spawn record flags:** the first two bits are separate from the following
  16-bit `8704 + wire_id` field. Previous captures validated `00` and `10`;
  chained spawns also use `01`. Treating the second flag bit as part of the wire
  number incorrectly produced `65536 + wire_id`, including Nuzzles's initial
  spawn. The unvalidated `11` flag form remains unsupported.
- **Coordinate suffix:** both `000011101010010111001110000001` and
  `000011101010010111001110000011` follow the same checked coordinate window.
  Their differing flag remains opaque. The second form appears on 169 raid
  spawn candidates. Nearby tick-checked motion independently confirms the
  existing 15/15/17 partition; no new coordinate layout was introduced.
- **Generation tags:** the two-bit values `01`, `10`, `11`, `00` distinguish the
  four observed uses of a wire ID. `Life.generation` exposes logical generations
  1–4, with `Life.id = wire + 256 * (generation - 1)`. Spawn, pawn updates and
  combat events use the same mapping. In this film, 255 recognized lives use
  generation 1, 256 use generation 2, 256 use generation 3, and 175 use generation
  4. A later repeat of the identical wire/tag pair remains ambiguous and rejected.

Captured fixtures in `src/theater/fixtures/raid_records.json` at the repository
root check the new spawn forms, both later generation tags, nearby position
updates, firing events, malformed guards and an unsupported initial spawn.
The small fixtures keep the native test suite independent of the full raid.
All player streams, summary events, clocks and projectile tracks in the prior
29 films remain unchanged after this extension.

## Coverage and replay

Select **Raid gameplay → Raid · 1 hour**. All recorded timestamps remain on the
full timeline; seeking near 65 minutes exercises the fourth generation of lives.
The replay consumes the typed Film export and preserves the same sample-gap,
death, reverse-seek and unknown-value rules as the other recordings.

One initial spawn, roster slot 15 (Valen Quinn) at 9.334889 seconds, has an
unvalidated coordinate suffix and remains unparsed. The decoder recognizes
942 of the 943 captured pawn-spawn records; it does not invent the missing life.
Other unsupported fields remain unknown, including larger-roster crouch inputs,
physical crouch/slide, complete initial inventory and general grenade paths.

The compact typed JSON is about 309 MiB. The combined standalone replay is
about 110 MiB; samples remain intact. Initial browser loading is heavier than
for a single controlled clip.

From the repository root:

```sh
HALO_TIMEOUT_SECS=120 HALO_PROBE_FILM=raids/01-hour-long-raid \
  experiments/target/release/examples/download_film_chunks
experiments/target/release/examples/decode_theater_film \
  experiments/films/raids/01-hour-long-raid --compact
node experiments/examples/build_decoded_replay.cjs --corpus experiments/films
```

The catalog and chunks live under `experiments/`; the portable export is
`films/raids/01-hour-long-raid/decoded-film.json`. The compact evidence summary is
`films/analysis/raids/evidence.json`.
