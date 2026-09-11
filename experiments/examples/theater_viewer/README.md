# Theater motion and aiming replay

> Active lab cleanup: 32 films are retained, 31 replayable (natural-end and later controls,
> plus the two Arena games and an hour-long raid). Forced-end recordings and standalone probes are
> archived under `archive/cleanup-2026-09-10/` at the experiments root. Historical
> 36-film/30-clip counts below describe earlier validation runs. See the main
> experiments README for current commands. Python field readers used by the
> byte inspector now live under `examples/legacy/`.


The replay covers the controlled movement/aiming experiments, two-player Octagon,
and both eight-player Ranked Arena matches: Bandit EVO and three-round Oddball.
The recorder visually confirmed the Bandit result on **2026-09-10**. See the
[current format status](../../FILM_FORMAT.md) and
[Ranked decoding evidence](../../FILM_BANDIT.md) for the supported fields and gaps.

All commands below run from `experiments/` (start with `cd experiments` at the
repository root). Downloads and API validation
need authentication; decoding and HTML generation run locally. The Python tools
use the standard library. The browser needs WebGL, with no web server required.

## Native decoded Film workflow

The default workflow is now the upstream Rust decoder and its typed JSON export:

```sh
cargo run --release --example decode_theater_film -- films/weapons/03-br-sniper-zoom
node examples/build_decoded_replay.cjs --corpus films
open films/analysis/theater_viewer.html
```

The builder includes existing `decoded-film.json` files. **Open decoded film**
also imports a typed JSON file directly into the running replay. Both use the
same adapter; neither parses bytes. The current native corpus build has **32
recordings, 31 replayable**. It includes checked spawns in idle clips and leaves
initial aim unknown. Titles, categories and menu order come from `films.csv`.
The first catalog entry opens unless `?clip=group/slug` selects another recording.

When served by Theater Lab, both views read the same `/api/catalog` list and use
the shared `recordings.js` picker. Switching tabs keeps the selected recording.
Aquarius stays in both menus; replay shows **Replay unavailable** with a link to
its inspector until positions can be decoded. A cached recording missing from
the embedded replay can load its existing `decoded-film.json` through the local
server. Opening the HTML directly uses its embedded catalog snapshot and data.
Files opened manually remain local to the replay session.

The **Velocity** toggle shows blue, fixed-length arrows for recent recorded
velocity directions. **Velocity sample** shows unit XYZ, the nonlinear magnitude
code, sample time, and previous/next controls for the selected player. The speed
scale remains unknown. Arrows disappear after 100 ms without a sample, without a
recent position, at death, and on life changes. A held value is labeled as the
last sample; no velocities or positions are synthesized. See
[velocity decoding](../../FILM_VELOCITY.md).

The **CROUCH INPUT** badges now cover both players in **Octagon gameplay →
Octagon · First to 50**. Seek to **2:34.84–2:35.36** to see Nuzzles toggle crouch
during a recorded jump. The badges follow decoded input, reset to unknown after
100 ms without a sample or at death/new life, and recompute when seeking backward.
They do not animate a physical stance or infer a slide. See
[the crouch evidence](../../FILM_POSTURE.md) for the command forms and limits.

**Raid gameplay → Raid · 1 hour** opens the 20-player raid, lasting 1:05:53.667.
The complete timeline is available, including later wire-ID generations. The
large film contributes almost one million recorded positions, so the combined
offline HTML grows with the additional velocity stream and its initial load is
larger than the controls.
No samples are downsampled for export. See [the raid notes](../../FILM_RAID.md).

See [the decoder guide](../../THEATER_DECODER.md) for decoding the whole corpus,
source offsets, performance, and `check_decoded_replay.cjs`. The sections below
retain the historical CSV/Python pipeline for research reproduction; its
30-clip counts and presentation assumptions describe that older export.

## Historical CSV replay build

```sh
open films/analysis/theater_viewer.html
```

On other platforms, open that file in a browser. If the CSVs and scene JSON
already exist, rebuilding the HTML needs only:

```sh
python3 examples/build_theater_viewer.py
```

The builder defaults to `films/analysis/all-films/positions.csv`, the neighboring
`aim.csv`, and all `films/analysis/*/scenes.json` files. Explicit options are
`--csv`, `--aim-csv`, `--corpus`, `--scenes`, `--firing-json`, `--melee-json`, `--grenade-json`, `--weapon-json`, `--zoom-json`, `--vitality-csv`, and `--output`. `--scenes PATH`
selects one scene file instead of discovering all scene files. A scene with the
same ID replaces the corresponding CSV-derived clip.

When present, `films/analysis/firing/events.json` adds firing activity to matching
players. `--firing-json PATH` selects another firing export. Similarly,
`films/analysis/melee/events.json` adds melee activity; use `--melee-json PATH`
for another export. Match IDs, player identities, event ordering, and life/death
boundaries must agree.
When present, `films/analysis/vitality/samples.csv` and its sibling `evidence.json`
add provisional health/shield bars. `--vitality-csv PATH` selects another paired
export. The builder checks match, player, spawn serial, and life boundaries.

The generated HTML embeds the data, CSS, JavaScript, and Three.js. It makes no
network requests. The historical CSV build has **30 clips, 458,491 player position samples,
and 362,112 aim samples**. The catalog tracks 36 films; films without supported
position/aim scenes do not appear as replay clips.
The recording picker uses catalog categories, including **Ranked Arena gameplay**.

**Ranked Arena · Oddball** opens first, with Nuzzles selected. It contains all
eight players across three rounds, with 264 spawns and 248 deaths. Bandit EVO
comes next, followed by the Octagon AR kill and isolated right-turn aiming clips.
Both Ranked films include checked shooting/melee events and partial health/shield
observations. Oddball carrier/scoring remains undecoded; see
[Oddball evidence and limits](../../FILM_ODDBALL.md).

## Reloads and magazine ammo

Choose **Weapon controls → Manual vs automatic reload**, or open
<http://127.0.0.1:8766/replay?clip=weapons/02-reload-comparison> with the lab server
running. The replay shows reload starts and recorded magazine amounts for three
Bandit shots and twelve pistol shots. The Bandit refills to 15 and the pistol to
12. Manual and automatic reloads share the same event form; cause comes from the
recorder description. The 500 ms LOAD pulse marks the start, not duration.

Weapon/ammo values stay unknown until observed, are cleared on selection and
death/new life, and use a dashed underline when held. Hover for sample time/age.
The native replay now includes **2,306 Oddball**, **1,071 Ranked Bandit**, and
**16,288 raid** magazine observations. In Oddball, select Nuzzles at film
**30.040560–31.408623 s** for 14 → 13 → 12 → 11, then **36.113054 s** for the
recorded refill to 15. A firing observation without a supported magazine update
supplies its checked slot and holds the prior quantity while weapon and slot stay the same;
it does not subtract ammunition. Weapon-set/identity changes clear that
association. Coverage remains partial; energy ammo, reserves and starting
inventory remain unknown.

Choose **Weapon controls → BR75 & Shock Rifle** for the new switch control:
slot 1 at 57.011430 s, then slot 0 at 66.971899 s (the UI displays slots 2 and 1).
There are no firing or magazine observations in this clip. Spawn references
calibrate the BR75/Shock fingerprints, but their inventory binding is unparsed,
so this entry keeps gun names and ammo unknown. The replay recognizes seven
calibrated weapon fingerprints in either firing slot, including BR75 in the
older Ranked matches and raid. See [weapon identity evidence](../../FILM_WEAPONS.md).

For the historical CSV workflow, `analysis/weapons/events.json` merges automatically, or pass
`--weapon-json PATH`. See [weapon evidence](../../FILM_WEAPONS.md).

**Weapon controls → AR & Stalker Rifle** directly shows the Stalker name at
**59.310053 s**, followed by Assault Rifle with magazines **35 → 34 → 33** at
**68.569291–68.736064 s**. Both recorded switches (56.957497 and 67.050945 s)
clear the name until the next shot. Stalker ammo stays unknown; the cooling-shaped
component is not a magazine count. Stalker names also appear on 279 raid shots.
See [the control evidence](../../FILM_WEAPONS.md#stalker-rifle-firing-control).

## Selected-player armor

The separate **Selected player · Armor** panel is below the timeline. It shows
only the selected player; choose another roster button or floating card to
change it. Collapse the panel using its header. Armor never crowds the floating
player cards. **Recorded identifiers** expands the sample and its source.

The BR75/Shock and AR/Stalker controls show Mark VII, Cadet Brick, Cavallino,
Arcadian Green, Capaxx, UA/Type SA, TAC/Packrat Rig, Myesel Ammo Pouch,
TAC/Holodyne Milspec, Alpha Augmentor on both shoulders, and Beyond the Burrow.
Names come from the film's identifiers matched to cached official metadata.
Live account equipment never supplies historical values. Before a sample, with
missing old-export fields, or with unrecognized identifiers, the panel shows
unknown. Zero tags do not imply every geometry-based slot is empty.

Appearance follows the playhead in both directions and persists independently
of lives. The 3D avatar remains schematic. Helmet attachments, armor effects,
kits, emblems and some model choices still need decoding. See
[appearance evidence](../../FILM_APPEARANCE.md).

## Scope stages

Choose **Weapon controls → BR / S7 scope steps**, or open
<http://127.0.0.1:8766/replay?clip=weapons/03-br-sniper-zoom> with the local server
running. Its five observations read **SCOPE 1 → UNSCOPED → SCOPE 1 → SCOPE 2 →
UNSCOPED**. This establishes zoom stages, not magnification or camera FOV.

Scope readouts and previous/next navigation also show the 538 Bandit and 871
Oddball observations. Values describe the last accepted sample; held values use
a dashed underline and have timestamp/age tooltips. Ranked coverage is partial.
Selection, death, and new life clear old values. Missing observations show `?`.
The camera FOV stays unchanged. `analysis/zoom/events.json` merges automatically,
or pass `--zoom-json PATH`. See [scope evidence](../../FILM_ZOOM.md).

## Inspect recording bytes

The **02 · Recording inspector** tab opens the local format inspector. Start it
from `experiments/` with `python3 examples/theater_lab.py`, then open
**http://127.0.0.1:8766/**. It links hex, ASCII, individual bits, decoded fields,
and unparsed regions for all cached recordings. The server also serves this
replay through its Motion replay tab. See the
[inspector guide](../theater_inspector/README.md).

## Download the captures

To find the latest match ID and UTC start/end times without a scoreboard request:

```sh
HALO_GAMERTAG=Nuzzles HALO_TIMEOUT_SECS=120 cargo run --example latest_match
```

The [CSV catalog](../../films.csv) describes all 36 tracked matches. Downloading
skips films whose `film.json` already exists, so rerunning resumes incomplete
corpus downloads. CSV quoting preserves descriptions containing commas.

```sh
HALO_TIMEOUT_SECS=120 cargo run --example download_film_chunks

# Optional: download only this category, or only the new Oddball match.
HALO_PROBE_CATEGORY="Ranked Arena gameplay" cargo run --example download_film_chunks
HALO_PROBE_FILM=ranked-arena/02-oddball cargo run --example download_film_chunks
```

`HALO_PROBE_MANIFEST` can select another catalog with the same CSV columns.
The original TSV manifests are frozen under `archive/manifests/` and are no
longer downloader inputs.

Films land under `films/<group>/<slug>/`, with decompressed chunks and the real
chunk metadata in `film.json`. `HALO_FILM_CORPUS` changes that root. The shared
Halo API/auth timeout defaults to 60 seconds; `HALO_TIMEOUT_SECS` overrides it.
Xbox sign-in/XSTS timeouts are controlled separately by the dependency.

## Rebuild controlled-film CSVs

`film_motion_probe` supports the **34 controlled films**. It does not support the
Ranked map's 18/18/15 coordinate layout. Running it over the mixed corpus can fail
with a counter mismatch after opening its output CSVs. Use an explicit
`HALO_PROBE_FILM=group/slug` for one capture, or the isolated corpus below for the
complete controlled set. Category/manifest download filters do not filter
`film_motion_probe`. Oddball also uses a separate Python scene builder.

This command creates temporary directories with links to the cached controlled
files, decodes into a temporary output, and copies the three CSVs into
`all-films` only after a successful run. It leaves the original corpus intact.

```sh
python3 - <<'PY'
import csv
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

corpus = Path('films').resolve()
output = corpus / 'analysis/all-films'
with Path('films.csv').open(newline='') as catalog:
    films = [row for row in csv.DictReader(catalog)
             if row['analysis_profile'] in ('controlled', 'octagon')]
with tempfile.TemporaryDirectory(prefix='halo-controlled-') as temporary:
    root = Path(temporary)
    for row in films:
        group, slug = row['group'], row['slug']
        shutil.copytree(corpus / group / slug, root / 'corpus' / group / slug,
                        copy_function=os.symlink)
    env = dict(os.environ, HALO_FILM_CORPUS=str(root / 'corpus'),
               HALO_MOTION_OUTPUT=str(root / 'decoded'))
    env.pop('HALO_PROBE_FILM', None)
    subprocess.run(['cargo', 'run', '--offline', '--example', 'film_motion_probe'],
                   env=env, check=True)
    output.mkdir(parents=True, exist_ok=True)
    for name in ('positions.csv', 'aim.csv', 'inputs.csv'):
        shutil.copyfile(root / 'decoded' / name, output / name)
PY
```

The linking step uses filesystem symlinks (available on the current macOS
workspace). The expected controlled exports are **4,605 positions, 1,798 aim
samples, and 130,112 first-player movement-input pairs**. The Ranked data is
exported separately as a scene, not appended to these CSVs.

Optional trace views reuse those exports:

```sh
python3 examples/plot_aim.py --directory films/analysis/all-films
python3 examples/plot_controller_inputs.py --directory films/analysis/all-films
```

They write `aim_trace.html` and `controller_trace.html` beside the CSVs.

## Rebuild multiplayer scenes

The following API checks produce validated roster/event exports. If those
validated files already exist, skip directly to the Python builders. Running
`film_roster_probe` without `HALO_VALIDATE_STATS=1` inspects local data but writes
an unvalidated export, which the scene builders will reject; use a separate
`HALO_ROSTER_OUTPUT` for such inspection.

```sh
HALO_TIMEOUT_SECS=120 HALO_VALIDATE_STATS=1 \
  HALO_ROSTER_OUTPUT=films/analysis/octagon cargo run --example film_roster_probe

HALO_TIMEOUT_SECS=120 HALO_VALIDATE_STATS=1 HALO_PROBE_FILM=bandit/01-evo \
  HALO_ROSTER_OUTPUT=films/analysis/bandit cargo run --example film_roster_probe

HALO_TIMEOUT_SECS=120 HALO_VALIDATE_STATS=1 HALO_PROBE_FILM=ranked-arena/02-oddball \
  HALO_ROSTER_OUTPUT=films/analysis/oddball cargo run --example film_roster_probe

python3 examples/build_octagon_scene.py
python3 examples/build_bandit_scene.py
python3 examples/build_oddball_scene.py
python3 examples/film_firing_probe.py
python3 examples/film_melee_probe.py
python3 examples/film_grenade_probe.py
python3 examples/film_weapon_probe.py
python3 examples/film_zoom_probe.py
python3 examples/film_vitality_probe.py
python3 examples/build_theater_viewer.py
open films/analysis/theater_viewer.html
```

The roster probe defaults to the two Octagon films. The Bandit and Oddball
builders each validate their specific match ID and layouts. Oddball uses the
`oddball` catalog profile and a separate scene builder. Firing, melee, and
vitality probes support both Ranked captures. Melee reads firing's checked life
export, so regenerate firing before melee, grenades, weapons, and scope. The grenade probe adds
362 throw events and two controlled projectile paths.

## Controls and timeline

- Drag to orbit; scroll to zoom. **Focus** centers and zooms on the selected
  player's current displayed location. **Reset view** returns to the overview;
  **Top** uses a view from above.
- Select a player to inspect their coordinates and aim. In the Ranked match,
  timeline coverage and markers also follow the selected player.
- Play/pause, change speed, scrub, loop, or step to the previous/next observation.
  Space toggles playback; left/right arrows step; R resets the camera when focus
  is outside form controls.
- **Action window** uses the clip's chosen analysis range; **Full film** includes
  startup and post-game. Timestamps are film seconds, not the match countdown.
- **Window** selects the visible timeline span: full range, 5 minutes, 1 minute,
  10 seconds, or 1 second. Use **+ / −** or scroll over the timeline to zoom;
  wheel zoom centers on the pointer. **Shift + scroll** or **Pan window** moves
  the visible range without seeking. Manual panning pauses playback.
- **Fit** restores the entire timeline; **To playhead** recenters the current
  window on the playback time. Zooming changes only the visible timeline, not
  playback or loop bounds. Playback and observation/event jumps automatically
  reveal the playhead, including when it leaves the zoomed window.
- Green/gold strips show position/aim observations. Red/green marks below the
  Ranked timeline indicate deaths/spawns. **Jump to event** lists all players'
  death/spawn events within the selected time range.
- **Look** toggles the gold ray. **Trail** toggles paths. Ranked trails show the
  last five seconds; the controlled clips retain their full faint reference path.
- Every player has a persistent roster card with separate shooting/melee badges
  and slim shield/health meters, including single-player clips. These remain
  visible when another player is selected or the avatar moves off camera.
- Floating player cards stay anchored above their players. Nearer cards stack
  above farther ones. Unobstructed cards are fully opaque; a card fades only
  when a nearer card overlaps it, with deeper overlaps fading more. This updates
  as the camera or players move, independently of
  selection. Crowded desktop scenes use compact cards with health/shields and
  active combat badges; click a card to select the player and expand their
  details. Small screens use name labels, with full details in the roster.
- **Cards** toggles all floating player cards. The
  choice stays in effect when changing recordings; roster and selected-player
  readouts remain available.
- Orange player badges/rings indicate a decoded firing event for 150 ms of film
  time. A schematic muzzle flash appears when pose is available. The firing
  readout's arrows jump to the selected player's previous/next event; orange
  timeline ticks show the same events. The indicator clears at death/respawn.
- Purple **MELEE** badges and a schematic ground arc pulse for 350 ms of film
  time. The readout arrows and purple timeline ticks navigate decoded melee events.
  This is a display pulse, not a measured animation or strike direction. Shooting
  and melee badges light independently when they overlap.
- Compact blue shield and coral health bars appear in every roster card, above
  players, and in the selected player's **Health & shields** card. The **Health
  changes** arrows jump between
  that player's decoded amount changes. General sample stepping includes vitality.

## Grenades

The replay adds a cyan **THROW** badge, previous/next grenade buttons, and cyan
throw marks on the timeline. These stay independent of shooting and melee.
Accepted throws total 122 in Bandit, 238 in Oddball, and one in each of the two
original grenade controls. The 450 ms pulse is a display duration.

Choose **Keyboard controls - natural end → Grenade throw · natural end** to see
the recorded projectile flight and bounces. Its throw is at 26.408748 s; the ball
appears at the recorded spawn, 26.709491 s, and its track ends at 28.344107 s.
Each control has 87 projectile samples including spawn; earlier aim samples are
preserved. The projectile has its own lifetime and is not removed by an owner's
death. No blast radius or Ranked ballistic trajectory is inferred.

`films/analysis/grenades/events.json` is merged automatically, or select another
export with `--grenade-json PATH`. The builder checks match/player/life binding,
ordered finite event times, and projectile ownership/lifetime. The inspector
shows the original throw/clock fields and supported projectile coordinates.
See [FILM_GRENADES.md](../../FILM_GRENADES.md) for evidence and coverage limits.

## Health and shield preview

Bar lengths use provisional scales of **64 shield raw** and **126 health raw**;
the readout displays raw values, not calibrated hit points or percentages.
Unknown amounts show `?` and a hatched bar in every readout. Missing values are
never assumed full. Each amount retains its own observation timestamp: a shield
update cannot refresh an older health sample. Values are held without interpolation
or regeneration simulation; samples older than 100 ms use dim, dashed bars and
tooltips report their age. Recharge delay and recovery state describe
the recorded sample, not a predicted current value. Death clears the values to
`—`; the meters remain visible. Respawn starts with both amounts unknown until
the new life has decoded observations.

For a useful first check, choose **AR kill**, select **Yet**, and play film time
**22.8–25.3 s** at 0.25×. Shields reach zero at 24.086002 s, then the recorded
health amount drops. Health remains unknown before its first decoded sample.
In Bandit EVO, select **ELITExBOSH** around **56.7–58.5 s** to inspect shield recovery,
or **66.148–66.166 s** for simultaneous body/shield recovery.

The export contains 178,472 shield and 2,010 body observations across the two
Octagon films, Bandit EVO, and Oddball. Many immediate Ranked damage frames remain unsupported;
these bars visualize the partial observations and can retain old values across
missing updates. They are not a complete game HUD. See
[FILM_VITALITY.md](../../FILM_VITALITY.md) for the source evidence and limits.

## Positions, aiming, and gaps

**Recorded sample** means a recent decoded observation. **X/Y/Z** are that
position expressed relative to one shared origin: the first track's first X/Y
and the lowest observed Z across the scene. Changing the selected player does
not change this origin. Raw units are uncalibrated, and the floor is schematic.

Only gaps of at most 100 ms within one life are interpolated. Larger gaps retain
and label the last observation; stale Ranked players also fade and show their
last-seen age. Absence of an update does not establish that a player stood still.
The trail never connects across long gaps or different lives.

Yaw interpolation takes the shortest cyclic path across 4095/0. Aim gaps retain
the last aim with a dim ray and an age readout, within the same life. Missing aim
is never inferred from movement. A decoded spawn clears aim from the previous
life; it stays unknown until a new observation arrives. Death hides the body and
ray and places a marker at the last observed position. Seeking backward computes
state from observations, so it also restores earlier lives correctly.

The isolated aiming clips have no exported position track. They use a schematic
pivot, leave XYZ blank, and label the missing position. Before either stream has
an observation, the avatar is hidden. In the two Octagon scenes, stationary
behavior and initial facing follow the recorder's explicit description; initial
facing is labeled reported. Nuzzles switches to decoded aim when available.
Those assumptions are not applied to the Ranked match.

The display uses yaw `yaw_raw * 2π / 4096` and pitch
`(pitch_raw - 1024) * 2π / 2048`, with yaw zero along positive raw X. The recorder
confirmed the isolated aiming view and later the combined Ranked replay visually.
Exact angular endpoints and absolute camera transforms remain uncalibrated; see
[FILM_AIM.md](../../FILM_AIM.md). The schematic gun follows the aim direction.
Beam length, weapon mesh, bullet trajectories, hit locations, damage, and map
geometry are not decoded. Firing pulses come from separate guarded event records,
not aim changes or kill summaries. They show activity, with small discrepancies
from API bullet totals; see [FILM_FIRING.md](../../FILM_FIRING.md).

## Scene data for decoder work

`scenes.json` is an array of scenes. Each has `id`, `name`, `group`, `duration`
(film seconds), `action` (start/end range), `players`, `events`, and a display
`note`. Ranked scenes additionally set `trailWindow: 5` and select Nuzzles
(`selectedPlayer: "5"` in Bandit, `"3"` in Oddball).

Each player has a stable roster `id`, `name`, `color`, `samples`, and `aim`.
The Ranked rows are:

```text
samples: [film_seconds, x_raw, y_raw, z_raw, input_forward, input_left, spawn_serial]
aim:     [film_seconds, yaw_raw, pitch_raw, spawn_serial]
firing:  [film_seconds, spawn_serial, observed_sequence]
melee:   [film_seconds, spawn_serial]
grenade: [film_seconds, spawn_serial]
reload: [film_seconds, spawn_serial]
zoom: [film_seconds, spawn_serial, stage_0_1_or_2]
ammo: [film_seconds, spawn_serial, zero_based_slot, amount]
switch: [film_seconds, spawn_serial, zero_based_slot]
weapon: [film_seconds, spawn_serial, zero_based_slot, window_or_null, "shot"|"selection"]
vitality.body:   [film_seconds, body_raw, state_bits, spawn_serial]
vitality.shield: [film_seconds, shield_raw, delay_ticks, state_bits, spawn_serial]
lives:   {id: spawn_serial, start: spawn_time, death: death_time_or_null, end: next_spawn_or_film_end}
events:  {time: film_seconds, kind: "Kill" | "Death" | "Spawn", player: gamertag}
```

Position and aim are separate ordered observation streams; matching array indices
do not imply matching timestamps. Ranked input columns are `null` because input
mapping is not decoded there. A life's `end` includes the dead interval before
the next spawn; `death` is the separate elimination boundary. The viewer checks
the row's spawn serial before interpolating or holding aim. Oddball uses unique
chronological life IDs even when wire IDs wrap; surviving round resets leave
`death: null` and end the life at the next round's spawn.

Earlier single-life rows omit the serial and `lives`. The Octagon scene uses
`deathTime`, `stationary`, `initialAim`, and `initialAimSource` for its explicitly
reported stationary/facing setup. These fields should not be added to an ordinary
match as a substitute for missing observations. Spawn evidence and CSV record
offsets are kept separately in each analysis directory.

`firing`, `melee`, and `grenade` are merged from their separate exports by the HTML builder. An absent field
means that activity stream is unavailable; an empty array means the probe ran
without finding a supported event for that player. Firing has 3,360 Oddball,
1,579 Bandit, 30 Octagon, and two single-shot-control events. Twelve events after
death and seven identical copies stay in evidence but are excluded from replay.
Sequence gaps are unfilled. Melee has 90 Oddball, 13 Bandit, and two controlled
events; the new controlled scenes keep all 12 original aim samples. See
[FILM_MELEE.md](../../FILM_MELEE.md) for guards and partial coverage limits.

`vitality` is merged from the CSV/evidence pair by the HTML builder. Its two
streams are independent; absent `vitality` means no export, while empty arrays
mean no supported observations for that player. Values never carry across lives.
`window.theaterViewerState` and its `players` entries expose `vitality.supported`,
`body`, and `shield`; known amounts include raw value, source time, age, spawn
serial, state bits, provisional bar fraction, and recorded recovery/delay state.

## Validation and dependencies

The current native workflow is checked with:

```sh
cargo test --offline --manifest-path ../Cargo.toml --lib theater
node examples/check_decoded_replay.cjs
```

This covers the 31 active replay clips, both BR75/Shock selection directions,
BR75 names in either slot, and recorded ammo holding/refills. The measurements
below describe historical probe/browser runs; their sources are archived.

The committed Chrome driver covers all 127 Bandit spawns/122 deaths and
264 Oddball spawns/248 deaths, no aim or
trail crossing lives, deterministic reverse scrubbing, the 1 ms kill/death event
rounding difference, mobile layout, and earlier Octagon/controller/aim/jump clips.
The browser driver uses the generated offline HTML. Set `CHROME_PATH` for a
Chrome installation outside the default macOS path, or `THEATER_REPLAY_URL` for
a served replay. Screenshots are saved in the system temporary directory. API validation compares each
player's aggregate kill/death/medal totals. Visual confirmation is qualitative.
The vitality preview was checked in Chrome against 1,311 Bandit and 2,861 Oddball
sampled observations,
independent health/shield timestamps, unknown/held values, change navigation,
reverse seeks, death/respawn clearing, and desktop/mobile layout. The firing and
earlier motion/aim checks also passed with the overlay enabled.
Combat checks cover all 4,986 firing events, 105 melee events, and 362 grenade
throws, plus all 174 controlled projectile samples, at their exact
timestamps, simultaneous badges, melee pulse expiry, controlled aim preservation,
event navigation, reverse seeking, and controlled negative clips.

`projectiles` is an optional scene array of `{id, player, serial, start, end, samples}`;
projectile samples are `[film_seconds, x_raw, y_raw, z_raw]`. Only small within-life
gaps are interpolated. `window.theaterViewerState.projectiles` exposes visibility,
position, source sample, and staleness independently from player state.

`vendor/three.min.js` is Three.js 0.160.1 under the MIT license in
[`vendor/LICENSE.three`](vendor/LICENSE.three). Downloaded captures and generated
artifacts under `films/` are gitignored.

Weapon checks also cover all 344 reload starts, 17 magazine observations and 15
shots in the new control, selection invalidation, zero/refill, unknown Ranked
ammo, backward seeks, new-life resets, and desktop/mobile readouts.

Scope checks cover all 1,414 observations, five BR/S7 transitions, literal zero
and second zoom, held state, selection/death/new-life resets, backward seeking,
and responsive scope readouts.
