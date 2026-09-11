# Grenade throws and controlled projectile paths

Run commands from `experiments/`. The new grenade probe reads all **36** cached
films. It exports **362 checked throw events** and two recorded projectile paths.
Grenade type, damage, blast radius, and general Ranked projectile tracking remain
unresolved. The replay displays a cyan **THROW** badge for 450 ms of film time;
the duration is a visibility aid, not a measured throw animation.

## Controlled evidence and corrected recording

| Recording | Match ID | Throw time | Projectile spawn | Terminal event |
| --- | --- | ---: | ---: | ---: |
| `force-end/09-grenade` | `f82b8fa5-b218-459a-bcaf-e5b52ad4feb3` | 20.684191 | 20.984780 | 22.619554 |
| `natural-end/09-grenade` | `6fcc2ec6-fa2d-4d55-91a2-09c7c683e85f` | 26.408748 | 26.709491 | 28.344107 |

Times are film seconds and include startup. Each throw precedes its projectile
spawn by about **300 ms**. Each projectile has a spawn coordinate and **86**
subsequent position samples. Both trajectories show flight, ground contacts,
shorter rebounds, and settling. The sampled coordinates agree between the two
controls even though their timestamps differ.

The newly supplied match `83fc43ff-e8d3-4d9d-b747-14c4ffee5a69` was subsequently
confirmed by the recorder to contain **Bandit EVO → Pistol switching at 0:50**,
not a grenade throw. It is cataloged as `weapons/01-switch-bandit-pistol`.
At 27.041175 s it updates registry component 42, `biped-desired-weapon-set`;
the 27.225008 s event matches the earlier weapon-switch controls. It has **zero**
accepted grenade events and provides an additional negative comparison. Its
cached film is v41, seven chunks, 4,645,119 decompressed bytes. No replacement
frag recording was supplied; grenade findings use the two older throw controls.

## Throw window and player binding

The checked event window is **32 bits**, MSB-first, with offsets relative to the
matched reference. Its last roster bit overlaps the following record's leading
bit, as in the previously checked melee companion. This does not establish
complete record boundaries or the meaning of the leading bit.

| Relative bit | Width | Observation |
| ---: | ---: | --- |
| 0 | 1 | Uninterpreted leading bit |
| 1 | 9 | `101001111` |
| 10 | 8 | Wire ID |
| 18 | 2 | Generation tag: `01` / `10` |
| 20 | 7 | `0000010` |
| 27 | 5 | Persistent roster index |

The unique life serial is `wire + 256 × (generation − 1)` for the supported
generations. Both life and roster must independently agree with the checked
spawn evidence. The export also requires the following **37-bit frame clock**
at relative offset **31**: its 28 signature bits after the leading bit must
match a supported clock prefix. All three Oddball round prefixes are accepted.

This guarded subset contains:

| Film | Displayed throws |
| --- | ---: |
| Forced-end grenade control | 1 |
| Natural-end grenade control | 1 |
| Bandit EVO Ranked | 122 |
| Ranked Oddball | 238 |
| Other 32 films | 0 |

Every Ranked player has accepted throws. No accepted event is duplicated or
after death. Another **82 identity-bound candidates** have other following
record forms; they are retained separately and **not displayed**. Random prefix
matches that fail life/roster validation are counted as rejected candidates.
The counts above are partial activity coverage, not complete grenade inventory
or usage totals. Weapon/grenade switching, shooting, melee, aiming, movement,
and idle controls do not produce accepted throws.

Original throw sources: forced end at chunk 2 / payload byte **479009** / bit 0;
natural end at chunk 2 / payload byte **494875** / bit 0. Source offsets for every
Ranked event and candidate are in `analysis/grenades/evidence.json`.

## Recorded projectile positions

Trajectory support is restricted to the two single-player controls. The
projectile has identity **9216**, generation 1, and a checked sparse component
list. Registry archetype 41 names components 0/1/2 as position, translational
velocity, and forward/up vectors, with projectile-specific components at 18–21.

Supported lists are `[0,1,2]`, `[0,1,2,5]`, and `[0,1,2,20]`. The position
component uses **52 bits**:

| Component-relative bit | Width | Observation |
| ---: | ---: | --- |
| 0 | 3 | `000` flags |
| 3 | 15 | X raw |
| 18 | 15 | Y raw |
| 33 | 17 | Z raw |
| 50 | 2 | `00` suffix |

Components 1/2 occupy **58 opaque bits together** in these forms. The optional
component occupies 29 bits (5) or 9 bits (20). The full supported block must end
at the established 16-bit End/input guard. Velocity, rotation, and the projectile
tick are not decoded by using these lengths.

The two controls share a **226-bit opaque spawn signature** followed by the
same 15/15/17 coordinate window. The spawn at `[16368,16391,109932]` is near the
sole player's checked spawn. The first delta is `[16368,16379,109932]`; its
maximum coordinate difference is 12 raw units. The final sampled coordinate is
`[16355,15661,109862]`. The probe requires exactly one spawn and terminal event,
all samples inside that lifetime, and matching spawn/first-delta coordinates.
It does not generalize this signature to other maps or projectile types.

Each replay path therefore contains **87 recorded positions**, including spawn.
The cyan ball interpolates only gaps of at most 100 ms. The track ends at the
terminal event; a short trail remains for one second. This does not identify
an explosion center or infer a blast radius. Ranked throws have no projectile
path export and never receive an invented ballistic arc.

## Reproduce and inspect

After building the controlled CSVs and multiplayer scenes using the
[replay guide](examples/theater_viewer/README.md):

```sh
python3 examples/film_firing_probe.py
python3 examples/film_grenade_probe.py
python3 examples/build_theater_viewer.py
python3 -m unittest discover -s examples -p 'test_grenade_records.py'
node examples/check_theater_replay.cjs
```

The grenade probe reuses firing's checked spawn/life evidence. It writes
`analysis/grenades/events.json`, `evidence.json`, and `scenes.json`. Scenes retain
the earlier controlled aim samples while adding the checked player spawn.
The builder accepts `--grenade-json PATH` and validates event identity, life,
time ordering, and projectile ownership/lifetime. Projectiles may outlive their
thrower; they have their own lifetime.

In Motion replay select **Keyboard controls - natural end → Grenade throw ·
natural end** and press **Next grenade throw**, or play its action window. Both
Ranked matches now have per-player throw badges, readout arrows, and cyan timeline
marks. Shooting, melee, and grenade badges operate independently.

The recording inspector exposes the throw and following-clock fields, projectile
coordinates, and terminal signature. Unsupported continuations and surrounding
unknown fields remain unparsed; velocity/rotation and spawn metadata stay opaque.
Captured fixtures test original bit values, corrupted/truncated guards, incorrect
player binding, the confirmed weapon-switch negative, and independent projectile
lifetime. Browser checks exercise all 362 throws and 174 projectile observations,
navigation, reverse scrubbing, expiry/removal, and desktop/mobile layout.
