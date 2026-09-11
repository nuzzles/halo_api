# Experiment changes

## 2026-09-11 — shared recording list

- Both views use the same cached-film catalog, short titles, category grouping,
  CSV row order and shared menu code. Navigation preserves the selected recording.
- Keep all 32 recordings in both menus, including unresolved Aquarius. Replay
  displays an unavailable state without stale player data and links to the same
  recording in the inspector. Hosted replay can open a local decoded export that
  was absent from its embedded build; offline replay retains a catalog snapshot.
- Validation: 11 Python tests, armor-name checks, and a focused Chrome check for
  identical menus, both navigation directions, Aquarius, local export loading,
  restored gameplay state and mobile layout. Rebuilt the 32-recording replay.

## 2026-09-11 — whole-recording inspector coverage

- Added a compact pie chart, percentages, exact bit counts and JSON export above
  inspector navigation for decoded, checked structure, opaque and unparsed data.
  Totals include every decompressed chunk byte, headers, padding and unknown types.
- Scan chunks incrementally, cache small totals, and cancel stale requests on
  recording changes. CSV offset indexes avoid rereading full exports for each
  chunk; overlapping annotations count each bit once using the existing priority.
- Keep the packet coverage bar and clearly label the whole-recording totals as
  verified inspector coverage. Native source windows do not inflate semantic
  coverage; missing or rejected annotations retain unknown regions.
- Validation: 11 Python tests and the focused Chrome coverage check pass,
  including JSON export, film changes during scanning, cached results and mobile
  layout. The Oddball scan accounts for all 438,720,768 bits across 63 chunks:
  55,061,934 decoded; 7,512,450 structure; 10,225,164 opaque; 365,921,220 unparsed.

## 2026-09-11 — track Theater Lab sources

- Track experiment tools, replay assets, research documentation, and the film
  catalog. Keep the downloaded/generated `films/` corpus, build outputs, caches,
  and local cleanup recovery snapshots ignored.
- Document the download/decode/rebuild workflow for a fresh checkout and which
  historical analysis artifacts remain local.

## 2026-09-11 — selected-player armor and attachment identifiers

- Added a separate collapsible armor panel below the replay timeline. It follows
  the selected player and latest recorded sample, handles reverse seeking and
  persists independently of death. No equipment is added to the floating cards.
- Matched five attachment TagIds and the complete Beyond the Burrow FxIds array
  in both recent weapon controls against official metadata. Added optional typed
  attachment/effect fields upstream; older exports retain unknown values.
- Added an offline item-name catalog and checks for complete region matches,
  unknown/ambiguous IDs, old exports, selection changes and mobile layout.
  Live account equipment never supplies replay state; unresolved slots stay unknown.
- Validation passed: 38 native Theater tests, wasm library check, Clippy,
  metadata-name checks, the 32-film preservation audit (4,895 appearance samples),
  and the 31-clip offline browser regression including selected-player armor.
  A focused browser check also verifies keyboard expansion without starting playback.

## 2026-09-10 — Stalker Rifle firing and combined return switch

- Added `9ea11b0a-dddc-4061-a7f7-ac22fa62a05b` as **AR & Stalker Rifle**:
  32 films, 31 replayable. Its Stalker shot identifies `daf193c7`; three AR shots
  confirm `48c19d2d` and magazine amounts 35 → 34 → 33. The Stalker fingerprint
  also names 279 existing raid shots.
- The return switch shares a record with component 35, `weapon-state-overheated`.
  Added its checked nine-bit continuation form, bounded by 496 captured records,
  to recover primary selection at 67.050945 s. Heat/energy ammo remain unexported.
- Added captured-byte rejection checks, replay checks for both names/switches,
  AR magazines, unknown Stalker ammo, backward seeking and raid Stalker naming.
  See [FILM_WEAPONS.md](FILM_WEAPONS.md#stalker-rifle-firing-control).
- The 32-film refresh preserved all previous samples and unrelated streams.
  Combined component-35 records add 17 older magazine observations and 18 raid
  selections; current corpus totals are 19,723 magazines and 1,338 selections.
- Validation passed: 37 native Theater tests, wasm library build, Clippy, the
  32-film preservation audit and the 31-clip offline browser regression.

## 2026-09-10 — BR75/Shock fingerprints and primary-slot selection

- Added `731ef1f7-a119-49ca-a056-4b8305aaf7c6` as **BR75 & Shock Rifle**;
  31 active films, 30 replayable. Switches decode at 57.011430 and 66.971899 s.
- Split checked firing windows into carried slot, guards, and 32-bit fingerprint.
  All 17,311 independent magazine-slot comparisons agree. Raw windows remain
  exported, with new typed accessors and slot observations even without ammo.
- Calibrated BR75 `2b1824d5`, Shock Rifle `9387a8b9`, and AR `48c19d2d` alongside
  existing names. Shock naming uses reported spawn loadout; no Shock shot is
  decoded in the new control. Initial inventory remains unparsed and unexported.
- Added component-42 primary selection `0010001`; corrected the old claim that
  the first three bits encoded slot. Replay recognizes names in either slot and
  retains only observed ammo through compatible consecutive shots.
- The 31-film refresh retains all existing samples and unrelated streams;
  selection coverage reaches 1,318. Two combined primary-selection/ammo records
  bring raid magazines to 16,273 and the corpus total to 19,703.
- Added captured-byte and browser regressions. See [FILM_WEAPONS.md](FILM_WEAPONS.md)
  and `films/analysis/weapon-identities/` for calibration and corpus evidence.
- Validation passed: 36 native Theater tests, wasm library build, Clippy, the
  31-film preservation audit, and the 30-clip offline browser regression.

## 2026-09-10 — multiplayer magazine ammo

- Generalized weapon-delta boundaries to checked neighboring pawn records and
  accepted literal magazines through 36, with paired inventory field widths.
- Native exports now contain 19,701 magazine samples: 2,305 in Oddball, 1,070 in
  Ranked Bandit and 16,271 in the raid. All previous samples and unrelated streams
  survived the 30-film refresh; bottomless Octagon still has no ammo observations.
- Replay holds recorded slot associations for the same weapon across shots with
  missing ammo updates, so recorded reload refills remain visible. Quantities
  still come only from magazine observations. Reserve and energy ammo stay unknown.
- Added captured-byte regressions and Ranked replay checks; Rust, wasm and Clippy
  checks pass. See [FILM_WEAPONS.md](FILM_WEAPONS.md).

## 2026-09-10 — active lab cleanup

- Reduced the active catalog/replay to 23 films by archiving 13 forced-end clips.
  Kept later controller, aiming, weapon and Octagon controls plus both Arena games.
- Archived 22 superseded standalone source files; only three Rust examples remain.
- Reduced nine Python probe/builder modules to the field readers still required
  by the byte inspector and isolated them under `examples/legacy/`.
- Removed duplicate experiment fixtures; inspector tests use the tracked upstream
  fixtures. Preserved all full sources and original catalog in a recovery archive.


## 2026-09-10 — consolidated upstream decoder

- Added `halo_api::theater::Film::try_from_chunks` with typed v41 observations,
  bounded byte readers, source spans, partial coverage diagnostics, and preserved
  legacy imports. All supported motion/combat/vitality/weapon/scope/projectile
  families now share the upstream decoder.
- Added the chunk-folder example, portable JSON, native replay builder and JSON
  file picker. Native corpus replay has 36 films and 458,516 recorded positions;
  historical counts below retain their original scope.
- Verified 1,139,015 observations/lifetimes against the saved research exports.
  Local release decoding: 6.79 seconds for 36 films; JSON writing: 0.38 seconds.
- Added small Rust fixtures, pipeline round trips, malformed-input checks,
  native replay regression and a wasm CI build. See THEATER_DECODER.md.


## Organization and catalog

- Moved experiment tools, documentation, cached films, and the auth timeout
  extension into this separate Cargo package under `experiments/`.
- Consolidated 33 tracked matches into `films.csv`; download tools and replay
  grouping use it directly, and pending layouts are excluded from checked probes.
- Added Ranked Arena gameplay as a category and registered the Oddball match
  `4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73` for analysis.

## 2026-09-10

### Added

- The 36th film, `0131715c-568e-49d8-af6a-90e63c5c8156`: BR scope in/out,
  switch to S7, first/second zoom, then out. Five original scope transitions
  establish stages 0/1/2 in a separate record with pawn identity/generation.
- 1,414 checked scope observations (five controlled, 538 Bandit, 871 Oddball),
  scope readouts/navigation, held/unknown values, life/selection resets, exact
  inspector annotations, captured fixtures, and browser validation. Magnification
  and FOV remain unknown. Documented in `FILM_ZOOM.md`.
- Current replay: 30 clips, 458,491 positions, 362,112 aim samples. Catalog and
  inspector list 36 films. Controlled input CSVs now contain 130,112 pairs.

- The 35th catalog film, `4050c25f-3da5-4b16-9a50-2fab9660fed4`: three Bandit
  shots, manual reload, switch to pistol, twelve pistol shots and automatic reload.
- Reload-start decoding: manual/automatic share a form; 344 guarded starts across
  35 films. Magazine decoding establishes 1–15 and the two-bit zero, with 23
  accepted observations and four controlled slot-1 selections. Ranked magazines
  and reserve ammo remain unknown. See `FILM_WEAPONS.md`.
- Two weapon-control replay scenes, compact weapon/ammo readouts, independent
  reload badges and navigation, exact byte annotations, captured fixtures and
  browser checks. Replay links accept `?clip=group/slug`. Current artifact:
  29 clips, 458,490 positions, 362,110 aim samples, and 4,986 firing events.

- Checked grenade throws: 122 Bandit, 238 Oddball, and one in each grenade control;
  an additional 82 candidates with unchecked continuations stay out of the replay.
- Recorded grenade projectile paths in the two controls: a spawn plus 86 position
  samples each, independent removal, cyan labels/trails, throw navigation and badges.
- Grenade byte annotations, captured fixtures, and browser checks covering all
  exported throws and projectile samples. Documented in `FILM_GRENADES.md`.
- The 34th film, `83fc43ff-e8d3-4d9d-b747-14c4ffee5a69`, correctly categorized as
  Bandit EVO → Pistol switching after the recorder corrected the initial description.

- Checked melee events and matching companion roster fields: one event in each
  isolated melee film, 13 Bandit events, and 90 Oddball events. Added decoder
  fixtures, inspector annotations, and `FILM_MELEE.md`.
- Oddball firing support with full wire IDs, generation reuse, sequence wrap,
  duplicate/late exclusion: 3,360 displayed events. Added 115,978 shield and 1,442
  body observations with passing state and clock checks.
- Persistent per-player shooting/melee badges, compact shield/health meters,
  melee navigation and timeline marks, and independent simultaneous actions.
- Committed replay browser regression covering every displayed combat event,
  health observations, life resets, controlled aim preservation, and mobile layout.

- Theater Lab Experiment 02: a local, read-only recording inspector with linked
  hex/ASCII/bit and decoded field views, explicit opaque/unparsed regions, exact
  bit coverage, source evidence, packet/time/offset navigation, text/hex search,
  and page export. All 34 cached recordings are selectable.

- Three-round Oddball replay, selected by default: eight players, 264 spawns,
  248 API-validated deaths, 296,138 positions, and 234,952 aim samples.
- Checked round-clock changes, eight-bit spawn IDs and generation-tag reuse,
  plus a 32-bit spawn-layout shift. Source evidence and captured regression
  fixtures are documented in `FILM_ODDBALL.md`.

- Provisional health/shield bars above replay players, with raw values, independent
  sample ages, value-change navigation, and clearing at death/respawn.
- A partial body/shield vitality probe with registry-backed component identities,
  controlled damage/recovery evidence, and checked regeneration-delay ticks.
- Guarded firing-event extraction and replay indicators with player/life binding,
  previous/next firing navigation, and a recorded comparison against API shot totals.
- Theater research examples for caching film chunks with their real metadata,
  inspecting packet/record structure, and exporting supported v41 position, aim,
  and movement-input observations.
- A roster/event exporter with optional per-player kill/death/medal validation
  against the match-statistics API.
- An offline 3D replay with player selection, look directions, trails, timeline
  controls, and death/spawn transitions. Includes controlled recordings and the
  checked eight-player Ranked Arena capture with Bandit EVO starts.
- Capture-specific decoding of map-dependent coordinate widths, sparse pawn
  components, and spawn-serial → roster identity bindings. These findings live in
  examples and research notes; the generic library decoder is not upgraded yet.
- `latest_match`, which prints the most recent match ID and UTC start/end times
  through player match history.
- `HaloAuthClient::from_xbox_client_with_timeout` for configuring Halo
  authentication request timeouts.

### Changed

- Health readouts use slim S/H bars and raw values; sample age and regeneration
  detail are available in tooltips. Unknown/dead values remain explicit and the
  meters stay present. Controlled melee scenes preserve all original aim samples.

- Replay position observations are labeled “Recorded sample” to match “Aim sample.”
- Shared examples use a 60-second Halo API/authentication request timeout,
  configurable with `HALO_TIMEOUT_SECS`. Library defaults remain 10 seconds;
  Xbox dependency timeouts are separate.
- Research documentation distinguishes current findings from superseded padding,
  component-list, and spawn-index interpretations. The recorder visually
  confirmed the Ranked replay on 2026-09-10.
