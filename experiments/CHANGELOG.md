# Experiment changes

## 2026-09-13 — recover Octagon motion at terminal command boundaries

- Share exact terminal-command validation with the pawn motion reader. The old
  short End/input guard rejected motion followed by roster 0 / tag 14 or roster 1.
- Correct nonempty crouch/jump tails from eleven bits to ten: paired commands
  establish the boundary; final nonempty commands retain a separate zero
  terminator and exact byte padding. Correct their exported source endpoints.
- Octagon gains 5,098 positions, 4,777 aim, 4,924 velocity, 2,441 shield and 3,778
  crouch/input observations. Same-life gaps >=400 ms containing firing fall
  from 67 to zero. Preserve the replay's 100 ms threshold and recorded-only motion.
- Retain nine full FRAME regression payloads; check both the motion boundary and
  suffix scanner, invalid tags/ticks/buttons/padding, truncation, extra bytes and
  incorrect coordinate layouts. All 97 library tests, Clippy and wasm checks pass.
- Validate all 32 full films: all prior observations survive, with only the
  crouch source endpoint corrected by one bit where it included the terminator.
  Additional observations are confined to Octagon gameplay. Refresh the typed
  film exports and evidence CSVs for replay.
- Document before/after evidence and remaining non-leading-clock limitations in
  [FILM_OCTAGON.md](FILM_OCTAGON.md#movement-freshness-and-terminal-command-boundaries).

## 2026-09-13 — remove the legacy Theater decoder

- Remove `src/theater/legacy.rs`, the old gamertag-based event scanner, permissive
  packet scanner, duplicate bit helpers, unused world/diagnostic types, and
  speculative record-header/component-mask readers.
- Move active registry and roster readers into focused modules and use the
  shared bounded bit reader. Packet models and event counts belong to `types.rs`.
- Keep the highlight client return types through adapters over the current
  summary decoder and validation; unsupported versions now consistently error.
  Share `SummaryKind` with the `FilmEventKind` API name.
- Update `position_probe` to report actual typed lives and positions, removing
  guessed respawn-frame correlations. Document removed low-level helpers and
  their supported replacements in [THEATER_DECODER.md](THEATER_DECODER.md).
- Validation: 96 library tests, four doc tests, Clippy and wasm compilation pass.
  Fresh full exports of all 32 films preserve registry, player/life streams,
  clocks and projectiles exactly, and all 3,667 summary events match the footer
  decoder. Obsolete speculative-parser tests were removed with those parsers.

## 2026-09-13 — connect native medals to recording inspector

- Index summary chunk packets 9/7 and consume the current native footer decoder
  through a read-only `--stdout` mode. Saved summary JSON is no longer needed
  for inspector evidence. Add a Match events shortcut and named event selector.
- Count only exact semantic fields as decoded; keep name padding/guards as
  checked structure and intervening event state as opaque. Catalog NameIds do
  not inflate decoded coverage. Page summary fields alongside visible bytes so
  the hour-long raid remains responsive.
- Add Refresh decoding and source-revision cache invalidation for packet/evidence
  indexes and whole-recording percentages. Check original AR-kill byte spans,
  rejected stale evidence, cache invalidation and native decoder failures.
- Validation: 13 inspector tests, Clippy, all 32 native footer annotations
  (3,667 events / 722 medals), and desktop/mobile browser checks for Oddball's
  full coverage, medal byte selection and cache refresh. Oddball now reports
  14.86% decoded, 1.78% checked structure, 3.08% opaque and 80.28% unparsed.

## 2026-09-13 — upstream recorded match events and complete medal catalog

- Add a wasm-compatible v41 footer-only summary decoder with declared-count
  diagnostics, recorded identities, raw type codes, and exact tail/XUID source
  spans. `Film::try_from_chunks` uses it for the normal typed export.
- Cover all 155 published film medal codes and all 151 current CMS `NameId`
  mappings, following Den's article and pinned SPNKr references. Correct Mounted
  & Loaded and identify 52 awards missing from the previous name table.
- Validate all 3,667 declared events and 722 medals across 32 films. All five
  nonempty films match each human player's kills, deaths, and individual medal
  counts against independently cached match stats. Warm footer loading/decoding
  takes about 100 ms across the corpus.
- Add `match_summary_events`, `match_summary_events_with_validation`, offline
  `decode_film_events`, and the reference-cache example. Existing v41 highlight
  client calls now download footer chunks only. Preserve unknown codes and
  distinct duplicate awards; do not infer assists, mode subtypes, or kill pairings.
- Document the empirical identity-to-tail offset and opaque state in
  [FILM_EVENTS.md](FILM_EVENTS.md); retain real byte/stats fixtures for fast tests.
- Verification: 104 library tests, four doc tests, Clippy, wasm library check,
  and full Oddball export with all 734 enriched events and unchanged motion streams.

## 2026-09-13 — pause geometry research and preserve extraction routes

- Return research focus to the Theater film format at the user's request.
  Document Ekur's experimental level importer and the raw-module alternative,
  including the Recharge asset folder, portable exports and validation work.
- Record feasibility: Forge navigation surfaces are demonstrated; built-in map
  render meshes have concrete external extraction routes. Recharge remains
  unextracted pending game assets. Preserve existing work without treating missing
  assets as an impossibility or adding their coverage to film decoding.

## 2026-09-13 — Recharge mesh availability

- Check the Ranked Oddball film's exact Recharge map revision: only images and
  `.mvar` placements are published; a direct navigation-asset GET returns
  HTTP 404 `BlobNotFound`. Record the missing geometry input and Ekur extraction
  lead. No Oddball mesh has been generated; film coverage is unchanged.

## 2026-09-13 — spatial tree, face links and boundary distances

- Compare AHP's seven small outer polygons with exact-revision `.mvar` placements:
  each has both spawn-like types directly above it. Find an eighth outer pair
  without a separate navigation polygon and 16 placements inside the central
  polygon. Preserve the 30 matches / 2 unmatched records with byte provenance;
  active spawn settings and film-to-map alignment remain unresolved.
- Add upper/lower surface groups to frame separated geometry, including AHP's
  eight upper arena polygons independently of its large canvas floor. Add
  camera panning, cursor-centered zoom and a Frame selection button; selecting
  a group exits tree mode. These controls do not change decoded coordinates.
- Decode the fourth Havok object's six-byte bounds nodes, relative array and
  child offsets. All 49,910 nodes form valid trees; all 24,956 leaf indices
  uniquely reference the spatial cells. Squared-nibble/226 bounds with float32
  intermediates contain every cell without enlarging the decoded boxes.
- Decode the sample face index at +16: all 395,710 positions fit their referenced
  polygon in XY. Decode +40 as squared XY distance to the connected region's
  boundary: all 54,810 supplied values agree within 0.000003805. Keep 340,900
  unavailable sentinels null and preserve 24 opaque bytes per sample.
- Extend the standalone preview with parent/child navigation and cell lookup,
  displaying compressed tree boxes alongside the original cell bounds.
- Capture complete trees/cell headers and samples from every referenced face.
  Validate malformed references, bounds, distance outliers, missing values and
  work budgets. All 46 Python tests pass in about one second; both full assets
  pass the new checks. Core film decoding and film coverage remain unchanged.

## 2026-09-13 — navigation surfaces and spatial samples

- Decode the existing Str8/AHP `navmesh.blob` envelopes as Bond v2 with zlib,
  declared lengths and four bounded Havok TAG0 objects. Add `--navmesh` downloads.
- Read the first `hkaiNavMesh` through a checked SDK/schema and ITEM/PTCH links:
  69 vertices, 15 closed polygons, and four reciprocal shared-edge pairs.
- Establish the fifth block's complete record grammar: 24,956 enclosing boxes
  with 395,710 spatial samples. Every position lies inside its recorded box;
  the remaining 32 bytes per sample stay opaque. These are spatial subdivisions,
  not decoded Forge-object bounds. Other Havok values remain opaque.
- Export JSON with decompressed byte provenance, OBJ polygons, an HTML orbit
  preview and original XYZ samples as binary PLY. No inferred geometry, film
  coordinate transforms, or core/replay changes.
- Export 284 separate `.mvar` shape records using raw and 16.16 scalar values;
  219 have all four dimensions explicit. Keep missing fields null and externally
  sourced family labels provisional. Document why render-model size references
  do not yet settle Neutral's wall dimensions.
- Validation: 36 Python tests, both full navigation assets, all eleven `.mvar`
  files, offline downloader Clippy, and a local Chrome preview render.

## 2026-09-13 — Neutral's minimal Forge map

- Correct the 13 early controls' recorder attribution to Neutral. Add
  `HALO_PLAYER_XUID` to map lookup; Neutral's first history page recovers all 13
  missing map references, bringing exact map provenance to 32 of 32 films.
- Verify that all 13 use the same NuzMapTest revision. Its 1,780-byte custom map
  contains exactly two objects, matching the reported wall and spawn point.
  Preserve the entire map as a captured regression fixture, including the wall's
  three raw candidate scale values. Object dimensions/bounds remain unverified.
- Record that the wall is a flat floor beneath the spawn; find the same type-like
  identifier on 184 AHP Octagon objects. Capture one of those instances for direct
  comparison. Validation: 23 Python tests and offline downloader build/Clippy pass.

## 2026-09-12 — map assets and coordinate anchors

- Add a bounded offline Bond Compact Binary v2 reader and `.mvar` inspector with
  scalar byte ranges, raw identifiers and nulls for omitted coordinates. Nine
  exact-revision assets parse to EOF, exposing 4,595 object records.
- Fit an empirical Recharge transform from 55 distinct Oddball spawn positions.
  Independently match 42 Bandit positions, maximum distance 0.01345 world units;
  retain eight unmatched points per film. Keep the fit separate from exact BSP
  bounds and replay. Aquarius's map placement independently corroborates X/Z.
- Add optional `.mvar` downloads and a single-film filter to `download_film_maps`.
  Capture small object fixtures from four maps. Document the distinct Str8/AHP
  Octagon canvases despite their shared coordinate widths.
- Check the posture controls against snapshot times: all holds fall between
  snapshots. Document a longer observed hold as the next useful control without
  promoting physical crouch/slide hypotheses.
- Validation: 22 Python tests, both earlier engine/menu Bond files, the two-film
  calibration check, and offline Cargo check/Clippy for the downloader. Core
  decoder and replay observations are unchanged.

## 2026-09-11 — pawn velocity direction

- Decode component 1's 19-bit 3D direction and 10-bit nonlinear magnitude code,
  plus its explicit two-bit stationary form. Six movement/jump cardinals, joystick
  circles and multiplayer displacement checks corroborate the cube-face mapping.
  World-speed conversion remains unresolved; no physical speed is invented.
- Add typed `PlayerTrack.velocities` with exact source ranges, old-export defaults,
  life guards and support in input chains, clocked prefixes and weapon records.
  All 32 films retain their pre-existing streams and gain 1,318,149 velocity
  samples, including 880,097 in the raid.
- Add a blue direction arrow, toggle and selected-player sample readout with
  previous/next navigation. Fixed arrow length, gap/death/respawn handling and
  explicit held values keep uncertainty visible. Compact replay arrays preserve
  all samples while limiting the added load for long recordings.
- Export an inspector evidence CSV alongside the typed film. Revalidate each
  velocity field against its bytes and replace opaque payloads with direction
  and raw magnitude. Oddball decoded coverage rises from 12.55% to 14.34% with
  zero annotation errors; aggregate source spans do not inflate coverage.
- Validation: 40 native Theater tests, 12 Python tests, wasm check, Clippy, the
  32-film preservation audit, and focused Chrome checks across controls, Bazaar,
  Octagon, Oddball and the raid. See [FILM_VELOCITY.md](FILM_VELOCITY.md).

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
