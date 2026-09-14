# Map placements, navigation geometry and film coordinates

## Status: geometry research paused

Paused at the user's request on **2026-09-13**. Active research returns to the
Theater film format. Preserve the map decoders, fixtures and extraction findings
for a later return; obtaining geometry is a separate asset task, not a prerequisite
for continuing film decoding.

**Useful map geometry is feasible with the source assets.** We have already
decoded and visualized navigation surfaces from the exact Str8/AHP companions.
For built-in arena render meshes, Ekur and the corrected LevelUp triangle notes
provide concrete extraction routes. Recharge extraction has not been performed:
its downloaded `.mvar` files contain placements, its published revision has no
navigation companion, and we do not currently have its installed game geometry.
This is an asset dependency, not evidence that geometry cannot be recovered.
Complete collision geometry, materials, props and compatibility with the film's
game build still require separate checks.

### Resume later with Ekur or raw game assets

- **Existing extractor:** [Ekur](https://github.com/TheHaloArchive/ekur), a Blender
  importer, documents an experimental level importer for built-in multiplayer
  maps. Its extraction setup uses a Halo Infinite installation's `deploy/`
  folder on Windows or Linux. It imports extracted `levels/*.json` with their
  model dependencies; its documented level support excludes materials and props.
  Verify Recharge support when source files are available. A GLB/OBJ export with
  original placement and scale would provide a practical input for our viewer.
- **Continue format research:** obtain
  `deploy/pc/levels/multi/sgh_blueprint/`, including companion files if present,
  then follow the corrected LevelUp `sbsp` → `rtgo` → vertex/index buffer route
  described below. Inspect references for any shared-module dependencies. The
  source game build, hashes, dequantization and instance transforms need validation
  before using the output as geometry for this recording.

For either route, check the extracted surfaces against the `.mvar` placements
and recorded player positions before replay integration. Keep extraction offline
and export portable geometry data so the core `halo_api` remains wasm-compatible.
Companion-asset decoding does not increase film-byte coverage. Additional Theater
recordings are not needed merely to obtain installed map geometry.

## Current decoded assets

The exact map revisions linked from match history contain `.mvar` assets encoded
as **Bond Compact Binary v2**. The reader structurally parses all eleven downloaded
assets through their final byte: 877,389 bytes and 4,697 object records. It exposes
numeric field IDs, types, nested lengths and byte offsets. This extends the earlier
Bond work on engine/menu settings; it is not a new film packet encoding.

The existing Forge navigation companions also expose closed polygons and a
separate spatial sample block. Their bounded readers and limitations are below.

These are external companion assets. Their structural coverage must not be added
to film coverage. Object placements do not supply mesh/collision geometry, a
complete Forge schema or effective runtime state. Film packet types 1/2/6/8 have
not been established as these Bond root structures.

## Wire format and object fields

The reader follows Microsoft's Compact Binary v2 implementation at commit
`e27170b0f768037087e5d3103b9e8035c0968626`,
[`cpp/inc/bond/protocol/compact_binary.h`](https://github.com/microsoft/bond/blob/e27170b0f768037087e5d3103b9e8035c0968626/cpp/inc/bond/protocol/compact_binary.h).
Structs have a varuint32 byte length including the final STOP. Field headers
contain a five-bit type and three-bit ID; IDs 6/7 select a following uint8/uint16
identifier. Signed integers use zigzag decoding. Floats are little-endian IEEE
values. Lists/sets contain typed counts; maps contain typed key/value pairs.

[`examples/bond_compact.py`](examples/bond_compact.py) checks nested boundaries,
STOPs, integer widths, types, depth and value-count limits. End-exclusive spans
identify values separately from field headers. Trailing bytes are errors by
default. Explicit zero-padding mode supports the earlier menu files and reports
padding separately. Nonfinite IEEE values retain their original bytes in JSON.

Root field 3 is a list of object structs:

| Object field | Observed value |
| --- | --- |
| `2 / 0` | int32 type-like identifier |
| `3 / 0,1,2` | float32 X/Y/Z placements |
| `4 / 0,1,2` | apparent up vector; exported as `field4_vector` |
| `5 / 0,1,2` | apparent forward vector; exported as `field5_vector` |

Missing fields remain **null**. Bond schemas can define defaults; a schema-free
reader cannot assume them. Other numeric fields remain in the optional full tree.
`-361555940` occurs at initial-spawn-like placements; `-1533673853` occurs at the
Recharge placements matching recorded spawns below. These are raw Bond int32
values, not an established cross-format tag-ID conversion.

Metadata path `1/0/0` is retained numerically. Aquarius, Recharge and Str8 values
are bitwise complements of cached `CustomData.TagLevelId`; Bazaar's cached value
does not follow that comparison. The signed reader agrees with Bond; the
identifier relationship needs a separate schema.

### Object bounds and candidate scale

Placements identify object origins, not their bounding boxes. A follow-up scan
finds an explicit three-float property at object path `8/23/*/0/{0,1,2}` on
1,339 Str8 objects, 302 AHP objects and 91 Recharge objects. Many Recharge tuples
are `[1,1,1]` or uniform multipliers such as approximately `[0.89,0.89,0.89]`;
Forge tuples also vary independently by axis. This is a candidate scale vector,
not established dimensions or half-extents. It remains unnamed in the full tree.

Per-object world bounds would require validated local model bounds plus scale,
rotation and translation. Local geometry/bounds may require the corresponding
game asset definitions, and built-in level geometry may reside outside these
placement lists. The enclosing bounds of all object origins would omit object
sizes and are neither complete map bounds nor the film's quantization bounds.

### Separate volume parameters

Object path `8 / 0[] / 0[]` contains separate shape records. Field 0 is an int32
family; fields 5, 6, 7 and 8 are **struct wrappers**, each with an int32 field 0.
The exporter retains the raw integers and divides by 65,536 for signed 16.16
values. Empty wrappers remain null, including missing below-center distances.

The pinned [external Forge schema](https://github.com/JGtm/LevelUp/blob/cf333a3889771c6462dfce9e1bc287a897043a47/.ai/ETAT_DE_L_ART_FORGE_PALETTE_ZONES.md)
suggests family 2 is a cylinder and 3 is a box. Field 5 is radius/width, field 6
is box depth, and fields 7/8 are above/below-center distances. These labels remain
candidates: effective gameplay usage and collision boundaries have not been
verified in these films. Field 6's applicability to cylinders remains unclear.

Across the eleven downloaded `.mvar` files, **284 shape records** are readable;
**219** explicitly supply all four scalars. Counts include base/custom duplicates.
All 16 AHP shapes have family 3 and values `[105,45,5,5]`. Recharge object 215
has `[5,3.1999969482,2.299987793,null]`. NuzMapTest's custom wall/spawn file has
no such shapes; its base asset includes family 2 with `[1,1,2,0.25]`.
[`fixtures/map_shape_records.json`](fixtures/map_shape_records.json) captures
the original structs and scalar spans. These volumes are not the wall's mesh bounds.

## Neutral's two-object control

All 13 early natural-end controls list **Neutral** in the film roster and reference
the same **NuzMapTest** map asset `325b3aee-e747-4580-8e0b-98c36af994cf`, revision
`ae5e40fb-2070-4d6e-902a-f6954be940b8`, on `fo11_blank`. Querying Neutral's history
found all 13 in its first page. The previous map lookup queried only the signed-in
account; the missing results did not establish that these films were too old.
Exact map references are now cached for all 32 active recordings.

The custom `map.mvar` is only **1,780 bytes** and contains exactly two objects,
agreeing with the API's object count and the user's wall-plus-spawn description:

| Index | Raw type-like value | Explicit position components | Evidence |
| --- | ---: | --- | --- |
| 0 | 1759788903 | X/Y omitted; Z = 10 | Wall by the user's two-object setup report |
| 1 | -361555940 | X ≈ 0, Y ≈ 0, Z = 50.0050011 | Same spawn-like identifier as other maps |

The user further confirms that the wall was underneath the spawn and served as
a flat floor, rather than a vertical obstacle. Its Forge object name and displayed
dimensions/scale are unknown. Stationary player height provides a surface-height
constraint, subject to the pawn reference point; it cannot establish the wall's
horizontal extents, thickness, or the meaning of the scale property by itself.

The wall's candidate scale property at `8/23/*/0/{0,1,2}` is
`[100.3921585083,100.3921585083,99.9024353027]`, stored at byte ranges
`[294,298)`, `[299,303)` and `[304,308)`. These values are not yet identified as
dimensions, percentages or multipliers. Independent model bounds or an identified
scale schema are still needed. The full two-object asset is captured in
[`fixtures/minimal_map_variant.json`](fixtures/minimal_map_variant.json).

The same raw type-like identifier occurs on **184 objects in AHP Octagon 2.0**.
For example, object 19 has candidate scale approximately `[0.5,3,0.5]`. This links
the isolated floor object to many gameplay-map placements; recovering its base
shape/bounds could serve all those instances. It does not establish that every
instance uses identical dimensions or collision behavior.

The companion `fo11_blank.mvar` separately contains 100 base-map object records.
Thus the two custom placements do not imply the canvas has only two engine objects
or that all built-in geometry is contained in the small variant file.

External [`cls_all.csv`](https://github.com/JGtm/LevelUp/blob/cf333a3889771c6462dfce9e1bc287a897043a47/.ai/V7.5/dumps/forge_zones/cls_all.csv)
links raw type `1759788903` to `bloc` tag `728854100`, with size values
`[0.1,0.1,0.1]`. Those values were extracted from render-model compression bounds,
not validated instance/collision geometry. They omit local min/max and pivot;
multiplying them by the candidate scale does not establish the floor dimensions.
The object's name and its `food` scale-limit/unscaled-size values remain unresolved.
The external research's unit conversion is also unverified here.

## Navigation geometry from existing companion assets

### Recharge Oddball: mesh source still needed

The Oddball film `4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73` references
**Recharge - Ranked**, asset `336b5174-3579-4fd8-b2f0-922e4a5f7628`, revision
`c0c6705e-167b-4335-afe0-2bafc7290f40`, base map `sgh_blueprint`.
Its published file list contains three images, `map.mvar` and
`sgh_blueprint.mvar`. It does not list a navigation companion. A direct GET of
the exact revision's `navmesh.blob` on 2026-09-13 returned HTTP **404** with
Azure error **BlobNotFound**, resolving the earlier inconclusive timeout.
The check and local asset hashes are cached in the ignored
`films/ranked-arena/02-oddball/settings/mesh-availability.json`.

The custom `map.mvar` contains 305 serialized object records; the API metadata
separately reports 431 objects on the map. Neither these placement records nor
their candidate volume parameters provide the arena's surface mesh.
No Oddball mesh or mesh preview has been
generated. The successful Str8/AHP navigation export below therefore cannot
currently be applied to this recording. This result concerns the exact published
revision; it does not establish that Recharge geometry is unavailable elsewhere.

If this work is resumed, the needed input is a Halo Infinite installation's
`deploy/` folder, or an extracted Recharge level with its geometry dependencies.
No local installation or extracted geometry was found in the locations checked.
See [the Ekur and raw-asset handoff](#resume-later-with-ekur-or-raw-game-assets)
for the two extraction routes and their validation requirements.

### Pinned LevelUp research review

Reviewed the `.ai/` notes and associated source at LevelUp revision
`cf333a3889771c6462dfce9e1bc287a897043a47` on 2026-09-13. The most useful
geometry reference is the later
[`HANDOFF_GEOMETRIE_TRIANGLES.md`](https://github.com/JGtm/LevelUp/blob/cf333a3889771c6462dfce9e1bc287a897043a47/.ai/V7.5/cartes/HANDOFF_GEOMETRIE_TRIANGLES.md).
It describes `sbsp` instances → `rtgo` → mesh LOD records → vertex/index buffer
descriptors → resource bytes → transformed triangles. It reports successful
extraction on Cliffhanger (`ridgeline`), not validation on Recharge.

That handoff explicitly corrects its predecessor: vertices use raw unsigned
16-bit values, the buffer link comes through mesh LOD records rather than
the guessed `0x88/0x8c` fields, and reproducing a transformed bounding box does
not independently validate the vertex buffer. It also leaves the choice of
dequantization bounds unresolved. Treat it as an implementation lead requiring
validation against source assets, not a ready-to-use universal decoder.

The checked-in
[`mapstruct-build/extract.go`](https://github.com/JGtm/LevelUp/blob/cf333a3889771c6462dfce9e1bc287a897043a47/apps/go-api/cmd/mapstruct-build/extract.go)
still exports instance AABB footprints. The triangle scripts and `world.npz`
referenced by the handoff are scratch artifacts absent from this tree. No
exported Recharge triangles were found in the pinned revision. For extraction,
the source directs readers to `deploy/pc/levels/multi/`, making
`sgh_blueprint/` the first Recharge folder to inspect; dependencies may also
require shared modules.

An immediate cross-check against
[`cls_all.csv`](https://github.com/JGtm/LevelUp/blob/cf333a3889771c6462dfce9e1bc287a897043a47/.ai/V7.5/dumps/forge_zones/cls_all.csv)
matches **all 50 distinct type values / 305 records** in our custom Recharge
`map.mvar`: 171 `bloc`, 100 `scen`, 34 `mach`. For example, its first type
`-400138457` maps to `bloc` object tag `-899558595`. None of these 50 types has
a resolved name in the accompanying `palette_noms.csv`. The ignored
`films/ranked-arena/02-oddball/settings/map-type-crosscheck.json` preserves the
source hashes, raw catalog rows and matching record indices. This is an external
ID association; it does not validate geometry, game-build compatibility or
runtime activation, and it adds no film-byte coverage.

The Forge research also supplies candidate gameplay-label hashes, team fields,
instance identifiers and object-name/scale schema paths. Those are useful next
checks against our original Bond tree. Its parser fills omitted coordinates and
shape values with zero; ours continues to preserve them as null until defaults
are independently established. Older film notes likewise contain withdrawn or
contradictory claims, so findings need to be checked against their later
corrections and our controlled corpus before porting.

### Available Forge companions

**New recordings are not required for this lead.** The exact Str8 and AHP map
revisions already list `navmesh.blob`; Neutral's minimal map does not list one.
[`examples/decode_navmesh.py`](examples/decode_navmesh.py) reads these assets
offline and exports JSON with byte provenance, original polygons as OBJ, a
standalone HTML orbit viewer, and optional spatial sample points as binary PLY.

| Asset | Compressed file bytes | Inflated bytes | Navigation vertices | Closed polygons | Spatial cells | Samples |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Str8 Octagon | 56,838 | 388,572 | 29 | 5 | 558 | 8,090 |
| AHP Octagon 2.0 | 1,928,931 | 18,042,228 | 40 | 10 | 24,398 | 387,620 |

### Wrapper and Havok layout

The outer file starts with three big-endian uint32s: version 2, the remaining
length after byte 12, and raw value `0x001fffff` (semantics unknown). At byte 12
is a Bond Compact Binary v2 struct:

| Field | Wire type | Observed value |
| --- | --- | --- |
| 0 | int32 | 1; meaning not established |
| 1 | list<int8> | Exactly one zlib stream |
| 2 | uint32 | Uncompressed length |

The reader follows the actual varint/list lengths, checks the final STOP, caps
both source and inflated data at 64 MiB, and checks zlib EOF, checksum, exact
length and absence of extra streams. Byte 22 happens to start zlib in both files;
it is **not** used as a fixed offset.

The inflated container starts with big-endian `[2,1]`, then five uint32 segment
sizes. Four complete Havok `TAG0` objects follow at byte 28, followed by a spatial
sample block and four zero trailer bytes. Sizes account for the exact file length.
Havok section headers are big-endian: the lower 30 bits give size including the
8-byte header, and the upper bits distinguish containers (0) from leaves (1).
All nested section boundaries are checked.

The first object is `hkaiNavMesh`, SDK `20220100`. Both assets have identical
`TST1`, `TNA1`, `FST1` and `TBDY` schema contents. This is a **narrow schema-gated
reader**, not a general Havok decoder: it rejects a different SDK/schema rather
than applying these offsets speculatively. Root pointers at DATA offsets 24,
40 and 56 select ITEM entries for faces, edges and vertices, with matching PTCH
relocations. Serialized array counts come from ITEM, not the zeroed in-memory
array size slots. ITEM descriptors are little-endian `(type/flags,offset,count)`.

| Array | Stride | Layout |
| --- | ---: | --- |
| Face | 12 | LE int32 start edge, int32 start user edge, uint16 edge count, uint16 user count |
| Edge | 20 | LE uint32 a/b/opposite edge/opposite face, two raw bytes, raw uint16 |
| Vertex | 16 | Four LE float32s; XYZ plus retained W |

All 15 polygons close, all edge indices are in range, and all shared edges are
reciprocal: three pairs in Str8 and one in AHP. External/user edges are unsupported
and rejected. Finite values, nonoverlapping arrays, ownership and enclosing AABBs
are checked. Root navigation AABB at DATA offset 144 and up vector at 192 are
exported separately. **Navigation AABBs are not BSP quantization bounds.**

Str8 vertex extents are X `[-27,27]`, Y `[-18,18]`, Z `[50.03125,70]`.
AHP includes a broad floor near Z50 and small upper polygons near Z98.85;
its extents are X `[-155.6015625,168.75390625]`, Y `[-194.19921875,155.5]`,
Z `[49.9921875,98.85546875]`. Navigation geometry may be offset or eroded and
omit unwalkable surfaces. AHP's nearby spawn is around Z98.66; no position is
adjusted to fit the navigation surface, and no replay coordinate transform is inferred.

The second and third Havok objects name `hkaiClusterGraph` and
`hkaiTraversalAnnotationLibrary`. Their section envelopes are checked, but their
values remain opaque. The fourth `hkcdStaticAabbTree` is now linked to the cells below.

### Spatial bounds tree: explicit links to every cell

The fourth object contains **1,115 nodes / 558 leaves** in Str8 and
**48,795 nodes / 24,398 leaves** in AHP. Each is a complete binary tree with
`2 * leaf_count - 1` nodes, at maximum depths 12 and 17. **Every leaf value is a
unique index into the fifth segment's cell array, covering every cell exactly once.**
These are cell indices, not Forge object or navigation face identifiers.

Both files share the same tree schema. The reader checks its four schema hashes,
SDK, ITEM table and PTCH relocation. DATA offset 24 references an implementation
object; its member at +32 holds an `hkRelArray` header and a domain AABB. The
relative int64 pointer and uint32 count select the same node array as ITEM's
type-12 array. In both files, implementation offset 32, relative-array offset 64,
relative displacement 48 and node-array offset 112 agree. The reader follows
these references rather than searching for plausible floats.

Each node is six bytes:

| Node-relative offset | Bytes | Meaning |
| --- | ---: | --- |
| 0 | 3 | One compressed bounds byte per axis |
| 3 | 1 | High data byte; bit 7 marks an internal node |
| 4 | 2 | Low data uint16, little-endian |

The payload is `((high & 0x7f) << 16) | low`. An internal node's left child is
the next node; its right child is `node_index + 2 * payload`. A leaf's payload
is the cell index. Subtree boundaries, all node visits, unique leaves and the
relative-array bounds are checked; corrupt child offsets cannot silently relink.

For each axis, high and low nibbles inset the parent box:

```text
extent = parent_max - parent_min
node_min = parent_min + extent * high_nibble^2 / 226
node_max = parent_max - extent * low_nibble^2 / 226
```

This matches the squared-nibble codec described in the pinned
[external Havok implementation](https://github.com/kishimisu/Crash-NST-Level-Editor/blob/f1fe4f3e295789c3402d914f0571628d2f7709ef/src/Havok/BVH.cs),
and is independently checked against all local cell boxes. The reader rounds
subtraction, reciprocal scaling, inset multiplication and addition/subtraction
to float32 without fused operations. **Every decoded leaf box contains its
referenced cell box with no tolerance expansion.** Pure float64 arithmetic gave
nine tiny AHP containment violations, at most 0.00000753; preserving float32
operations resolves them. This establishes a compatible arithmetic sequence,
not the engine's exact instruction sequence.

Tree boxes are conservative compressed bounds and are slightly larger than the
stored cell boxes. They are not model bounds or the film's coordinate-quantization
domain. The HTML viewer's **Spatial tree** controls navigate branches or jump to
a cell; cyan shows its tree box and amber shows its original cell box.

The full AHP view includes the large canvas floor near Z50, making the upper
arena cluster look small. **Show → Upper surfaces (8 polygons)** frames those
polygons together at Z98.65–98.86. Upper/lower filters are generated from a large
empty interval between polygon Z ranges; they are display groups, not decoded
object identities. Choosing a polygon/group leaves tree mode. Shift-drag or
right-drag pans; the wheel zooms toward the cursor; **Frame selection** recenters.

### The small outer polygons align with spawn-like placements

The user's suggestion that the outer squares are spawn locations is supported
by a direct comparison with the same revision's `.mvar`. Every small AHP polygon
has both previously identified spawn-like types above it, with their explicit XY
positions inside that polygon. No film coordinate transform or fitted alignment
is involved. The squares themselves are navigation surfaces at these locations.

| Navigation polygon | Respawn-like object (`-1533673853`) | Initial-spawn-like object (`-361555940`) |
| --- | ---: | ---: |
| 2 | 312 | 313 |
| 3 | 274 | 275 |
| 4 | 357 | 358 |
| 6 | 345 | 346 |
| 7 | 278 | 279 |
| 8 | 301 | 302 |
| 9 | 280 | 281 |

The two placements in each outer pair share XY. They are respectively
0.30751–0.30790 and 0.36999–0.37038 world units above the navigation surface.
The map also contains an eighth outer pair, objects **333/334**, at approximately
`(154.1011,-179.5907)` with the same two heights, but there is no separate
navigation polygon underneath it. Thus the visible squares are not a complete
spawn list. Sixteen additional spawn-like placements, objects 244–259, lie inside
the central polygon 5 about 0.0096 units above its surface.

[`examples/check_navmesh_spawns.py`](examples/check_navmesh_spawns.py) reproduces
the comparison, retains original byte ranges/hashes, and reports **30 matched /
2 unmatched** placements. Its maximum vertical gap of 0.5 is an explicit analysis
threshold. This is a spatial association, not a recovered object-to-polygon
pointer. Type names remain provisional, and this comparison does not establish
which placements are enabled or used in the recording. Film-to-map alignment
remains unresolved; raw film spawn positions were not forced onto these objects.

### Spatial sample block

The fifth segment has a count followed by exactly that many variable-sized records:

| Cell-relative offset | Bytes | Observed field |
| --- | ---: | --- |
| 0 | 12 | Maximum XYZ, three LE float32s |
| 12 | 12 | Minimum XYZ, three LE float32s |
| 24 | 4 | Sample count, LE uint32 |
| 28 | count × 44 | Samples with the fields below |

This grammar consumes **all 371,588 / 17,738,428 bytes** in the two blocks.
Every one of the 395,710 combined sample positions is finite and inside its
recorded enclosing box. Most cells contain 16 samples. AHP boxes are approximately
2×2×2; Str8 has 532 boxes approximately 2×2×1 and 26 approximately 2×2×2.
These regular subdivisions support spatial cells, not one box per Forge object.
Their precise navigation-build role is still unknown.

### Samples link to navigation faces and boundary distances

| Sample-relative offset | Bytes | Meaning |
| --- | ---: | --- |
| 0 | 12 | XYZ float32 position |
| 12 | 4 | Opaque; observed zero |
| 16 | 4 | Navigation face index, LE uint32 |
| 20 | 20 | Opaque words; no weapon/object/flag meanings assigned |
| 40 | 4 | Squared XY distance to the connected region's boundary, float32; `0xff7fffff` means unavailable |

All **395,710** face indices are in range, and every sample lies inside the
referenced polygon's XY outline. Str8 samples per face are
`[7776,156,152,0,6]`; AHP has `[193131,193753,4,2,3,717,2,2,2,4]`.
This does not assert exact membership in the polygon's 3D plane: sample Z can
exceed its face's vertex Z range by 0.04078 in Str8 and 0.25003 in AHP.
No point is moved or snapped to resolve that difference.

The last word supplies **5,914** numeric values in Str8 and **48,896** in AHP.
All match squared horizontal distance to the nearest boundary edge of their
**connected navigation region**: maximum absolute errors are respectively
`0.000003805` and `0.000003695`. Squared distance, ordinary distance, and
per-face-only boundaries were compared. Str8 has 45 samples whose nearest
boundary belongs to another connected face; considering that region resolves
all 45. Shared internal edges do not count as boundaries.

The other **2,176 / 338,724** values are exactly `0xff7fffff` (negative maximum
finite float32). They export as null/unavailable; the checker never fills them
from geometry. Why some samples supply this value and others do not is unresolved.
The observed distance relationship is validated on these two maps, and does not
establish a general 3D collision-distance or character-clearance formula.
[`examples/navmesh_samples.py`](examples/navmesh_samples.py) reads individual
44-byte records and checks them against the mesh. New mismatches remain explicit
in the report, with original measurements preserved. Geometric checks have a
bounded work budget.

The resulting chain is **tree leaf → cell → sample → navigation face**.
A raw uint32 comparison of the sample words against each custom `.mvar`'s
type-like identifiers finds no matches (66 types / 1,469 placements in Str8;
15 types / 371 placements in AHP). This limited check does not rule out indirect
object references or another identifier encoding. Individual Forge object bounds
and Neutral's wall dimensions remain unresolved.

JSON retains every cell's bounds, count and exact sample byte span, plus separate
byte totals for geometry values, checked counts, face indices, boundary-distance
fields and the remaining **24 opaque bytes per sample**. Tree-node and cell links
retain decompressed byte provenance. PLY
exports each original XYZ triple, without inferred connectivity or downsampling.
This companion-asset accounting must never increase film semantic coverage.

## Aquarius: independent coordinate anchor

The externally sourced BSP bounds in [FILM_COORDINATES.md](FILM_COORDINATES.md)
convert spawn `[1980,2469,727]` to approximately
`[-20.202537, 0.001472, 3.609372]`. Object 43 in `ctf_aquarius.mvar`, bytes
`[6380,6483)`, contains X `-20.2000007629` and Z `3.6000001431`; Y is omitted.
The same placement appears in the custom asset. X differs by 0.00254 and Z by
0.00937 world units. This corroborates the coordinate interpretation; it does
not establish exact equality, a Y default or a pawn-height offset.

## Recharge: fit Oddball, validate Bandit

Both Ranked films reference asset `336b5174-3579-4fd8-b2f0-922e4a5f7628`, revision
`c0c6705e-167b-4335-afe0-2bafc7290f40`. Its `map.mvar` has 71 objects with type-like
value `-1533673853`, all with three explicit position components.

Calibration uses Oddball only: initialize from extents, alternate nearest-object
assignment and per-axis least squares, then trim inconsistent pairs at 0.4, 0.1
and 0.025 world units. Each distinct raw spawn counts once. Evaluate the separate
Bandit film afterward, without refitting. The fitted equation is:

```text
world[axis] = raw[axis] * scale[axis] + bias[axis]
scale = [0.010327557068459833, 0.010072996894608048, 0.015259647817445567]
bias  = [-993.8852347062459, -1320.2827317482331, -250.01305359694982]
```

| Dataset | Distinct raw spawns | Matches within 0.025 | Median matched distance | Maximum matched distance |
| --- | ---: | ---: | ---: | ---: |
| Oddball, training | 63 | 55 | 0.00525 | 0.01466 |
| Bandit, validation | 50 | 42 | 0.00508 | 0.01345 |

Five accepted Bandit raw points are absent from training; their maximum error
is 0.00696. Many spawns recur across games, so these are five novel raw points,
not 42 novel placements. Eight distinct points in each film remain unmatched;
the report retains their nearest candidates and errors without accepting them.

This is an **empirical calibration**, not a recovered BSP bounds field. Nearest
neighbor fitting can find local minima, and spawn positions can differ from
editor placements. Extrapolating the fit does not establish exact BSP bounds.
The native `CoordinateBounds` API and replay do not consume this fit.

## The two Octagons use different maps

Str8 controls use `fo11_blank`; first-to-50 uses AHP Octagon 2.0 on `fo09_academy`.
Both recordings use 15/15/17 bits. Equal widths do not identify a map.

Str8 object 17, bytes `[2340,2443)`, explicitly contains
`[-4.3499999046, 0.0004870892, 50.0149993896]`. Applying the external Vagabond
transform to the control spawn gives `[-4.34939,-0.00003,50.01103]`, 0.00405 away.
Some AHP spawns also align under that transform; others do not. These are leads
for shared Forge quantization, not established BSP bounds for these canvases.

## Reproduce and validate

From `experiments/`, using the existing Xbox authentication environment:

```sh
HALO_PROBE_FILM=ranked-arena/02-oddball cargo run --example download_film_maps -- --assets
python3 examples/decode_map_variant.py films/ranked-arena/02-oddball/settings/map.mvar \
  --tree --output films/ranked-arena/02-oddball/settings/map-decoded.json

HALO_PROBE_FILM=octagon/03-first-to-50 cargo run --example download_film_maps -- --navmesh
python3 examples/decode_navmesh.py films/octagon/03-first-to-50/settings/navmesh.blob \
  --output films/octagon/03-first-to-50/settings/navmesh-decoded.json \
  --obj films/octagon/03-first-to-50/settings/navmesh.obj \
  --preview films/octagon/03-first-to-50/settings/navmesh.html \
  --points films/octagon/03-first-to-50/settings/navmesh-points.ply
open films/octagon/03-first-to-50/settings/navmesh.html
python3 examples/check_navmesh_spawns.py films/octagon/03-first-to-50/settings \
  --output films/octagon/03-first-to-50/settings/navmesh-spawn-comparison.json

# Requires existing Oddball/Bandit decoded-film.json and exact map metadata.
python3 examples/check_map_coordinates.py \
  --output films/ranked-arena/02-oddball/settings/map-calibration.json
python3 -m unittest discover -s examples -p 'test_*.py'
```

The downloader validates immutable asset prefixes, restricts filenames, rejects
redirects, bounds downloads and reuses cached files. Generated reports and assets
stay under ignored `films/`. The offline reader uses only Python's standard
library; no filesystem/network code enters the wasm-compatible library.

[`fixtures/map_variant_records.json`](fixtures/map_variant_records.json) retains
unmodified Aquarius, Str8, Recharge and AHP structs, with source URLs, revisions,
hashes and absolute offsets. Tests cover captured values, omitted axes, every
truncated prefix, nested-boundary failures, integer overflow, invalid types,
inheritance ambiguity, budgets and padding. All eleven full map assets also pass.

[`fixtures/navmesh_records.json`](fixtures/navmesh_records.json) captures the two
unmodified first TAG0 segments, schema bytes and source hashes. The
[cell fixtures](fixtures/navmesh_cell_records.json) capture original variable-sized
spatial records. Tests reject truncated/oversized envelopes, streams, sections,
arrays, cell records, bad topology and out-of-bounds sample positions. Synthetic
wrappers test short varint lengths independently of the two complete local assets.

[`fixtures/navmesh_tree_records.json`](fixtures/navmesh_tree_records.json) captures
both complete trees and all original cell headers, compressed to keep the fixture
under 400 KiB. Tests independently verify all 24,956 links and enclosing boxes.
The [sample fixtures](fixtures/navmesh_sample_records.json) span every referenced
face, both distance states and a nearest boundary on a neighboring face. Tests
retain mismatching measurements, reject invalid indices and keep opaque bytes
uninterpreted. The Python suite now has 46 tests and runs in about one second
without the downloaded film corpus.
