# Projectile motion and recorded thrower references

The native Theater module now decodes supported projectile paths in both Ranked
Arena recordings. Paths use recorded coordinates and velocities; they are never
simulated arcs. The 32-film audit yields 139 tracks in the Bandit game and 283 in
Oddball, 154 in the raid, plus the retained natural-end grenade control. These are partial paths,
not complete grenade counts.

## Spawn and owner fields

Archetype 41 is checked against the film registry before enabling this decoder.
Its components include object position, translational velocity, forward/up,
shield vitality, projectile-at-rest and projectile-command-tick.

Offsets are relative to the supported spawn record:

| Bit | Width | Meaning |
| ---: | ---: | --- |
| 0 | 2 | Record flags; `11` unsupported |
| 2 | 2 | Checked `00` |
| 4 | 14 | Projectile entity identity |
| 18 | 2 | Raw generation tag |
| 20 | 6 | Archetype 41 |
| 26 | 98 | Checked but semantically opaque default-state prefix |
| 124 | 5 | Thrower roster index |
| 129 | 4 | Checked opaque guard |
| 133 | 8 | Thrower pawn wire identity |
| 141 | 2 | Thrower pawn generation tag |
| 143 | 83 | Checked default-state / component / position-prefix bits |
| 226 | Map-dependent | X/Y/Z coordinates, same axis widths as the pawn |
| Following | 2 | Position suffix `00` |
| Following | 1 or 30 | Projectile velocity |

The thrower life is `wire + 256 * ((generation_tag + 3) % 4)`. It must match a
checked player life and roster entry. This is an explicit reference, not a
nearest-player or nearest-throw match. Independent analysis found matching
references for **145 Bandit spawns and 297 Oddball spawns**. Of those, 119 and
237 respectively also have an accepted throw event from that owner 250–350 ms
earlier. A spawn can identify its owner even when the throw event's continuation
is unsupported. One short-signature Bandit candidate failed the full default
state and owner checks and was rejected.

The 14-bit identity and raw generation bind updates to each spawn. A new spawn
for a reused slot bounds the earlier track; no range of unobserved entity slots
is assigned projectile types. The supported identity band is currently 9216–9471.
Other allocation bands and spawn default-state variants remain unsupported.

## Delta components

A supported sparse update has identity14, generation2, two zero header bits,
count3, then strictly ordered 6-bit component indices. Accepted lists are
`[0,1,2]`, `[0,1,2,5]`, `[0,1,2,20]`, `[0,1,2,18]`, `[1,2]`, `[1,2,20]`,
and `[1,2,18]`.

- **0:** `000`, map-width X/Y/Z, `00`.
- **1:** `1` means explicit stationary; otherwise `0`, direction19, magnitude10.
  Direction and speed use the same quantizers as pawn velocity, without its
  outer dynamic-precision bit. See [FILM_VELOCITY.md](FILM_VELOCITY.md).
- **2:** direction gate, optional direction19, roll8. These boundaries are
  checked; a semantic body rotation is not exported.
- **5:** only the captured all-zero 29-bit form is accepted here.
- **18:** at-rest flag, optional-payload gate, optional 19-bit payload. The first
  flag is exported. Presence of component 18 alone is not an at-rest assertion.
- **20:** presence flag plus an optional byte. Its clock semantics are not exported.

The record must end at the checked End/input marker, another parsed projectile,
or a supported adjacent auxiliary header/payload. Direction holes and truncations
are rejected. Only spawn-bound tracks with at least three recorded positions and
no accepted-sample gap over 100 ms are exported. The first position must agree
closely with the spawn. Missing coverage truncates a path rather than bridging an
unknown interval or extending it into a reused identity.

The natural-end control retains all **87 original positions** and its original
terminal time. It now also exports **97 velocities**, including velocity-only
updates and its final explicit rest observation. Its launch magnitude q418 is
10.00059 world units/s. The earlier 58 opaque bits between position and optional
components split into **30 velocity + 28 orientation bits** in the long forms.

## Replay, inspector and limits

Replay shows cyan **PROJECTILE** markers and recorded paths, plus blue direction
arrows controlled by the existing Velocity toggle. Markers and arrows disappear
when their samples become stale. A decoded rest flag can label **AT REST**.
Unknown type, grenade inventory, fuse, explosion center and blast radius remain
unknown. A final replicated position or rest flag does not establish detonation.
For general paths, `end_us` is one microsecond after the last observation; it is
an observation boundary. The original controlled terminal remains separately
retained when its guards pass.

`decode_theater_film` exports `decoded-film.projectile.csv` alongside the Film
JSON and pawn velocity CSV. The inspector seeks to those accepted spans and
rechecks each coordinate, owner reference, velocity code and rest flag against
the original bytes. Opaque spawn defaults and orientation are not counted as
decoded semantics. The CSV is an evidence index, not a second replay parser.

Captured regression fixtures are in
`../src/theater/fixtures/projectile_motion_records.json`. The public
[Infinite reference](https://github.com/JGtm/LevelUp/blob/cf333a3889771c6462dfce9e1bc287a897043a47/apps/go-api/internal/analysis/filmdec/projectiles.go)
provided leads; its slot-range filling and rest-equals-end heuristics are not used.
Local evidence, exact boundaries and owner/identity checks constrain this implementation.

The full refresh preserves prior player streams (Aquarius gains its newly decoded
spawn/input) and all original control positions/terminal. It exports 41,750
projectile positions and 43,064 velocities across 577 tracks. The inspector
revalidated 84,929 source annotations without byte mismatches.
