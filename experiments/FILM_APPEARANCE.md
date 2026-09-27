# Recorded armor customization

## Selected-player armor and additional pieces

The replay has a separate **Selected player · Armor** panel below the timeline.
Selecting a roster button or floating player card updates this one panel. It
follows the latest appearance at or before the playhead, including reverse
seeks, independently of death/respawn. Before the first sample or in an older
export without appearance, it shows unknown. Equipment is not added to every
floating card or rendered onto the schematic body.

The BR75/Shock and AR/Stalker controls contain the same added equipment. Their
base armor remains Mark VII, Cadet Brick, Cavallino, Arcadian Green, Capaxx and
UA/Type SA. The accessory fields independently match official Game CMS metadata:

| Slot | Recorded identifier(s), hex | Item |
| --- | --- | --- |
| Chest | `722f60d1` | TAC/Packrat Rig |
| Utility | `f048a42d` | Myesel Ammo Pouch |
| Wrist | `9a201f3b` | TAC/Holodyne Milspec |
| Left shoulder | `f9d6cff1` | Alpha Augmentor |
| Right shoulder | `96f00c69` | Alpha Augmentor |
| Mythic effect | `5294fd4e 9ad0ac33 f4f430dc 607b1b0f` | Beyond the Burrow |

These are five `TagId` matches and one complete `FxIds` array match. Both clips
repeat the same identifiers in all six appearance snapshots. The earlier
Cadet Blue/Brick controls have zeroes in those fields. Zero attachment tags are
not promoted to a general “None” label: geometry-based variants can use other,
unparsed fields. Unmatched identifiers show **Unidentified**. Missing fields
in older exports show **Not decoded**, never a fabricated empty loadout.

The offline name catalog matches region/permutation pairs for helmet, gloves
and knees, coating StyleId, armor VariantId, visor identifiers, attachment TagIds
and mythic FxIds. Conflicting names stay unidentified. It does not query live
customization during replay. The account was wearing Chimera when the reference
metadata was fetched; these films still correctly show Mark VII. Helmet
attachments, armor effects, kits, emblems and remaining model fields stay
unsupported. **Recorded identifiers** expands the selected sample and source.

Official references are under `films/analysis/appearance/attachment-catalog/`;
the compact presentation catalog is `examples/theater_viewer/armor-catalog.json`.
`attachment-evidence.json` records the two films and the reference paths;
`attachment-corpus-validation.json` audits the refreshed corpus. No film values
are replaced by the account's currently equipped items.

The 32-film refresh contains 4,895 appearance snapshots. All previous appearance
fields and all unrelated player streams were preserved. The native fixture
checks cover the added fields at every bit alignment and old-export defaults;
`node examples/check_armor_names.cjs` checks metadata matching and ambiguity.

## Original coating controls

The High Ground controlled pair identifies **armor coating in the film bytes**.
Only the coating differs between the two complete 3,493-byte player-snapshot
payloads. Each film repeats its snapshot six times, with the same appearance.

| Match | Reported coating | Recorded StyleId | Hex |
| --- | --- | ---: | --- |
| `5156fc6f-47da-4e7b-b54a-857e8d3a3128` | Cadet Blue | 1362766512 | `513a2ab0` |
| `cc336817-2d7a-431a-b704-fe782013943b` | Cadet Brick | 1920803940 | `727d2464` |

These values independently match Game CMS `StyleId.m_identifier` in:

- `Inventory/Armor/Coatings/002-001-olympus-80607276.json` — Cadet Blue.
- `Inventory/Armor/Coatings/002-001-olympus-bc220d04.json` — Cadet Brick.

The live customization endpoint supplied reference item paths only. The historical
values above come from the films; changing the account today cannot change them.
This is not an RGB color, inventory path hash, or a universal cosmetic asset ID.

## Controlled setup and other matches

The recorder reports Nuzzles idle on High Ground, Mark VII without a kit,
Cavallino helmet, Arcadian Green visor, Capaxx gloves, and UA/Type SA knee pads.
Chest, shoulders, wrist, utility, emblem, armor effect, and mythic effect are
reported absent. Full descriptions remain in `films.csv`. Both manifests ended
normally; film lengths are 96.479 and 96.525 seconds.

The constant part of the snapshot also matches official metadata:

| Field | Recorded identifier | Reference |
| --- | --- | --- |
| Armor variant | `13d24f1f` | Mark VII theme VariantId |
| Helmet region / permutation | `e5f7f02b` / `1da90411` | Cavallino RegionData |
| Glove regions / permutation | `7c8065e9`, `62059329` / `13d24f1f` | Capaxx RegionData |
| Knee regions / permutation | `88078d0d`, `0d75f26e` / `13d24f1f` | UA/Type SA RegionData |
| Visor / color variant | `db8cecaf` / `3b41406f` | Arcadian Green VisorId / ColorVariant |

These are model selections, not a fully decoded equipped-item list. Shared
permutations need region context. VariantId alone does not establish a universal
core/kit identifier. Unparsed fields do not establish "none" for accessory slots,
emblems, or effects. Those reported absences have not been promoted to general empty-slot observations.

The original investigation found this snapshot form **966 times across 28 films**:
264 in the Bandit Arena game, 496 in Oddball, and 45 in the full Octagon game.
Each captured player's identifiers remain stable within these recordings.

## Checked layout

Read only packet type **8**. Anchor at the existing roster marker `2d c0`, whose
bit position can be unaligned. All offsets below are bits relative to that marker.
Integer payloads consist of **little-endian bytes**, themselves read MSB-first
from the packed stream. Source ranges are half-open.

| Relative bits | Interpretation |
| --- | --- |
| `[-488,-232)` | Existing 32-byte UTF-16LE roster name window |
| `[-232,-64)` | Required 21 zero bytes |
| `[-64,0)` | Recorded XUID, little-endian u64 |
| `[0,16)` | Checked `2d c0` marker; alternative markers remain unsupported here |
| `[16,46)` | Opaque prefix; observed values 0 or 1 |
| `[46,54)` | Checked value 22 for this base-region form |
| `[54,110)` | Checked zero prefix |
| `[110,1518)` | 22 pairs of signed i32 region/permutation identifiers |
| `[1518,2030)` | Unparsed additional model/customization fields |
| `[2030,2062)` | Chest attachment TagId |
| `[2062,2094)` | Unparsed identifier |
| `[2094,2126)` | Utility/hip attachment TagId |
| `[2126,2158)` | Wrist attachment TagId |
| `[2158,2190)` | Left shoulder TagId |
| `[2190,2222)` | Right shoulder TagId |
| `[2222,2318)` | Unparsed identifiers |
| `[2318,2350)` | Visor ID |
| `[2350,2414)` | Unexported visor parameters |
| `[2414,2446)` | Visor color variant |
| `[2446,2574)` | Four mythic-effect identifiers, matching Game CMS FxIds |
| `[2574,2606)` | Armor variant ID |
| `[2606,2638)` | Armor coating StyleId |

Both controls' first snapshots are chunk 1, payload byte 524113, roster marker
bit 12724. The coating occupies payload bits **[15330,15362)**, spanning bytes
1916–1920. Those five physical bytes are the only differing bytes in the complete
payload. Unaligned extraction yields `b0 2a 3a 51` / `64 24 7d 72`.

## Native API and replay

`Film::try_from_chunks` now exports `player.appearance: Vec<AppearanceSample>`.
Samples contain their timestamp, `ArmorAppearance`, and source window. They bind
to the existing roster by **XUID and gamertag**, independently of pawn lives;
snapshots can precede spawning and remain valid after a death. IDs are signed
32-bit integers matching the CMS representation. Missing streams default empty
when reading older typed Film JSON.

`ArmorAppearance.attachments: Option<ArmorAttachments>` exposes the five
attachment tag fields above. `mythic_effect_ids: Option<[i32; 4]>` exposes the
recorded effect identifiers. Both default to `None` in older exports. Parsing
stays offline and wasm compatible; cached official metadata supplies names in
an independent presentation step.

The decoder uses no network, name-to-item assumptions, match-ID branches, or live
account defaults. Unsupported prefixes/counts and truncated windows are rejected.
SourceSpan covers the checked roster/appearance window and includes opaque gaps;
it is not a claim that every intervening bit is understood.

Both films appear under **Armor customization controls → Cadet Blue / Cadet
Brick** in replay. The schematic avatars keep the viewer's player colors; this
panel displays recorded equipment names without inventing rendered coatings.

## Reproduce and evidence

From `experiments/`:

```sh
HALO_TIMEOUT_SECS=120 HALO_PROBE_CATEGORY="Armor customization controls" \
  cargo run --example download_film_chunks
cargo run --release --example decode_theater_film -- films/appearance/01-cadet-blue --compact
cargo run --release --example decode_theater_film -- films/appearance/02-cadet-brick --compact
node examples/build_decoded_replay.cjs --corpus films
```

`download_customization_reference OUTPUT_DIRECTORY [XUID] [EXTRA_ITEM_PATHS_JSON]`
captures current account customization and raw metadata for equipped items and
optional extra paths. Its provenance explicitly marks current state as reference
data, not historical evidence.

`films/analysis/appearance/coating-samples.csv` contains all 12 coating samples
with source coordinates. `evidence.json` preserves the first decoded appearance
per control and the item-reference paths; `current-reference/` and
`catalog-reference/` retain the fetched JSON. The original binary film chunks
remain beside `decoded-film.json` for each control.

Small captured fixtures in `../src/theater/fixtures/appearance_records.json`
cover both coatings, another armor variant, and a Ranked player. Tests verify all
eight bit alignments, CMS-matched IDs, corruption/truncation, supported packet
type, XUID/name binding, and independence from spawning/death. Native corpus
verification preserves every prior player stream, summary event, and projectile.
