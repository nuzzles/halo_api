# Theater Lab

Run commands here from `experiments/` (`cd experiments` from the repository root).
The tools, replay assets, research notes, and `films.csv` catalog are tracked.
Downloaded chunks and generated outputs under `films/`, build outputs under
`target/`, and local recovery snapshots under `archive/cleanup-*/` are gitignored.
Decoder code and independent byte fixtures live in the upstream
`../src/theater/` module.

## Active recordings

[films.csv](films.csv) contains **32 films** (31 replayable; Aquarius coordinates
remain unresolved). Its `title` column supplies short
replay dropdown labels; `description` retains the full experiment notes. The replay
builder uses the catalog to override older cached titles and categories:


| Group | Films |
| --- | ---: |
| Natural-end keyboard controls | 13 |
| Controller circles | 1 |
| Pitch and yaw controls | 2 |
| Weapon switch, reload, and scope controls | 5 |
| Octagon idle and AR kill controls | 2 |
| Octagon gameplay: first to 50 kills | 1 |
| Map controls: Bazaar and Aquarius | 2 |
| Armor customization: Cadet Blue / Cadet Brick | 2 |
| Posture control: crouch holds and sprint/slide | 1 |
| Ranked Arena: Bandit EVO and Oddball | 2 |
| Raid gameplay | 1 |

The Bazaar control adds the observed `X17Y17Z16` coordinate layout. Layout names
now describe bit widths; they do not select gameplay-specific parsers. See
[FILM_COORDINATES.md](FILM_COORDINATES.md) for the evidence and remaining limits.
Its `decoded` analysis profile means Rust decoding without legacy probe exports;
the byte inspector can open its chunks but has no historical field annotations.
Aquarius is also downloaded and cataloged. Its idle spawn has a new candidate
coordinate window and changed flags, with insufficient evidence to partition
the axes. The replay builder reports and skips films with no decoded positions;
their chunks and partial JSON diagnostics remain available for investigation.

**Raid gameplay → Raid · 1 hour** contains the 1:05:53.667 recording
`9d644d09-38c3-429a-bf3d-de0ea577815f`: 20 players, 942 recognized lives,
958,688 position samples and 814,749 aim samples. It uses `X15Y15Z17`.
The decoder now handles the raid's continuation spawn flag, alternate coordinate
suffix and all four observed generation tags. See [FILM_RAID.md](FILM_RAID.md).

**Octagon gameplay → Octagon · First to 50** contains the full 50–49 match
between timesknightt and Nuzzles: both players, 100 lives, position/aim, shooting,
melee, scope, and partial health/shield observations. It uses `X15Y15Z17` with an
additional checked spawn prefix. The opponent's first spawn is at film second
146.19; the opening portion contains only Nuzzles. See [FILM_OCTAGON.md](FILM_OCTAGON.md).

**Weapon controls → BR75 & Shock Rifle** contains
`731ef1f7-a119-49ca-a056-4b8305aaf7c6`. Its recorded slot switches occur at
57.011430 and 66.971899 film seconds. The spawn references calibrate BR75 and
Shock Rifle fingerprints; the replay leaves this no-shot clip's gun names/ammo
unknown until inventory bindings are decoded. BR75 shots in older games now
display the name in either slot. See [weapon evidence](FILM_WEAPONS.md).

**Weapon controls → AR & Stalker Rifle** adds
`9ea11b0a-dddc-4061-a7f7-ac22fa62a05b`. Select secondary at 56.957497 s, fire the
Stalker at 59.310053 s, select primary at 67.050945 s, then fire three AR shots
at 68.569291–68.736064 s. Those shots show recorded magazines 35 → 34 → 33;
Stalker ammo remains unknown. The same fingerprint labels 279 raid shots.

The 13 forced-end films were moved to
`archive/cleanup-2026-09-10/films/force-end/`. The archive retains their bytes,
metadata, original 36-film catalog, and superseded probe sources. It is outside
the active corpus and does not appear in the replay or inspector.

## Daily workflow

On a fresh checkout, download the cataloged films, decode them, and build the
replay using the commands below. The downloaded corpus and generated replay
HTML are not included in Git. Historical inspector annotations and recovery
archives are local research artifacts; downloading chunks does not recreate them.

Three Rust examples cover the main replay workflow. A separate
[`download_match_settings`](examples/download_match_settings.rs) example retrieves
the saved variant revision, raw overrides, and linked engine files; see
[FILM_SETTINGS.md](FILM_SETTINGS.md). These external settings do not drive replay.

```sh
# Print the latest match ID/start/end (authentication required).
HALO_GAMERTAG=Nuzzles HALO_TIMEOUT_SECS=120 cargo run --example latest_match

# Download uncached matches from films.csv (authentication required).
cargo run --example download_film_chunks

# Decode a folder offline using halo_api::theater.
cargo run --release --example decode_theater_film -- films/natural-end/09-grenade --compact

# Build the replay from existing typed Film JSON files.
node examples/build_decoded_replay.cjs --corpus films
open films/analysis/theater_viewer.html
```

Use **Open decoded film** to import a `decoded-film.json` directly into the replay.
To decode every active recording, build once and run the binary:

```sh
cargo build --release --example decode_theater_film
for metadata in films/*/*/film.json; do
  target/release/examples/decode_theater_film "${metadata%/film.json}" --compact
done
node examples/build_decoded_replay.cjs --corpus films
```

Download filters: `HALO_PROBE_FILM=group/slug` or
`HALO_PROBE_CATEGORY="Ranked Arena gameplay"`. `HALO_TIMEOUT_SECS` controls
experimental Halo API/auth timeouts; Xbox sign-in has separate timeouts.

## Byte inspector and Python

```sh
python3 examples/theater_lab.py
```

Open <http://127.0.0.1:8766/> for hex/text/bit inspection, or `/replay` for motion.
In motion replay, use **Window**, **+ / −**, or scroll over the timeline to zoom.
**Shift + scroll** or **Pan window** moves the visible range; **Fit** restores
the whole timeline. Playback and loop bounds stay unchanged. Floating player
cards stay above their players, with nearer cards on top. Only cards overlapped
by a nearer card fade; unobstructed cards stay fully opaque. Click a card to select its player, or use
**Cards** to hide them.
See the [replay controls](examples/theater_viewer/README.md#controls-and-timeline).

The **Selected player · Armor** panel below the timeline shows equipment only
for the selected player. Click a roster button or floating card to switch it.
It includes the added chest, utility, wrist, shoulder pieces and mythic effect
in the BR75/Shock and AR/Stalker controls. Names match recorded IDs to cached
official metadata; unidentified pieces stay unknown. See [armor evidence](FILM_APPEARANCE.md).

Recorded magazine ammo now appears in both Ranked games and the raid. In
**Ranked Arena · Oddball**, select Nuzzles around film **0:30–0:36** to see shot
observations and the refill to 15. Old values are marked held; missing updates
do not trigger a simulated countdown. See [weapon evidence](FILM_WEAPONS.md).

The Python **film probes are no longer part of decoding or replay generation**.
Their complete sources and old tests are archived in
`archive/cleanup-2026-09-10/examples-before-cleanup.zip`.

The inspector still needs a small set of historical field readers in
`examples/legacy/` and the matching exports in `films/analysis/`. Those modules
contain only the functions/constants needed for exact field annotations; their
standalone probe/build commands have been removed. Keep them until the inspector
reads exact scalar bit ranges from the Rust decoder. Current Rust `SourceSpan`s
identify checked windows, which can include opaque bits; substituting them for
exact field offsets would overstate semantic coverage.

Other retained Python files serve the lab, load the catalog, test inspector
annotations, or optionally compare Rust exports with historical evidence. They
do not replace the upstream decoder.

## Checks and layout

```sh
# Fast decoder tests, independent of downloaded films.
cargo test --manifest-path ../Cargo.toml --lib theater

# Only the remaining catalog and inspector tests.
python3 -m unittest discover -s examples -p 'test_*.py'

# Optional corpus/browser checks.
python3 examples/check_decoded_parity.py
node examples/check_decoded_replay.cjs
# With theater_lab.py running:
node examples/check_theater_inspector.cjs
```

- `examples/`: download/decode tools, replay builder, lab server and checks.
- `examples/legacy/`: minimal historical readers used only by the byte inspector.
- `films/`: active chunks, typed JSON and required annotation/replay artifacts.
- `FILM_*.md`: retained research findings, including historical corpus counts.
- `src/auth/`: the experimental authentication timeout extension.
- `archive/`: retired captures and reproducibility snapshots.

The experiment package keeps its local API variant for download/auth timeouts;
`decode_theater_film` depends directly on the upstream crate. No old Rust probes
are included in Cargo's example discovery anymore.

See [THEATER_DECODER.md](THEATER_DECODER.md) for the typed API, partial-format
limits, full original-corpus measurements, and wasm compatibility.

[FILM_APPEARANCE.md](FILM_APPEARANCE.md) documents the High Ground coating pair,
the recorded appearance snapshots, and independent CMS identifier matches. Both
controls are replayable; appearance IDs are exported in typed JSON, while the
toy continues to use schematic player colors.

[FILM_POSTURE.md](FILM_POSTURE.md) documents the crouch command tail. Select
**Posture controls → Crouch & slide** to inspect recorded holds and releases.
**Octagon gameplay → Octagon · First to 50** now shows both players' recorded
crouch inputs across respawns; Nuzzles toggles during a jump around film **2:35**.
Physical slide state remains unresolved; the badge is explicitly an input sample.
