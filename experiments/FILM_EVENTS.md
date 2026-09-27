# Recorded match events and medals

Updated 2026-09-13; film major version 41.

The upstream `halo_api::theater` decoder reads the recorded human-player
highlight timeline directly from footer chunks: kills, deaths, generic mode
events, and medal awards. All **3,667 declared events in 32 retained films**
decode, including **722 medal awards**. The five nonempty films also match the
independent stats API for every human player's kills, deaths, and counts of each
medal `NameId`.

This covers the stored highlights. It does not establish a complete gameplay
event log, assists, specific objective actions, or killer/victim pairings.
Motion/combat observations still come from their separate replication decoder.

## Sources and medal identities

[Den's article](https://den.dev/blog/extracting-stats-film-files-halo-infinite/)
describes chunk discovery, unaligned event tails, and player identities, and
credits Andy Curtis's research. The complete film-code/name reference used here
is SPNKr's [`medal_codes.json`](https://github.com/acurtis166/SPNKr/blob/71bd2d449fbc28dd0a3053cdb033d8b16733c518/spnkr/film/medal_codes.json),
pinned to revision `71bd2d449fbc28dd0a3053cdb033d8b16733c518`.

The September 13 official CMS snapshot at
`https://gamecms-hacs.svc.halowaypoint.com/hi/Waypoint/file/medals/metadata.json`
contains 151 medals. Its SHA-256 is
`c8d9f44bb2cbf97d8d028741fac1876f557d1fe2804931901ecd2ae57259cf30`.
Every CMS English name joins exactly to the published film-code table. Film IDs
are **u8 codes**, while the stats API uses a separate **u32 `NameId`** namespace.
They are not interchangeable.

`FILM_MEDAL_DEFINITIONS` contains all 155 published codes. Four names are absent
from that CMS snapshot and retain `None` for `name_id` and `sorting_weight`:
150 Big Deal, 157 Clash of Kings, 158 Contract Killer, and 160 Watch the Throne.
No film in the corpus awards those four medals. External mapping coverage is
broader than the medals observed in these recordings.

The pinned [LevelUp research directory](https://github.com/JGtm/LevelUp/tree/cf333a3889771c6462dfce9e1bc287a897043a47/.ai)
also provides `TABLE_MEDAILLES_FILM.tsv`; all its covered `NameId` mappings agree.
The compact reference fixture is
[`medal_catalog.json`](../src/theater/fixtures/medal_catalog.json).

The older table missed 52 awards already present in our corpus: 23 Reversal,
18 Guardian Angel, nine 360, one Hail Mary, and one Sneak King. Code 166's label
is corrected from “Mounted” to **Mounted & Loaded**. Unknown codes are preserved
as awards with a raw film ID and absent name/`NameId`, rather than discarded.

## Checked binary layout

Manifest **chunk type 3** contains framed **packet type 9** and a zero-length
type-7 packet. Chunk types and packet types are different namespaces. The normal
16-byte little-endian packet header bounds the payload. The first four payload
bytes of type 9 are a **big-endian u32 declared event count**.

All 3,667 captured records share this empirically checked layout:

1. A little-endian u64 XUID immediately precedes an unaligned `2d c0` or `25 c0`
   marker. The decoder does not constrain identities to a particular numeric
   range or require the bootstrap roster.
2. The event tail begins **14,926 bits after the start of the XUID**. This is a
   version-41 captured offset; the intervening state is still opaque.
3. The tail is 60 bytes long, followed by the MSB-first `0x00002ee0` end marker.

Tail offsets are relative to its unaligned start:

| Bytes | Meaning |
| --- | --- |
| 0–31 | UTF-16LE gamertag, up to 16 code units, zero-padded |
| 32–46 | Opaque; not required to be zero |
| 47 | Event type, or sorting weight for a medal |
| 48–51 | Big-endian u32 timestamp in milliseconds |
| 52–54 | Checked zero reserved field |
| 55 | Medal flag, accepted values 0 and 1 |
| 56–58 | Checked zero reserved field |
| 59 | Metadata; film medal code when the medal flag is 1 |

All observed nonmedal type/metadata pairs are 50/0 (1,391 kills), 20/1
(1,395 deaths), and 10/2 (159 generic mode highlights). Mode metadata does not
identify a specific objective action. Unknown types remain `SummaryKind::Other`.
For medals, every observed byte-47 value matches the CMS sorting weight; it is
not another event subtype.

Each event retains the raw type/metadata/flag, recorded name and XUID, and exact
source windows for the XUID and tail. These windows do **not** imply the whole
record or all 60 tail bytes are semantically decoded. Offsets are MSB-first,
half-open, relative to the decompressed packet payload. Timestamps are exported
as integer microseconds on the recorded summary timeline; no match-clock offset
or rounding is inferred. Separate records with identical player/time/medal are
retained, including awards sharing a timestamp.

Packet truncation, duplicate footer indices, and unsupported versions return
`DecodeError`. Unaccepted records cause an explicit declared-count mismatch.
A missing footer is distinguishable from a valid empty footer; neither is filled
from the stats API. Unsupported footer packet types are counted as unparsed.

## Upstream APIs

For offline callers with decompressed chunks:

```rust
use halo_api::theater::{decode_summary_events, validate_summary_events};

let report = decode_summary_events(&chunks, 41)?;
assert!(report.matches_declared_counts());
let validation = validate_summary_events(&report, &match_stats);
assert!(validation.matches_stats());
```

Only type-3 chunks are needed. `Film::try_from_chunks` uses this same decoder
for `Film.summary_events` and additionally binds roster indices when available.
The full Film diagnostic list flags missing/count-mismatching summaries.
Older Film JSON without the new optional event fields still deserializes.

For connected callers:

```rust
let report = halo.match_summary_events(&match_id).await?;
let checked = halo.match_summary_events_with_validation(&match_id).await?;
```

Use either call as needed. The first downloads the manifest and footer chunks
only. The second also fetches match stats concurrently and returns both the
original decoded report and validation diagnostics. It checks each medal
identity, so matching total medal counts cannot conceal a wrong mapping.
Unmapped medal codes make validation incomplete. Neither call replaces film
observations with API values.

`film_medal_definition(id)` and `FilmMedal::name_id()` expose the mapping.
`FilmMedal::metadata(&catalog)` resolves localized official metadata by `NameId`.
Existing `clients::hi::film` imports, `FilmMedal`, `KNOWN_FILM_MEDALS`, and
`match_highlight_events` remain available; the latter now uses footer-only
downloads for v41. The established highlight report still exposes aggregate
counts, adapted from current validation. The old `decode_events` / `validate_events`
byte-scanner workflow and speculative version fallbacks are removed; use the
summary APIs above for decoding and per-medal validation.

## Reproduce

In the running recording inspector, choose **Match events** and then an event
from the selector. Named medals are linked to their exact eight-bit film code;
the stats `NameId` is shown as a catalog mapping, not counted as extra decoded
bytes. Packet and whole-film coverage include the new fields. Unknown state
remains opaque/unparsed, so event-count completeness does not imply high byte
coverage. **Refresh decoding** clears cached annotations and totals.

The inspector calls the same example with `--stdout` to consume the current
native result directly, without writing exports or reading stale summary JSON.
Large summaries page their field lists alongside visible bytes for responsive
navigation. See the [inspector guide](examples/theater_inspector/README.md).

From the repository root, decode an already downloaded film without network:

```sh
cargo run --release --offline --manifest-path experiments/Cargo.toml \
  --example decode_film_events -- experiments/films/ranked-arena/02-oddball
```

This writes `summary-events.json`. If `settings/match-stats.json` exists, it also
writes `summary-events-validation.json` and checks its match ID before using it.
The example fails on a declared-count or stats mismatch while retaining the
diagnostics. It reports loading/decoding time separately from export/validation.

To cache independent references (authentication required):

```sh
cargo run --release --manifest-path experiments/Cargo.toml \
  --example cache_film_event_references -- \
  bandit/01-evo ranked-arena/02-oddball octagon/02-ar-kill \
  octagon/03-first-to-50 raids/01-hour-long-raid
```

The cache example skips existing files. Downloads and generated reports remain
under ignored `experiments/films/`. To decode every cached footer, build once:

```sh
cargo build --release --offline --manifest-path experiments/Cargo.toml \
  --example decode_film_events
for metadata in experiments/films/*/*/film.json; do
  experiments/target/release/examples/decode_film_events "${metadata%/film.json}"
done
```

For a match ID without downloading the gameplay chunks, the root example uses
the new validated client API:

```sh
HALO_MATCH_ID=4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73 cargo run --example film_events
```

## Validation and limits

Release-mode measurements on September 13, including footer loading and decoding:

| Film | Events | Medal awards | Time |
| --- | ---: | ---: | ---: |
| Ranked Bandit | 345 | 55 | 9.5 ms |
| Ranked Oddball | 734 | 132 | 19.7 ms |
| Controlled AR kill | 3 | 1 | 0.2 ms |
| Octagon first to 50 | 264 | 66 | 7.9 ms |
| Hour-long raid | 2,321 | 468 | 60.0 ms |
| All 32 films | 3,667 | 722 | About 100 ms |

All declared counts match. All five nonempty films pass per-human-player
kill/death and per-`NameId` comparisons; no observed medal is unnamed or has a
sorting-weight mismatch. The other 27 films have valid empty summaries.
These are warm local timings, excluding compilation and network access.

The checked-in [AR-kill footer and stats fixtures](../src/theater/fixtures/summary-v41.md)
exercise a real unaligned recording in ordinary fast library tests. Other tests
cover malformed framing/tails, unknown codes, duplicate awards, catalog mappings,
JSON compatibility, and incorrect medal identities despite equal totals. HTTP
mock coverage verifies that gameplay/bootstrap chunks are not downloaded.

Validation passes 104 library tests, four doc tests, Clippy, and the
`wasm32-unknown-unknown` library check. Full Film integration was also checked on
Oddball: all 734 events match the footer-only output and bind to roster players;
player, projectile, and clock streams are unchanged from the previous export.

The fixed identity-to-tail offset is guarded by framing, an end marker,
reserved fields, strict UTF-16, and declared counts, but it is not a decoded
schema for the intervening state. New game builds may introduce unsupported
layouts. Bot/AI events, assists, objective subtypes, and kill/death pairings are
not established by these results. Complete summary-count coverage is not a
claim to decode every byte or gameplay action in a film.
