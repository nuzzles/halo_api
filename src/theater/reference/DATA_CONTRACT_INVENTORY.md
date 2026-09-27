# Native source and output contract audit

Reference: JGtm/LevelUp `feat/v75`, commit
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

## Scope of this checkpoint

The manifest contains all 485 non-test `.go` files below the reference film root.
A live comparison found no missing paths, stale paths, or SHA-256 mismatches.
This proves source inventory completeness, not implementation completeness.
The status vocabulary is historical and mixed; counts of `ported`, `partial`,
and `pending` must not be converted into a parser completion percentage.

The mappings below audit declaration-only files. They do not promote the
algorithms that produce the declarations, or assert full Film fidelity. The
three-layer architecture remains deferred in NEXT_PHASE.md.

## types/source.go

| Native declaration | Rust representation | Data mapping |
| --- | --- | --- |
| ChunkMeta | FilmSourceMetadata in source.rs | Index -> index; ChunkType -> chunk_type; StartMS -> start_ms. |
| Packet | FilmPacket and FilmSource in types.rs/source.rs | Chunk -> chunk_index (source position in FilmSource); Type -> packet_type; TS -> timestamp_us; Payload -> source.payload(packet), bounded by payload_offset/payload_size. |
| Packet.Index | Ordered per-chunk packet slice | source.packets(position)[index]; zero-based enumeration recovers the native Index. It is not a separately stored FilmPacket field. |

The borrowed payload is backed by retained decompressed source chunks. Rust does
not expose Go's mutable alias between Payload and its chunk. That is an ownership
API distinction, not lost decoded bytes; byte-for-byte native film re-encoding
is not provided by this mapping. FilmPacket also retains the two header flag
bytes and source offset/length. Other APIs use FilmPacket.chunk_index as a chunk
number; consumers must preserve which API produced it.

Evidence: source.rs::native_source_loading compares 512 pinned native cases,
including all chunk bytes, metadata, packet counts, source positions, per-chunk
indices, packet types, timestamps and payload bytes. Source directory/bridge tests
cover numbering and metadata alignment separately. This file's declaration data
is represented; general source admission and downstream fidelity remain separate.

## types/killsource.go

| Native type | Rust type | Every native field |
| --- | --- | --- |
| ApparStats | KillMatchingStats (kill_hybrid.rs) | Identite -> identity; Fenetre -> window; BotFenetre -> bot_window; NonRevendiqueeFenetre -> unclaimed_window; CouplesSansIdentite -> pairs_without_identity. |
| Assist | KillAssist (kill_assists.rs) | Name -> name; Index -> index; Rejected -> rejected; Known -> known; Extra -> extra. |
| CoupleStats | KillPairStats (kill_feed_pairs.rs) | MemeInstant -> same_instant; Lus -> read; Recolles -> neighboring_fallback; Perdus -> lost; VictimesBotLues -> bot_victims_read; Muet -> silent; Ambigu -> ambiguous. |

KillPairStats additionally retains agree/contradict diagnostics. Assist.known
remains distinct from an empty assistant name; rejected and extra remain data.
Extra measures distinct additional assistants in attached kill-event records,
not an assertion about every possible assist in the game. Rust counter fields
are nonnegative counts; these mappings concern values produced by v41 decoding,
not arbitrary negative Go struct literals.

Evidence: kill_assists.rs::native_kill_assists_oracle compares complete before/
after targets and all counters for 1,024 native cases, including surplus assists,
fallbacks and disagreements. kill_feed_pairs.rs compares native pair outputs and
counters. kill_decode.rs::tests::normalize explicitly maps all ApparStats and
CoupleStats field names in complete captured decoder comparisons. Those captured
comparisons have separate execution/coverage gates; declaration completeness is
not proof of all kill-source logic or complete-mode coverage.

## Facades and storage adapters

- decfilm/decfilm.go re-exports types/constants and forwards calls into other film
  packages. Its header explicitly describes an alias facade, not a decoder. Its
  symbol-to-Rust API mapping is recorded in PUBLIC_DECODER_AUDIT.md (71 function
  names, 50 aliases and 41 constants). These are inventory counts, not semantic
  completion: do not count the facade as an additional wire grammar or mark its
  full surface ported from the type mappings above. Transitive types, runtime
  contracts and remaining captured acceptance require their separate evidence.
- filmcache/filmcache.go reads the cache manifest, maps source positions to chunk
  numbers, and loads raw chunks. It parses manifest data even though it does not
  decode native recording fields. `film_cache.rs` has native manifest, path,
  writer-snapshot and partial-directory tests. The follow-up
  `../fixtures/cache-partial-metadata-v41.md` covers native row selection and
  full-width i64 type retention. Only conversion to the external FilmChunkData
  model checks that narrower API's i32 range.
  This is distinct from the derivative FilmFacts binary cache codec.
- filmcache/write.go writes supplied raw chunk bytes and a JSON manifest while
  preserving existing files. It does not serialize decoded native records back
  to wire bytes. This cache writer is not evidence of Film re-encoding support.
  Native writer snapshots cover duplicate chunks, existing files and failure
  paths; creation-mode tests cover filesystem permissions. These tests establish
  the tested disk-cache adapter contracts, not decoded-record re-encoding.

## Remaining inventory work

The declaration mappings for types/grammar_bipede.go, grammar_equipement.go and
grammar_monde.go are recorded below; their original inventory task is no longer
pending. Declaration coverage does not close producer/runtime acceptance.
Continue tracing supported outputs through producers, retained Rust structures
and portable exports, including rejected or opaque data, and reconcile replay
document contracts. Audit stale objective/replay manifest entries against live
code and fixtures before changing their status. Keep missing positive complete
VIP/bomb recordings and unvalidated reader contracts as explicit acceptance gaps.

## types/grammar_equipement.go

| Native declaration | Rust representation and field mapping |
| --- | --- |
| MPPFieldCount=4 | EquipmentCreation.mpp_present/mpp_val are arrays of length 4. |
| EquipmentChangeKind | EquipmentChangeKind serializes taken/spent/spawned. |
| EquipmentChange | source carries Chunk/TimestampUS; packet_index carries PacketIndex when source chunks are available; Slot/Counter/Rank/Previous/Kind/Recovered/Gap map to slot/counter/rank/previous/kind/recovered/gap. Native no-rank sentinel maps to None. |
| EquipmentChangeStats | EquipmentChangeStream.walk is Walk; assembly.stats maps Lives/Repeats/CounterJumps/MissedEstimate/LivesFirstOffSpec/Spawned/Taken/Spent/Recovered to the corresponding snake_case counters. |
| EquipmentCreation | Slot -> slot, Gen -> generation, Chunk/PacketIndex/TimestampUS/BitPos -> chunk/packet_index/timestamp_us/bit_pos; HasRef/Ref -> has_ref/reference; HasID/AbilityID -> has_id/ability_id; MPPPresent/MPPVal -> mpp_present/mpp_val; X/Y/Z -> x/y/z; Mask/MaskFull/MaskHasI0 -> mask/mask_full/mask_has_i0; DefaultStateBits/HasAmmo/Ammo/AfterBit -> default_state_bits/has_ammo/ammo/after_bit. |
| EquipmentCreationStats | Slots/Anchors/Overflow/MaskBad/PosBad/Accepted/MaskSparse/MaskFull/NoI0/WithRef/WithID/WithAmmo -> corresponding snake_case fields. |
| EquipmentLifeKey | EquipmentLifeKey.slot/generation, or ObjectLife.slot/generation in head observations. |
| EquipmentPlacement | Life/T0US/T1US/X/Y/Z/GlobalID/Points -> life/t0_us/t1_us/x/y/z/global_id/points. |
| EquipmentSpawnEvent | packet carries Chunk/TimestampUS; packet_index carries PacketIndex; spawned/source Options retain the paired life and its validity guard; Ref2Present -> reference_2_present. |
| EquipmentSpawnStats | Chunks/Packets/Lists/Events/WithSpawned/WithSource/Ref2 -> chunks/packets/lists/events/with_spawned/with_source/reference_2. Rust also reports truncated. |
| GroundWeaponAmmo | Mag/Res -> mag/res. |

The audit found that change and spawn outputs previously had source byte offsets
but no explicit native packet ordinal. Loaded scanners now attach an optional
packet_index using all natively framed packet types in the source chunk.
Standalone equipment assembly has no chunk bytes and retains None rather than
mislabeling payload_offset as an ordinal. Existing synthetic assembly fixtures
historically store their ordinal in payload_offset solely to test ordering; that
fixture convention is not the production field contract.

Equipment creation/placement/change/native-head fixtures independently cover the
remaining values and counters. A mixed keyframe/delta-envelope regression checks
that changes and spawns identify packet ordinals 1/2, while retaining byte offset
33 for the first event, and verifies serialization and standalone unknowns.
This test supplies already-decoded channels/heads, so it verifies attribution,
not positive component decoding or a complete captured recovery scenario.
Positive complete equipment recovery remains a separate acceptance gap.

Recovery diagnostics now preserve every component-consuming speculative probe in
EquipmentChangeStream.recovery_attempts. Each attempt includes source packet,
native packet ordinal when known, slot, candidate header bit offset, whether the
walk produced a candidate, and ordered component reads with raw status, bounds,
fields, references and hooks. Candidate=true does not mean window/counter
validation accepted it, and neither flag establishes a canonical native record.
Pre-component header rejections have no read entry; original Film source bytes
remain the evidence for those offsets. These diagnostics serialize through the
existing Film.equipment_changes field.


## types/grammar_bipede.go (partial audit)

| Native declaration | Rust representation and field mapping |
| --- | --- |
| BipedPickup | head_observations::BipedPickup: TimestampUS/Chunk -> source.timestamp_us/chunk_index; Slot/CatalogID/Class -> slot/catalog_id/class. |
| BipedPickupStats | All ten native counters map to snake_case fields, including other_type; Rust additionally retains truncated. |
| HeldWeaponChangeKind | HeldWeaponChangeKind retains taken/dropped/swapped/restated separately. |
| HeldWeaponChange | TimestampUS/Chunk/Slot/SlotIndex/Family/Low/Previous/Kind -> timestamp_us/chunk/slot/slot_index/family/low/previous/kind. Both halves of the weapon identifier are retained. |
| InventoryDelta | InventoryDeltaRead: Slot -> slot; Chunk/TimestampUS -> source; PacketIndex -> optional packet_index; Grenades -> optional four-element grenades; SelRead -> selection presence; Mask/Sel -> selection.mask/rank, with native no-selection sentinel represented by None. Ammo -> ammo. |
| InventoryDeltaAmmo | InventoryAmmo: WeaponSlot/Mag/FracQ/Res -> weapon_slot/magazine/fraction_quantum/reserve. All three values are independently optional; fraction remains a raw quantum. |
| KeyframeLoadout | KeyframeLoadout: TimestampUS/Chunk/PacketIndex/Slot/Families -> timestamp_us/chunk/packet_index/slot/families. Families preserve ordering. |
| PlayerSlot | PlayerTableSlot: FilmIndex/XUID/Gamertag/SessionToken/Bit/TotalBits/Shorts -> film_index/xuid/gamertag/session_token/bit/total_bits/shorts. |
| PlayerSlotShorts | PlayerTableShorts retains all ten fields: tete/deux/repr/q64/f10/f14/f6/f8/f7/f1. f6 remains signed. |

Inventory publication intentionally follows native plausibility and whole-film
ammo rejection. InventoryDeltaStream.components separately retains decoded raw
component fields and their bit ranges, including values rejected from published
records. The fixed four-element grenade output follows the native acceptance
rule; rejected variable counts remain in those component reads.

This audit found another ordinal omission in inventory outputs. Loaded framed
sources now provide packet_index; caller-supplied unframed component payloads
retain None. The captured comparison checks the exported ordinal directly.
The existing independent 16-case inventory oracle is also exercised inside
keyframe-plus-delta envelopes: positive component values, refusal counters and
publication remain identical, with ordinal 1 distinct from byte offset 33.
The focused four-film captured comparison passed: all 8,636 inventory records,
exported ordinals and counters match the pinned reference (89.50s).

CamoRead maps to BipedCamoState (source, slot, quantum, packet_index), and
GrappleRead retains source, slot, heavy, position_quantized and packet_index.
Loaded scans now populate ordinals for these and related ability ranks, impulse
and charge outputs; unframed caller-supplied inputs retain None. BipedChannelRead
also preserves the ordinal alongside its component bit ranges. The 4,096-case
ability fixture and independent channel-walk fixture now decode their reference
payloads after a keyframe and assert ordinal 1 versus byte offset 33, including
portable roundtrips. The full Theater suite passed (462 tests, 42 ignored),
along with final Clippy and WASM checks. The focused captured comparison of
these channel exports passed (88.71s): 4401 camo readings, 33 ability ranks,
39 charges, 4 impulses and 26 grapple reads, plus 8636 inventory records,
with all fields and counters compared.

BipedCreationStats retains slots/truncated/anchors/accepted/shape_bad/
signature_mismatch/gate_closed and the full other-representation histogram;
most_common_other exposes OtherWord/Count. Standalone payload scanning leaves
slots unset (zero), as does the native payload-only fixture. The loaded range
entry point sets the inclusive range cardinality. Truncated remains zero because
the scan loop requires the entire prologue before recognizing an anchor; native
readCreation has a defensive overflow branch that this loop cannot reach.
The 2048-case fixture now checks both counters explicitly and also exercises
loaded framing, selected-slot counts and serialization. Pickup comparisons check
exported other_type rather than inserting zero into their expected projection.
The native 0xC4 source gate can only yield types 8/9, so its OtherType remains zero.

Creation scanning now exposes scan_biped_creations_for_slots with native
FilmSlotBand membership. BipedCreationStream.slots preserves exact membership;
slot_band is its bounding range only. Loaded creation scans use the selected
native chunk prefix and tolerant packet framing, and FilmBipedCreation retains
packet_index. The range wrapper delegates to this native scan for nonempty bands.
The independent creation-source-v41 fixture covers 512 cases: 288 successes,
128 source failures, 96 empty-band failures, and 900 ordered creation records.
It compares every native field, counter and error category, including malformed
tails, duplicate/reordered/gapped metadata and sparse-band negative cases.

Automatic scan_biped_creations now checks the native source prefix before
requiring a discovered band. Film retains failures independently in
biped_creations_error; a missing band no longer looks like a successful empty
creation scan. Both map-based and explicit-encoding constructors retain the
error, remaining packets/replication, and portable roundtrips. The explicit
optional-range helper still treats caller-provided None as an empty legacy
selection; it is not the native automatic entry point.

The fixture now exercises explicit and automatic scans over 512 source cases.
Automatic results: 128 successes (320 records), 128 source failures and 256
missing-band failures. Explicit results retain the previous 288 successes
(900 records), 128 source failures and 96 empty-band failures. Every ordered
record, counter, slot membership and error category agrees with the pinned Go
parser. The native harness supplies valid ti35 or ti37 keyframes independently
of the explicit band. Full native Film fidelity and missing positive gameplay
mode evidence remain broader acceptance gates.

## types/grammar_monde.go

| Native declaration | Rust representation and field mapping |
| --- | --- |
| DeadState | march_facts::ObjectDeadState retains Mort/EnumA/EnumB/Val0c/Val0e/HasRef/GIDPresent/GlobalID/Val14/Val18/SrcTag0/SrcTag4c as mort/enum_a/enum_b/val_0c/val_0e/has_ref/gid_present/global_id/val14/val18/src_tag0/src_tag4c. Signed and all-ones absent sentinels remain intact. |
| ObjectDeath | ObjectDeath retains TimestampUS/Slot/Gen/TypeIndex/Dead/TailDesync as timestamp_us/slot/gen/type_index/dead/tail_desync. A broken record tail is distinguishable from a complete record. |
| VehicleOccupancy | VehicleOccupancy retains TimestampUS/Slot/Gen/Attached/ParentSlot/ParentGen/HasSeat/Seat as timestamp_us/slot/gen/attached/parent_slot/parent_gen/has_seat/seat. This is a recorded parent transition, not a complete inferred ride episode. |
| NavpointRadialRead | NavpointRadialRead maps Slot/TMS/Q/Chained to slot/time_ms/q/chained. Its clock remains manifest match milliseconds. |
| ProjectileSample | WorldObjectSample maps TimestampUS/Chunk/X/Y/Z/AtRest to timestamp_us/chunk/x/y/z/at_rest. |
| ProjectileTrack | WorldObjectTrack maps Slot/Gen/Pts to slot/generation/pts, preserving point order. The older Rust ProjectileTrack type is a different legacy observation and is not this mapping. |
| TranslocatorTeleport | FilmTranslocatorEvent.source retains TimestampUS; event.slot retains Slot; event.positions() returns the From/To pair only when valid, corresponding to HasPositions. event.from/to additionally retain quantized coordinates, regions and bit ranges, including partially read events. |
| VehicleEvent | VehicleEvent retains all thirteen native fields directly under snake_case names, including packet_index, occupant_sonde, vehicle_gen, vehicle_slot_valid and seat_valid. |

These declarations are represented, but this is not a claim that every producer
has complete coverage or that the assembled Film already has structural 1:1
fidelity. march_facts::tests::native_record_harvest compares the recorded death
and occupancy projections. Translocator fixtures compare paired position
validity, exact coordinate values and raw source bits. World-object and vehicle
fixtures cover their producer behavior separately. Retained dead-state runtime
handles must not be relabeled as weapon catalog identifiers by this declaration
mapping. Replay projections may reduce dimensionality or sample rate; those
projections do not replace the native WorldObjectTrack source data.

## Kill-source facade: result data and returned profile comparison

Audited against internal/facts/killsource/kill.go, assist.go and options.go at the
pinned commit. This closes declaration mapping questions, not whole-decoder or
arbitrary manually constructed Go-value equivalence.

| Native declaration/fields | Rust representation |
| --- | --- |
| Kill.TimeMS, Victim, Feed.Killer, Feed.Present | AttributedFilmKill.time_ms, victim, killer, feed_present. Credit stays separate from Source. |
| Kill.Source, Diverges, Read, Assist, KillerDamage, AssistDamage | AttributedFilmKill.source, diverges, read, assist, killer_damage, assist_damage. Rust also retains native-private packet identity as packet: Option<KillPacketIdentity>. |
| FeedTruth | AttributedFilmKill.killer and feed_present; no loss from flattening the two fields. |
| DamageShare.Pct, Known | KillDamageShare.pct, known. Unknown does not mean zero; values are not capped at 100. |
| Path | KillReadPath::Walk/Scan serialize as marche/scan. |
| Origin | KillSourceProvenance.origin retains native strings, including credit-concordant, source-victime, bot, tueur-bot and sans-revendication. |
| Provenance.Path, Origin, Multiplicity | KillSourceProvenance.path, origin, multiplicity. Display implements native String formatting; native_kill_provenance_display checks both paths, known/opaque origins and multiplicity independence. See kill-provenance-v41.md for domain limits. |
| PathStats.Population, Matched, Published, Ratio | KillPathStats.population, matched, published, ratio(). Nonnegative produced counters are in scope; arbitrary negative Go struct literals are not covered. |
| Coverage | KillSourceCoverage maps all nine fields to snake_case; real_pairs is signed and preserves subtraction outcomes. Bot populations stay separate from covered. |
| Stats.Walk, Scan, SelfWalk, SelfScan, Bot, BotKiller, Unclaimed | FilmKillSourceResult.attribution.{walk,scan,self_walk,self_scan,bot,bot_killer,unclaimed_stats}. |
| Stats.Redundant, NoBit, Agree, Disagree, MultiCandidate | FilmKillSourceResult.attribution.{redundant,no_bit,agree,disagree,multi_candidate}. |
| Stats.PacketsWithEvents, PacketsLocated, Assist, Couples, Appariement | FilmKillSourceResult.{packets_with_events,packets_located,assist_stats,pair_stats,attribution.matching}. |
| Result.Kills, UnclaimedDeaths, Coverage, Health, Stats, Roster | FilmKillSourceResult.attribution.{kills,unclaimed}, coverage, health, distributed stats above, roster. |
| Result.Calibration, ProfilCalibre | KillCalibration.to_string(), profile: KillWalkProfile, optional position_observer_context for native observer ranges/quantum, and optional reader_policy for fixed traversal/reader metadata. |
| Result.BijectionMargin, BijectionDetermined, Probe | FilmKillSourceResult.bijection_margin, bijection_determined, probe. Publication still uses both health and bijection gates. |
| Options | KillDecodeOptions carries Bots, SelfSource, MultiplicityMax, StrongTagRequired, BijectionRestarts and Views; Carte is the separate map argument to decode_film_kill_sources. Defaults match native true/true/2/true/40/8. Zero integer options normalize to defaults; Rust usize inputs do not represent negative native options. |

The kill_decode::tests::assert_native_result compares the complete returned result,
including every published kill, assistant/damage share, unclaimed death,
coverage/health value, pass statistic, public roster field, calibration text and
publication decision. ProfilCalibre is compared separately as a whole native-shaped
object reconstructed from retained Rust values, rather than a subset of fields.
This checks movement ranges/quantum, traversal/world descriptors, frame dimensions,
MPP widths, grammar/body flags, region mapping, calibrated/stub maps, and the five
remaining fixed policy values retained in KillCalibration.reader_policy.

Only float32 JSON spelling and native null versus empty override maps are
normalized. The comparison uses no expected-fixture constants to populate the
Rust profile. Internal assertions verify matching axis descriptors and region
maps, and the complete FilmKillSourceResult roundtrips through JSON. Old exports
without reader_policy or position_observer_context retain None.

The captured test uses Bandit and Oddball films and is opt-in. Focused native
fixtures cover assists, feed pairs, matching, hybrid publication, roster pinning
and health separately. The reader-policy fixture extension preserves every
previous captured kill-walk field. Current validation results are recorded at
the top of PORT_STATUS.md. This closes returned-profile field coverage only
when that complete comparison passes; arbitrary mutable profile APIs, negative
Go struct literals and whole-decoder corpus coverage remain separate gates.

## Weapon-hit records, histogram and distance callback

Audited against internal/grammar/weapon_hits.go and
weapon_hit_distance_resolver.go at the pinned commit.

| Native declaration/fields | Rust representation |
| --- | --- |
| WeaponDamage.TimestampUS, VictimIdx, ResponsibleIdx, Source, HasSource, Negative, MagClear, MagRaw | WeaponDamage in weapon_hits.rs retains every field and the native JSON names. Missing references remain signed -1; source presence and healing stay explicit. WeaponDamageRead additionally retains the secondary magnitude, body victim, end bit and synthetic padding length. |
| WeaponHitStats.FilmIndex, WeaponID, ShotsPaired, Hits, DistBuckets | WeaponHitStats retains all fields, including seven ordered distance counters. FilmWeaponHits.stats retains the returned sorted vector. |
| WeaponHitBucketCount | weapon_hit_bucket_count(), derived from WEAPON_HIT_DISTANCE_EDGES.len() + 1. The pinned default has seven buckets. |
| WeaponHitPairWindowUS | WEAPON_HIT_PAIR_WINDOW_US = 1_000_000. This is an association window, not an exact recorded action duration. |
| WeaponHitDistanceFunc | WeaponHitDistanceFn borrows WeaponDamage and returns Option<f64>; None corresponds to native ok=false and still counts the hit. Native receives the damage by value. Rust callers can use interior mutability for stateful callbacks. |
| PairWeaponHits | pair_weapon_hits retains native nearest-record selection and callback order, including the earlier record on equal-distance ties and repeated use of a damage for multiple shots. |

The 1024-case native pairing oracle now captures the full ordered WeaponDamage
arguments passed to the actual distance callback, in addition to both aggregate
outputs and the bucket count. This catches selected-record differences hidden by
equal histogram totals. It contains 22953 calls, including 10076 healing records,
and 205 cases with no calls. All previous fixture fields are unchanged.
The 2048-case event-reader oracle separately compares all decoded damage and shot
fields, secondary values, body victim and end cursor, including padded inputs.
Validation results are tracked at the top of PORT_STATUS.md.

Film::retain_weapon_hits stores shots, complete damage reads, landing/distance
bases, tracks, distance errors and stats in Film.weapon_hits. These are opt-in
existing APIs, not the proposed resolved replay architecture. Source-adapter
coverage and positive captured complete-pipeline evidence remain separate gates.
Rust uses the fixed native default distance edges; arbitrary mutation of Go's
exported WeaponHitDistanceEdges slice is not supported by this API mapping.
No inferred hit or distance should be called a directly recorded fact.

## Biped creation and position facade data

BipedCreation's native Slot/Generation/ParticipantIndex/Version/Representation
map to BipedCreation fields; BitPos maps to start_bit, and Chunk/PacketIndex/
TimestampUS live in FilmBipedCreation.source plus packet_index. HasIndex is true
for every published creation: both native and Rust reject the closed participant
gate rather than publishing index zero. The two-bit generation is retained;
life_key uses the same slot | generation << 16 expression. This declaration is
not a complete NEW body. The existing 512-case loaded creation oracle compares
automatic and explicit scans, ordered fields, counters, memberships and failures.

Native BipedPosition maps to BipedPositionCandidate.source and .record for
world-space scans, and SourceQuantizedPosition/SourceWorldPosition for loaded
explicit-layout scans. The latter already retained native PacketIndex. The
candidate-based scans previously dropped it; they now retain optional
packet_index, counting every packet type within each chunk before selecting
delta packets. Old exports and synthetic caller-created candidates retain None.
Film.biped_positions carries this stream into portable exports.

Slot/TimestampUS/Chunk/Q map to slot/source/quantized; HasWorld is represented
by the choice between world and quantized record types. World coordinates are
never fabricated in QuantizedBipedPositionRecord. Companions retain velocity,
forward/chassis orientation, both aim vectors and flags, and body/shield state;
compact_component_mask exposes MaskBits/MaskOver from the retained complete mask.
The mask projection is available independently of capture options, so consumers
must account for the native CaptureDirs publication gate. Record start, vector
start/end, generation and rejected position candidates are retained additionally.

The record-mask oracle now retains complete native accepted records, including
PacketIndex. Existing fixture data remains unchanged. Rust compares ordinals,
timestamps, slots and quanta, and roundtrips the candidate stream with explicit
legacy missing-field checks. A prefixed keyframe regression compares both
candidate scan paths and verifies the ordinal shifts by one while position
fields remain identical. This checks source attribution, not full Film/source
losslessness or the deferred architecture.

## Fire event and kill-table facade data

FireEvent in internal/grammar/fire_events.go maps field-for-field to FilmFireEvent:
Chunk, PacketIndex, TimestampUS, Variant, FilmIndex, ShooterIndex5, WeaponID,
Flags, HasAim and Aim. Both shooter widths are retained; the legacy four-bit
index must not replace the five-bit index in identity joins. HasAim remains
separate from a zero direction. Film.fire_events retains the ordered scan output.
The direct oracle compares all fields on 23171 heads and the modal aim position;
48 loaded-source cases compare source fields and admission. Direct reads check
length only; the loaded scanner selects long fire events. This does not identify
a hit or establish the short variant's meaning.

FilmTable in internal/facts/killsource/film_table.go maps to KillFilmTable:
Seats, Build, Occupied, Vacant, InterleavedVacant and Refusal are all retained.
The private native slots vector corresponds to read_table's second return value,
which feeds XUID motif resolution. read() matches Lue(): no refusal and at least
one named seat. The six refusal values are preserved literally: empty (read),
sans_registre, sans_section, build_inconnu, tronque and table_introuvable.
Refusal and an empty successful table are therefore not conflated.

FilmTablePinning maps all 20 native fields to KillTablePinning in snake_case:
Refusal/Build, Seats/Pinned/AddedNames/BotConflict/DuplicateName/OutOfRange,
Inferred/FreeNames, MotifPinned/Agree/Contradict/Duplicate, MotifReadings/
Disagreements/Absent, and Agree/Contradict/Silent. The motif-prefixed counters
remain distinct from the final vote counters. assignment_unique follows
AffectationUnique's zero-free-slot or one-slot/at-most-one-name rule.
The existing 1024-case roster oracle compares the full published pinning object.

The new 56-case native readFilmTable oracle checks every table field, read state
and every private PlayerSlot field through the actual source reader. It covers
first-buffer selection with absent/reordered/duplicate/negative chunk numbers,
empty input, missing identity, unknown build, truncated identity/table input,
and table-not-found. Six successful captured-source cases retain 48 total slots.
The original 49 refusal cases are unchanged. A versioned, hashed copy of the
Bandit bootstrap provides positive evidence independent of Rust. Test JSON
roundtrips include both the table and raw slots; native slot key spelling is
normalized only for case/underscores. Source selection follows first buffer,
not numbered chunk zero, as required by this native reader.

These are declaration and reader contracts. They do not promote whole-decoder
parity or claim that internal slot records are a native public result field.

## Damage-category catalog and health facade

Audited against internal/facts/killsource/{kill,label}.go,
damagetag/damagetag.go and internal/grammar/killhealth.go at the pinned commit.

| Native declaration | Rust representation and retained meaning |
| --- | --- |
| CatalogueProvenance | pinned_kill_damage_catalog().provenance retains IDsDate/IDsCount/LabelsDate/LabelCount as ids_date/ids_count/labels_date/label_count. The two dates and counts remain independent. |
| CatalogueSize | pinned_kill_damage_catalog().ids.len(), the ID-set cardinality, not the number of publishable labels. |
| Category | KillSourceTruth.category retains the signed numeric value; kill_damage_category_name implements Name/String including ?<n> outside the enum. Recorded categories fit this i32 field; arbitrary Go int values beyond i32 are not represented. |
| CategoryNone / Headshot / HeadshotMultiplier / SilentMelee / CollisionDamage / AttachedDamage | Values 0/1/2/3/4/5 respectively, retained as raw category and named through kill_damage_category_name. The same function covers WeakSpot=6, ChainedProjectile=7, SweetHeat=8 and VehicleTransferDamage=9. |
| KillSourceHealth | All fields map to snake_case, except UnexplainedBotIdx -> unexplained_bot. Candidates/Published, three unexplained counters, OutOfRoster, both out-of-catalog counters, DeathsReal and DeathsCovered remain separate. DeathsReal is signed. |
| UnexplainedWarnRatio | KILL_SOURCE_UNEXPLAINED_WARN_RATIO = 0.180; verdict uses strict greater-than. |
| UnexplainedAlertRatio | KILL_SOURCE_UNEXPLAINED_ALERT_RATIO = 0.360; alerts use strict greater-than. |
| CoverageWarnRatio | KILL_SOURCE_COVERAGE_WARN_RATIO = 1.00; verdict uses strict less-than. |

The catalog oracle compares 513 text-table cases, including the embedded tables,
complete labels, independent dates, duplicate handling, unknown classes/statuses,
publication decisions and parse failures. The kill-source oracle compares all
468 IDs and provenance, plus 6608 source-label/category combinations: categories
-2 through 11, including 28 AMBIGU outputs. Ambiguous labels retain their metadata
without being published as known source names. No category is converted into an
assertion about a physical action.

The 1024-case native matching/health oracle compares every health output method:
ratios, totals, alert/degradation messages, verdict, metric names/values and
line-by-line publication gate. It includes 26 exact-equality cases at each
unexplained threshold and 30 cases with nonpositive real-death counts. Thresholds
are now named exported constants used by the same methods. OutOfRoster remains
outside the unexplained numerator; it has its own degradation. Scan catalog
misses stay distinct from walk catalog misses. Broader producer coverage and
arbitrary negative manually constructed native counters are separate from these
produced v41-data contracts.

## Objective statistics records, series and round identities

Audited against types/objectives.go and internal/facts/objectives/{score,
slotidentity_rounds,statborg,rosterfit}.go at the pinned commit.

| Native declaration | Rust representation |
| --- | --- |
| DeathInstant.XUID/TimeMS | StatborgDeathInstant.xuid/time_ms; identity remains a decimal string and timestamp remains signed milliseconds. |
| PlayerLine.XUID/Kills/Deaths/Assists | StatborgPlayerLine retains every caller-supplied field and signed counter. It is match-sheet evidence, not a decoded film event. |
| ScorePoint.TimeMS/Slot/Value | StatborgScorePoint.time_ms/slot/value, all signed; ordered vectors preserve the returned series. |
| StatValue.A/B/C/D/HasC/HasD | StatborgValue retains all six fields. Optional C/D channels remain distinct from present zeroes. |
| StatRecord.TimeMS/Slot/Round/Comps | StatborgRecord retains time, entity slot, recorded round and complete component map. Film.statborg retains the decoder stream; derived series do not replace these records. |
| StatComponent.Comp/SideB/Strict/Unitary | StatborgComponent.component/side_b/strict/unitary. Unitary step bounds apply to action counters; cadence/score series keep their separate policy. |
| SeriesByRound / SeriesTotal | StatborgComponent.series_by_round / series_total delegate to the corresponding Rust series functions with all four policies. series_total_with_diagnostics additionally retains emitted diagnostics. |
| RealRounds | resolve_statborg_rounds(records).real retains the native RealSet; written rounds, contradicted rounds/record count and decreed fallback remain available separately in the decision. |
| SlotIdentityByDeaths | statborg_identity_by_deaths returns the signed-slot-to-XUID map inferred from death-counter timings. This is derived identity evidence. |
| RoundIdentity | StatborgRoundIdentity.publication retains native byRound and origins maps; starts retains ordered round/start_ms pairs. Empty origin maps represent native nil provenance. |
| ResolveRoundIdentity | resolve_statborg_round_identity retains the death-based origin per assigned slot and consensus starts with the native minimum-time fallback. at resolves by time; completed_by_lines retains the distinct sheet-derived origins. |
| StatPlayerSlots | STATBORG_PLAYER_SLOTS = 8, corresponding to player entity slots 10..24 at stride two. |
| RosterFitsStatborg | roster_fits_statborg accepts signed seat counts and implements n <= 8 exactly. Negative counts pass as in native; this capacity query is not general input validation. No new extraction gate is introduced. |

The component oracle compares all seven default descriptors, custom descriptors,
complete per-round/total queries over 128 generated record sets and 64 score-domain
cases. It now also compares the native slot count and eleven capacity queries,
including 7/8/9, zero, negative values and signed 64-bit extremes. Every previous
fixture field is unchanged. Independent round-admission, identity, completion,
series and source-decoder fixtures provide the complementary evidence; current
validation is recorded in PORT_STATUS.md. These mappings do not establish that
all observable gameplay actions are available, nor replace the deferred semantic
golden-film suite.

## Objective families and canonical identifiers

Native CountObjectiveFamily is now exposed as count_objective_family over the
ObjectiveStatName trait. Both StatborgNamedEvent and StatborgIdentifiedEvent
implement the native StatName contract; identity resolution is not a prerequisite
for counting. The three existing internal family-count sites now share this
helper. is_objective_family_stat matches the native case-sensitive prefixes
flag_, zone_, hill_, skull_, vip_ and bomb_, including the underscore.

ObjectiveTypeOf maps to replay_objective_type. Native simple Unicode lowercase
and keyword precedence are retained: flag/CTF first, then zones, hill, skull,
and bomb. VIP is a supported family but is deliberately not recognized by this
specific variant-name classifier; its selection is separate in the reference.
The native ObjectiveTypeBomb/Flag/Hill/Skull/Zone constants correspond to the
unchanged bomb/flag/hill/skull/zone strings retained in outputs and catalog keys.

Other canonical identifiers remain literal output data: EventTypeCapture is
capture and RoleScorer is scorer in objective_extract; StatFlagCaptures,
StatZoneCaptures and StatZoneSecures are flag_captures, zone_captures and
zone_secures in the named-stat table. IdentitySourceTotals is totals in
StatborgIdentityStats.source; OriginRoundResidue is residu_de_manche in the
round identity's per-slot origins map. These are distinct provenance/category
values, not interchangeable descriptions of certainty.

CaptureBurstTimes maps to scan_source_capture_bursts (and loaded-chunk
scan_capture_bursts). The first FRAME supplies the per-chunk clock anchor and
is skipped; subsequent six-tier bursts supply sorted millisecond timestamps.
These are corroborating capture signals, not invented capture actors. Native
footer/source extraction oracles compare both adapters and their capture times.

The extended objective-action oracle compares generic counts on named and
identified forms in all 1024 cases (9899 family events), alongside the existing
ordered actions, coverage, missing-track counts, twelve prefix cases and fifteen
mode-classification cases. All previous fixture values are unchanged. Broader
complete VIP/bomb captured coverage and independently annotated action validation
remain open gates.

## Weaponv3 identity resolver facade

KnownWeaponHigh32 maps to v41_weapon_families(), the complete pinned
high-word-to-canonical-name map (35 entries). It is derived from the reference
weapon/fusion tables. Rust exposes an immutable catalog; arbitrary mutation of
the native exported map is not supported. canonical_weapon_id retains the
separate suffix-only acceptance route, so accepted but unnamed families remain
possible. canonical_weapon_name returns an empty string for an unnamed family.

PIBits maps to PLAYER_INDEX_BITS = 5, now used by resolve_player_indices.
ResolveXuidToPI maps to resolve_player_indices: each XUID is byte-swapped to its
64-bit pattern, the first bit-aligned occurrence wins, and the preceding five
bits identify the participant. Missing patterns are absent from the returned
map. Reads before the start of the chunk contribute zero bits. This heuristic
is not a sequential player-table decode.

ResolveBest maps to resolve_best_player_indices, preserving input chunk order
and the first assignment for each XUID even if later chunks disagree. This is
intentionally different from scan_player_indices, which refuses a conflicting
identity. The decimal-string adapter preserves the native trimmed-digit parser,
wrapping arithmetic and last-original-spelling key choice; nonnumeric/bot keys
are excluded from that adapter.

The native weapon-helper oracle now compares the whole 35-entry family catalog
and width, and adds 512 deliberate cross-chunk conflicts with duplicate roster
XUIDs. Every case verifies two different raw assignments and native first-wins
merge output. Existing helper fields are unchanged, including 2048 weapon-ID
queries, numeric/string identity results and timestamp estimates. The separate
player-index scanner fixture checks its disagreement-rejection policy. These
checks do not upgrade heuristic identity evidence into a directly recorded
player relationship or establish arbitrary source/profile API equivalence.

## Header, player-table and highlight facade

FilmMajorVersionFromHeader is now exposed as film_major_version_from_header:
the first four bytes are read as little-endian u32, with None for short input.
Native FilmMajorVersionUnknown=0 is paired with ok=false; Rust represents that
pair as None, keeping a recorded Some(0) distinct. A caller needing the native
pair can use (value.unwrap_or(0), value.is_some()). This metadata reader does
not decompress bytes or assert decoder support. decode_v41_film_key now shares
it and still rejects every readable non-v41 major version.

The native film-key oracle adds 27 raw-header cases (lengths 0..8 and words
0/41/u32::MAX). Every previous key/identity fixture value remains unchanged.
Rust compares raw value and validity and separately verifies that reading other
major numbers does not enable their decoding.

ReadPlayerTable maps to decode_player_table. PlayerTable retains the slots,
full PlayerTableReport and optional error. All 17 report fields are represented:
build/perso_bytes/profile_delta_bits, film_delta_bits/film_delta_gaps/
calibration_agrees, the four gap counters, occupied/vacant/head_vacant,
interleaved_vacant/first_record_bit and both candidate counts. Profile geometry
and measured calibration remain separate, and diagnostics survive a failed
attempt while partial slots are withheld. Film.player_table retains the whole
result. Native error categories map to UnknownBuild/Truncated/NotFound; arbitrary
native error wording is not reproduced. The existing 289-case player-table
oracle compares every slot/report field and error category.

ParseHighlightEvents maps to parse_highlight_events for explicit major 41.
NativeHighlightEvent retains XUID, Gamertag, EventType, TypeHint, IsMedal,
TimeMS and MedalType. Plaintext and zlib paths preserve native decompression
failure handling and scan ordering; unknown event layouts are skipped by this
API as in native. The source-attributed summary decoder is a separate API and
retains unknown summary records. The native 9323-case highlight oracle compares
all returned event fields and error states. Older-major and unreadable-major
fallback parsing are intentionally not enabled. HighlightProfileFromHeader's
separate returned-profile contract remains to be audited.

## Highlight profile header contract

`HighlightProfileFromHeader` maps to `v41_highlight_profile_from_header`. Native
MajorVersion/Lue map to FilmHighlightProfile.major_version: None preserves
unreadable-header provenance, including the distinction from a recorded zero.
Implantation and GamertagOffsetBytes map directly to implantation and
gamertag_offset_bytes. For v41 and missing headers the native layout is
gamertag_en_tete with offset zero. Missing-header layout is fallback metadata,
not evidence of v41 compatibility. Readable non-v41 headers are rejected.
The explicit-version highlight scanner does not fabricate header provenance.
The film-key oracle retains all previous data and adds all four native profile
fields across its 27 raw-header cases; portable metadata is roundtripped.

## Facts revision and named fallback facade constants

Native decfilm.Rev aliases facts.Rev and maps to NATIVE_FACTS_REVISION, currently
killsource-2026-09-22.2. Replay decoder provenance uses the same constant. This
is the pinned algorithm identity, not a claim that Rust parity is complete.
NomGardeEquipementNegatifAZero and NomGestePremiereVieDuSlot map respectively to
NEGATIVE_EQUIPMENT_KEPT_FALLBACK and FIRST_SLOT_LIFE_FALLBACK. Existing usage
clamp and first-life attribution paths use these exact public identifiers.
The 577-case native coverage fixture and 1036-case usage fixture retain the
independent output expectations; 162 and 558 usage cases include the respective
fallback identifiers. No values, native fixtures or fallback policies changed.

## I0Layout and raw region index widths

I0Layout retains native GateBits/AxisW/Region as signed gate_bits, three unsigned
axis_widths and region. Native Valid, TotalBits and String map to valid,
total_bits and Display. axis_offset_signed now preserves the native signed
input behavior: every nonpositive input returns the gate offset. Zero through
three retain native wrapping sums; above three returns None rather than the
native array-bounds panic. The unsigned checked axis_offset API remains unchanged.
The 512-layout fixture includes 4608 independently executed signed-axis outcomes,
including signed extremes, while preserving all previous fixture fields.

LargeurIndexDePlage maps to position_region_index_width. It uses the raw region
count: one maps to one bit, negatives clamp to zero, other counts cast through
u32 and the result caps at 26. The native precision fixture retains 2048 original
rows and adds 13 explicit count boundaries, including 1 and u32 wrap cases.
Neither helper establishes a map identity or enables older film decoding.

## Caller-tagged source packets

Native Paquets maps to walk_tagged_film_packets. TaggedFilmPacket borrows each
payload and retains the caller's full signed 64-bit chunk tag independently of
FilmSource positional indexes. Native Index, Type and TS map to packet_index,
packet_type and timestamp_us. Rust additionally retains payload_offset and the
two otherwise opaque header bytes. A tag is not validated as a source index.
The native source oracle adds nine tags per original case, including both i64
extremes and values immediately outside i32, for 4608 calls / 10440 packets.
Original 512-case source expectations remain unchanged. Packet termination and
malformed-tail policy are unchanged; this adapter does not retain the trailing
unparsed region or establish whole-source portable Film fidelity.

## Kill-source facade identity constants and result records

BotSuffix maps to BOT_SUFFIX (" [bot]") and XUIDNamePrefix to XUID_NAME_PREFIX
("xuid:"). Roster publication and missing-gamertag feed construction now share
these public constants. Existing native oracles include 544 bot-bearing roster
rows out of 1024 and 46 fallback-name feed rows out of 512. These labels are
publication conventions, not additional recorded identity fields.

ErrNoKillFeed maps to NO_KILL_FEED, the existing typed
DecodeError::Missing("kill-feed highlights"). load_kill_feed returns this when
no supplied chunk has a successfully decoded kill highlight. The French native
error text and Go error pointer identity are not reproduced; Rust callers can
match the typed value. This mapping does not establish missing-feed corpus
coverage or change the explicit v41 version gate.

Assist maps to KillAssist: Name/Index/Rejected/Known/Extra map to
name/index/rejected/known/extra. Known=false remains unknown, not an assertion
that no assistant exists. Extra counts distinct additional assistants among
attached kill-event records only, not all possible damage contributors. The
1024-case native producer oracle checks every target field and aggregate,
including positive multi-assistant, window-fallback and field-disagreement cases.
Rust index is i32 and extra is usize; arbitrary signed native int inputs outside
those domains are not supported by this mapping.

CoupleStats maps to KillPairStats. MemeInstant/Lus/Recolles/Perdus map to
same_instant/read/neighboring_fallback/lost; VictimesBotLues/Muet/Ambigu map to
bot_victims_read/silent/ambiguous; Accord/Contradiction map to agree/contradict.
All nine counters are compared in the 2048-case native pair producer oracle,
which requires positive read, bot, ambiguous and fallback populations. These are
nonnegative producer counts represented by usize, not arbitrary signed native
struct inputs. Reference agreement does not independently validate inferred
neighboring pairs as actual recorded actions.

## Default and kill-starting scan profiles

The pinned native default and ProfilDeDepart are captured in
starting-profile-v41.json. Their only difference is GenerationStricte (false
in the default, true in the kill-starting profile). The Rust starting state is
projected from KillCalibration into all native Mouvement/Cadre/MPP/Grammaire
fields for comparison. Float dequantization values compare at native f32
precision; null empty width maps represent native default maps only.

The Rust representation is distributed across FrameEncoding, PositionEncoding,
KillCalibrationPolicy and PositionCaptureEncoding. This verifies starting values,
not a unified mutable ProfilDeBalayage API or arbitrary context replacement.
kill_replay_starting_profile currently returns only KillWalkProfile; full retained
policy/observer metadata is available on KillCalibration, not that convenience
return. Profile type and constructor declarations therefore remain partial.

## NewWorld and kill matching statistics

NewWorld maps to FilmWorld::default for world state. The native fixture now
captures constructors with nil and non-nil registry inputs before mutations:
empty slots/counters, zero current chunk/view, unknown keyframe namespace and no
anticipated table. All previous mutation, rollback and datum expectations are
unchanged. Rust passes FilmRegistry separately to walkers rather than retaining
a mutable registry pointer in FilmWorld. Native anticipationDite is a one-time
logging latch, with no Rust equivalent. These differences remain explicit.

ApparStats maps to KillMatchingStats. Identite/Fenetre/BotFenetre map to
identity/window/bot_window, NonRevendiqueeFenetre to unclaimed_window and
CouplesSansIdentite to pairs_without_identity. The first four classify published
matching paths; the fifth is a separate diagnostic and must not be included in
the publication sum. All five are compared in the full hybrid output of the
1024-case native kill matching oracle. Counts are nonnegative usize producer
values, not arbitrary signed Go struct inputs. Temporal and bot fallbacks remain
inferences; these counters do not turn inferred pairings into recorded facts.

## Death-context analysis

`ContextesDesMorts` maps to `build_replay_death_contexts`. Its inputs are decoded
world positions, the existing identity registry, external journal deaths and
external teams. `MortDuJournal` maps to `ReplayJournalDeath`; `ContexteMort`
maps to `ReplayDeathContext`, preserving victim, match-clock timestamp, optional
horizontal nearest distance, visible/waiting/out-of-sight counts and total.
Visibility and distance constants map to DEATH_CONTEXT_VISIBILITY_MS (1000)
and DEATH_CONTEXT_DISTANCE_SCALE (100).

The publication gate refuses index disagreements and an absent bridge, but does
not refuse recycled slots. Positions belong only to a named life covering the
wire time, in original life order. The general identity lookup's flattened-slot
fallback is deliberately not used. Named-life bounds and the 1000ms freshness
window are inclusive; the waiting test requires a strictly earlier death and
no later observation through the query time. Victims need a recent world point
and an external team, but do not require the teammate vitality gate. Z does not
contribute to horizontal distance. Results preserve external journal order.

The pinned 512-case oracle compares 5472 complete outputs, all three state
populations, optional/rounded distances and 56 suppressed cases; serialization
also roundtrips. These are derived analytical outputs, not wire-recorded actions.
They are exposed separately from the native replay document, matching the
reference's separate entry point. This ports existing reference functionality;
it does not start the queued resolved-model architecture or provide independent
captured-action validation. See fixtures/death-context-v41.md for provenance.

## Kill placement and opening proxy

`BuildKillPositions`, `BuildKillOpenings` and `ShiftKillRefs` map to
build_replay_kill_positions, build_replay_kill_openings and
shift_replay_kill_references. ReplayKillReference already retained all KillRef
fields. ReplayKillPosition retains that reference and optional killer/victim
world vectors; ReplayKillPositionReport retains all seven report counters.
ReplayKillPositions bundles the native result pair without dropping either part.

Placement shares the existing shot nearest-sample helper, including earlier-side
tie preference. It selects only exactly one world-bearing slot within 120ms per
identity, using the native temporal identity lookup with its fallback policy.
This differs intentionally from death-context attribution, which requires a
covering named life. The no_bridge diagnostic counts kills, not missing sides,
and overlaps published/dropped categories. Initial empty-input refusal only
increments dropped. Signed time shifts retain native wrapping arithmetic.

Opening publication samples 1500ms earlier but returns the original kill time.
It recounts categories after rejecting sides that do not span both query times
in the same replication life. Earlier placement drops remain in the report.
This is a native derived proxy, never a directly recorded first-damage event.
The 512-case independent Go oracle compares all ordered fields/counters for
7168 references, including 512 rejected opening sides. See
fixtures/kill-positions-v41.md for coverage and limits. These separate native
entry points do not alter the replay document or start the deferred refactor.

## Aim and basic track publication inventory reconciliation

The pending labels for replay/build_aim.go, document_aim.go and
document_tracks.go were stale. Source inspection maps headingForJSON and
pitchForJSON to replay_point's tenth-degree rounding: nonpositive rounded
headings publish 360, and pitch normalizes signed zero. zoomHoldUS is
REPLAY_ZOOM_HOLD_US. Native Point's ten fields map to ReplayPoint, including
optional shield/health values that retain measured zero separately from absence.
Loadout, Shot, Bounds and Track map to ReplayLoadout, ReplayShot, ReplayBounds
and ReplayTrack, retaining all fields and native JSON names/omission rules.

The independent track generator calls native decimateTracks on 1024 cases,
78336 input samples and 42302 published points. It includes absent aim/vitality,
health/shield values outside [0,1], multiple samples per frame, supplied lives
and replication-derived fallback lives. Native HealthAt/ShieldAt clamp before
fractionForJSON; replay_point's clamp is therefore equivalent at this production
boundary, not a missing raw-value retention fix. Raw companions remain separate.
The full-document schema oracle compares native zero/populated records through
Rust serialization. These checks establish the tested production publication
and schema contracts, not arbitrary IEEE special-value JSON support or the
future lossless Film hierarchy. No production code changed in this audit.

## Named-life export

IdentityRegistry.ViesNommees maps to IdentityRegistryOutput::named_lives.
VieNommee maps to NamedIdentityLife: XUID, DebutMS, FinMS, Cause and NomPar map
to xuid, start_ms, end_ms, cause and named_by. The native DeathsNamed gate maps
to owners.deaths_named; a nonempty roster or named life does not bypass it.
Anonymous lives are omitted. Each signed bound is divided by 1000 with
truncation toward zero before wrapping subtraction of the death-clock offset.
Stable start/XUID/end ordering preserves cause and naming provenance on ties.
The original registry lives are not mutated or replaced.

The native 512-case oracle compares 6396 exports and 128 gate-disabled cases,
including negative submillisecond bounds, offset overflow and exact-key ties.
Serialization retains every output field. This is an existing derived export,
not the queued canonical Film design or independent validation of inferred
lifetimes. See fixtures/named-life-export-v41.md.

## External roster-string adapter

RosterXUIDsOf maps to replay_roster_xuids. It accepts nonempty ASCII decimal
strings whose value fits u64 and is nonzero; signs, whitespace, other digit
scripts, numeric separators and overflow are rejected. Leading zeros are valid.
Input order and duplicates are retained. This is deliberately separate from the
weapon-index string adapter's trimmed/wrapping conversion and from the sorted
union built by replay_identity_roster. It does not authenticate an identity or
turn an external roster into film-recorded evidence. The pinned native 256-list
oracle compares the complete ordered output; see fixtures/roster-strings-v41.md.

## Statborg seat guard and round-residue inventory reconciliation

internal/facts/objectives/rosterfit.go maps to STATBORG_PLAYER_SLOTS and
roster_fits_statborg. The limit is eight seats, not eight roster rows; zero and
negative unknown counts pass. The native component oracle directly compares
RosterFitsStatborg across supplied seat counts. No identity is assigned by this
capacity predicate.

slotidentity_residue.go maps to StatborgRoundIdentity::completed_by_round_residue
and statborg_round_residue. The implementation copies the identity state before
completion, skips empty external totals and single-round inputs, excludes zero
KDA segments, requires mutually unique slot/player matches, and records the
native residu_de_manche provenance. It subtracts matching segments from all other
rounds and preserves unresolved candidates. The existing 1024-case native
residue oracle compares segments, residual KDA and the complete completed
identity, alongside elimination results. These are derived identities, not new
wire facts. Both source entries had stale pending labels; no code changed.

rounds_decision.go also has an existing all-field producer mapping:
resolve_statborg_rounds returns written/real/contradicted rounds, contradicted
record count and decreed status; the 1024-case round oracle compares the complete
decision and boundaries. Native RealSet is represented by collecting the public
real vector into a set rather than a dedicated method. Its source entry remains
partial to keep that API difference explicit; producer output is not missing.

### Round helper closure

The subsequent implementation adds StatborgRoundsDecision::real_set and
StatborgRoundBounds::outliers, plus STATBORG_OUTLIERS_NOMINAL_MAX (27). The
threshold is diagnostic; it does not alter exclusion policy. Outlier counting
shares excludes, including kept-segment exemptions. The native round oracle
was extended by calling RealSet and Outliers directly. All original fields in
all 1024 cases were checked unchanged before replacing the fixture. Added
expectations contain 6736 outliers across 485 positive cases. Rust now checks
those values, the constant and admitted-round sets alongside all previous
outputs. The earlier note about a missing RealSet method is superseded.

## Round identity query surface

StatborgRoundIdentity now provides from_flat, origin, at_round, resolved, rounds
and named_count for native FlatRoundIdentity/Origin/AtRound/Resolved/Rounds/
NamedCount. Empty round maps still establish Resolved, and NamedCount counts
stored entries including empty values. Origin reads its own map independently
of by_round. from_flat consumes an owned Rust map, always installs round zero,
and creates neither provenance nor starts; it does not reproduce mutable Go
map aliasing. Existing chronological at and completion algorithms are unchanged.
The independent 256-case oracle compares 67584 query tuples plus metadata and
flat construction, including empty and malformed-publication controls. See
fixtures/round-identity-queries-v41.md. This adds native queries, not stronger
identity evidence or the deferred replay architecture.

## Death-only source identity wrapper

Correction to the recent source audit: statborg_source_resolved_identity already
implemented native SlotIdentityResolved, including source-before-pass diagnostic
ordering and truncation. The missing wrapper was SlotIdentityFromDeaths only.
It is now statborg_source_identity_by_deaths, sharing source_pass and the existing
record-based death resolver. This preserves source diagnostics and truncation
without adding a totals comparison or scanning twice.

The four-case native source fixture now calls SlotIdentityFromDeaths directly;
all previous fixture fields were verified unchanged. Positive death increments,
missing deaths, a 151ms shifted feed, and equal-coverage conflicting totals are
compared separately from combined resolution. In the latter case the death-only
wrapper keeps death-player while the combined resolver prefers totals-player.
Complete wrapper output serialization is checked. This closes the named source
adapter gap; it does not establish independently observed player identities.

## Objective utility and weapon/translocation publication reconciliation

objectives/helpers.go uses language-level representations in Rust: intPtr is
Some for optional numeric fields, timeOrNeg is unwrap_or(-1) in objective event
ordering, formatXUID is unsigned decimal to_string, and abs uses wrapping_abs
for the native signed-minimum behavior. These helpers do not introduce separate
record fields. Existing objective-extraction and identity oracles exercise their
producer paths; no standalone mutable-pointer API is reproduced.

replay/document_weapon_changes.go maps to ReplayWeaponChange,
ReplayWeaponChangeCoverage and build_replay_weapon_changes. All five event
fields and seven counters are retained. Restated classification precedes the
origin check; there is no published-slot gate. Zero remains a real eight-digit
family, while all-ones is absent. spawnSetFrom's actual consumer is
weapon_changes.rs::classify: first input before any date, then last qualifying
input-order loadout, not a timestamp-sorted nearest choice. Unsupported arbitrary
integer enum values are not exposed by the Rust HeldWeaponChangeKind type.

replay/document_translocations.go maps to ReplayTranslocation and its coverage:
all six optional endpoint coordinates, frame/slot and five counters are retained.
Both endpoints are published together only when available; known zero remains
Some(0), coordinates round to centimeters, and input order is retained. The
native INFO logger additionally emits published-minus-positioned as sansPosition;
The follow-up now emits that native coverage record through tracing when the
Film document assembles this layer. The extended 1024-case oracle captures
actual native slog output and actual Rust tracing output, comparing level,
message and all six attributes, including empty and zero-step controls. Only
wall-clock log time and framework metadata are excluded; source fixture fields
were verified unchanged. See fixtures/replay-translocations-v41.md. This closes
this logger contract, not all replay logging or global cross-layer log order.

The existing independent publication oracles compare complete outputs; the last
full Theater run passed these tests. These are current native publication
contracts, not the deferred all-source Film export or independent action truth.

## Film player-table source projection

scan_replay_film_player_table now exposes native ScanFilmPlayerTable over the
existing FilmSource. It selects chunk number zero through source metadata,
falling back to loaded position zero when metadata is absent. Registry and
identity failures retain distinct native refusal categories; successful decoding
reuses PlayerTable and ReplayFilmPlayerTable::from_decoded rather than adding a
second slot parser. Unsupported complete major versions still return an error.

The 19-case pinned source oracle compares the complete result and Lue predicate:
three successful one-seat tables, all five refusal reasons, nonfirst metadata
selection, absent registry number and first duplicate-number selection. Every
result survives JSON roundtrip. The provenance file includes the captured input
hash and explicit mutations. Native source logging and process-global metrics
are not emitted by this adapter and remain open; film_player_table.go stays
partial. Existing Film assembly already projects decoded player tables and is
unchanged by this source convenience API.

## Schema chronicles

The complete document_chronicle.go and usage_summary_chronicle.go contain only
package replay plus blank lines and line comments. They are historical reference
documentation with no runtime declarations. Current schema/usage identities are
separate publication contracts; marking these two files reference-only does not
remove those requirements or establish parser completion.

## Scope and ability publication retention

zoom_state.go maps to ReplayScopeLookup and its periods. The 1024-case native
oracle checks 614400 lookup queries. Input events stay in supplied order; an
explicit exit closes at its exact timestamp even beyond hold/life bounds. A
later entry or unfinished period uses bounded_end with wrapping addition; both
query endpoints are inclusive. Native sort.Search behavior is preserved even
for nonmonotonic end times. Film.zoom_events feeds replay_from_film's lookup;
these are recorded zoom observations plus native hold policy, not new actions.

abilities.go maps to replay_abilities.rs: four read fields, four coverage fields,
palette identity/markers/labels/families, native rank-noise rejection and palette
classification. Both channels survive without deduplication; ordering is stable
by frame/slot/source. Ranks above 27 are discarded before the track filter.
Palette classification uses first matching markers and input palette order,
90 percent purity or 100 percent for fewer than ten reads. Label families are
copied from the selected palette. The native INFO records are not yet emitted.

Charges retain frame, slot, family, charge count and all eight coverage fields.
Impulses retain frame/slot/family, episode count, the same attribution fields
and all six scan counters. The shared rank resolver uses first qualifying life,
five-second tolerance and latest prior rank with last-input tie resolution.
Folding extends an impulse episode for gaps of at most one second from its last
reading. The 1024-case native ability fixture checks both complete outputs and
folded episodes, not only totals. replay_abilities_film and replay_document_film
retain these outputs and only publish coverage when the scan ran. Zero charges
remain data, while missing coverage remains distinguishable from zero reads.

Native logAbilityChargeCoverage/logAbilityImpulseCoverage and their not-scanned
WARN messages are absent in Rust; these entries remain partial for observations.
No field-retention gap was found on these inspected producer-to-document paths.
This is reference parity evidence, not the independently annotated semantic
golden suite or the deferred architecture.

## Equipment, pickup, pad and vehicle publication reconciliation

Equipment changes retain all seven event fields and twelve coverage fields.
The 1024-case native publication fixture checks the complete output. Spawned
announcements are excluded before origin filtering, while publication/taken/
spent counters are accumulated before the published-track filter. A filtered
list can therefore legitimately be shorter than coverage.published. Missing
rank is -1, recovered/gap flags survive, and the birthOfLives consumer in
equipment_recovery.rs takes each slot's minimum accepted position timestamp.
The follow-up now emits the native INFO coverage logger in document assembly,
immediately before the translocation log. An extension of the existing 1024-case
oracle captures actual native slog output and actual Rust tracing output; level,
message and all twelve fields compare, while prior fixture fields are unchanged.
Only wall-clock time and logging-framework metadata are excluded. See
fixtures/replay-equipment-publication-v41.md. Global cross-layer log order
remains a separate requirement.

Pickup publication retains eight event fields and fifteen coverage fields.
The 512-case oracle compares complete output, including origin metadata and
spawner-by-point-kind counts. Classes 0/1 use weapon catalogs, 2/3 use equipment
catalogs, and other classes have no family fallback. The temporal occupant is
optional; zero means unnamed. Only nonweapon pickups call the origin resolver.
Catalog IDs format as eight hexadecimal digits; XUIDs use unsigned decimal.
Film assembly retains these results in the document and coverage.

The four ground-weapon document types are represented by GroundWeaponPad,
GroundPadPresence, GroundPadPublishedCycle and GroundPadPickup. Every field is
retained, including nullable xuid versus omitted exact t, three presence bounds,
optional cycle and its five values. Existing 512-case pad/cycle tests and four
captured-derived layer comparisons cover the publication. Native optional zero
z omission is preserved; source position data is separately retained upstream.

VehicleTrack/Spawn/Sample/Ride/Aim/Coverage map to replay_vehicles.rs without
missing fields. Spawn heading, ride seat and end timestamp remain Option values,
so known zero does not become absence. Coverage is tallied after relay merging;
heading count uses nonzero heading, ride frame lengths are inclusive, overlaps
compare adjacent rides only, and unknown chassis retain their histogram and
neutral-marker fallback counts. End-state counting is consolidated in the Rust
tally, rather than omitted. Existing native fixtures compare 1024 track/coverage
cases and 512 combined publications, including cycles and fallbacks. The native
vehicle INFO and conditional WARN records remain absent and their source stays
partial. These mappings do not close broader captured producer or logging gates.

This audit changes inventory evidence only. The latest 493-test Theater run
already executed these named oracle tests; no new parser behavior is introduced.

## Bridge health confidence observation

coverage_bridge.go retains all 25 BridgeHealth fields through identity_health.rs,
including an optional measured zero death offset. The native identity-registry
fixture already compares complete health and verdicts. Final life repair updates
its five naming/unnamed counters separately. Layer verdict in replay_shots.rs
preserves zero-data precedence, accounting failure and the native 0.66 threshold.

The missing narrow-margin warning is now emitted by ReplayCoverage::new after
bridge_health, matching native buildCoverage. The accessor itself remains pure.
The 512-case native log fixture compares 225 warnings and 287 silent controls,
including unmatched clocks, no runner-up, exact two-to-one and one-below margins.
Level, message and all five fields come from actual captured logs; wall-clock
time and framework metadata are excluded. Provenance: fixtures/bridge-warning-v41.md.
This exposes alignment uncertainty without asserting additional recorded events.

## Roster seat declarations and ordinal fallback

RosterEntry's eight fields map directly to ReplayRosterEntry, preserving optional
team zero, bot identity, seat and seat-source fields. Seat assignment already
matches a 1024-case native oracle for both mutated roster and all ten counters.
Presence uses each identity's full envelope across lives, including gaps; named
bots use their bot-name key. Native equal-time sorting is retained. Ordinal
replacement requires departure strictly before arrival and a readable initial
film table. Concurrent occupancy uses end-plus-one boundaries; shared written
indices remain distinct from ordinal replacement. Document assembly publishes
apparies as repli_siege_du_remplacant_par_appariement_ordinal. These are existing
implementation mappings, not new roster inference or stronger identity evidence.

## Lifetime and closure source reconciliation

closures.go and closures_respawn.go map to identity_closures.rs. The native
1024-case fixture compares complete owners and closure report, designated lives,
respawn window, named-life results and slot-XUID extension. Shot deductions run
before respawn deductions over a copy of the caller's bridge. Four counters and
the per-slot life designation remain available; -1 marks conflicting designated
lives. Rust keeps claimant membership rather than unused per-claim frequency,
matching the native cardinality-based decisions. Claims are processed in native
sorted slot/XUID order. A shot closure requires a previous anchored body and a
strictly later terminal body; ambiguous or overlapping candidates are refused.

Respawn calibration uses the upper median of latest strictly prior death gaps
for named lives and a 750000us half-width. Missing calibration makes no deduction.
Both time bounds are inclusive; a unique victim claiming multiple bodies is
contested, and a missing index or overlapping named life is refused. These are
native deductions, not directly recorded identity facts.

lives.go retains Death's three fields and lifeSpan's seven fields, including
human/bot identity, cause and naming provenance. Owner composition preserves the
first conflicting index and separately marks ambiguity; later equal-index lives
may update the XUID. The scalar abs/min/max helpers are represented by Rust's
wrapping absolute/min/max operations in their consumers.

lives_decoupe.go maps to identity_lifetimes.rs and the owner/document consumers.
The 1024-case lifetime oracle compares scaffold, death-time conversion, full
refinement output and fallback counts. Death evidence has precedence over
creation and round boundaries; death matching begins 150ms before the previous
end, creation must be strictly after it, and round boundaries may equal it.
Unjustified gaps merge, transferring identity only if the preceding interval is
anonymous and not a bot. The no-death-evidence fallback remains counted and is
published as repli_vie_coupee_au_trou_de_replication. All conversions preserve
native signed truncation/wrapping behavior. This audit updates mappings for
existing behavior already exercised in the latest 494-test suite.

## Death-clock calibration and publication filtering

lives_death_offset.go maps to identity_death_clock.rs and identity_deaths.rs.
The 512-case native oracle compares candidate centers, their match counts,
refinement output and final offset/matched/runner-up. Voting uses two shifted
150ms grids with one vote per pivot/bin; the smaller directional count wins.
Up to three separated candidates are refined on a 10ms grid and the upper median
of the best plateau is selected. Candidates within 300ms of the current winner
are not counted as independent runner-ups. Refinement matching consumes deaths
in input order; final life/death pairing instead sorts by distance then life
using native unstable tie behavior, checked by the 1024-case lifetime oracle.
Owner assembly retains the complete clock result, and health exposes offset
only when matches exist. Pathological signed-limit refinement is bounded in Rust
rather than reproducing a native panic/nonterminating loop.

Published-track helpers are implemented through Rust sets and stable filtering
at their consumers. replay_published_track_xuid returns a nonempty track XUID
first, then a nonzero bridge XUID formatted as unsigned decimal. It does not
replace the explicit track identity on disagreement. Generic mutable Go slice
aliasing is not reproduced as an API contract.

Neutral-death filtering now also emits the native removal log. The extended
257-case fixture compares full output plus 245 actual INFO records and 12 silent
controls. All previous 256 input/output fields remain unchanged. A new nonempty
fully retained case confirms silence and explicit-track identity precedence.
Unknown kinds and actors without published identities are filtered in input
order; only actual removals produce the two-count diagnostic. Provenance is in
fixtures/neutral-deaths-v41.md. This is publication evidence, not independent
validation of death classifications or an architectural change.

## Inventory helper and dead-reading audit

inventory_ammo_rules.go maps to keyframe_inventory.rs: each of four slots retains
optional magazine/reserve/gauge, flags and overheat, plus raw gauge quantum.
Partial parses retain slots, selected index, cursor and completion status. The
2048-case native partial-ammo oracle checks those fields and exact f64 gauge
bits. Candidate solving requires the native end cursor, rejects simultaneous
magazine/gauge and rejects payload in an absent slot. The consumer retains total
candidate count and selects the first valid start in its 300-bit window.

Grenade counts use four slots and enforce the supplied per-slot maximum. The
free search requires a nonzero sum; the ammo-relative -216..-127 search accepts
a valid all-zero count vector. Selection scans offsets 200..210 after the last
known family and requires a matching availability mask and an available one-based
selection; disagreement returns -1. The consumer preserves read/positional flags
and selected rank. All 256 native keyframe outputs and JSON roundtrips exercise
these helper paths. Existing broader scanner/Film error-path acceptance limits
are not removed by this source-level audit.

inventory_dead_readings.go already maps its marking algorithm to
mark_replay_inventory_dead. Only unknown empty records are eligible; an
established bridge and temporal occupant are required. It finds the latest death
at or before the frame timestamp and marks through an inclusive 8000ms window.
Offset addition and frame reconstruction retain native wrapping arithmetic.
The 1024-case native publication oracle compares projected/filtered input,
complete marked output and count; FilmReplayInventory retains marked_dead.
The native empty-inventory INFO log is not yet emitted, so this source remains
partial. These are native attribution rules, not independently recorded causes
for every empty inventory observation.


## Inventory logging follow-up and stale-gate correction

The conditional empty-inventory log is now emitted after track filtering and
death attribution in build_film_replay_inventory, at the native assembly stage.
The extended 1024-case oracle captures actual native and Rust logs:887 INFO
records and137 silent controls, comparing all four attributes and level/message.
Every prior fixture field is unchanged. See fixtures/replay-inventory-publication-v41.md.

Correction to the generic inventory error-path notes: the legacy registry guard
was already narrowed to legacy signature passes, and complete-constructor
inventory-only, weapon-only, combined-failure and healthy controls already pass.
The current test is replication::tests::map_api_retains_other_channels_after_optional_scan_failure.
Independent missing-archetype errors and movement/navpoint partial-error retention
were completed in later checkpoints too. These supersede older pending claims;
remaining setup-order/context and broader captured gates are separate questions.


## Ability observation follow-up

The 1024-case ability fixture now also compares identity coverage INFO and the
actual native impulse/charge assembly-stage observations (2474 INFO, 598 WARN).
The previous fixture fields are unchanged. The Rust document emits identity
coverage before equipment changes/translocations and impulse/charge observations
after them. Missing-scan warnings use raw input counts; absent-component fields
remain booleans. The palette log and global document observation order remain
open; this closes the charge/impulse source logger gaps, not full acceptance.
See `../fixtures/replay-ability-charges-v41.md` for provenance.


## Palette and vehicle observation follow-up

Palette selection now logs the native identifier or `non classee` plus published
read/label counts. 1024 native assembly-stage observations agree, preserving all
prior fixture fields. The document order is identity, equipment changes,
translocations, palette, impulses, charges. This closes the earlier palette gap.

Vehicle coverage emits the three native summaries and conditional missing-aim,
per-chassis fallback and no-family warnings. 513 actual native logging cases
agree (1536 INFO, 937 WARN, nil silence); only the unspecified native map order
is canonicalized. Existing tally/publication oracles remain unchanged. Heading
source, ride-resolution and broader vehicle observer/context diagnostics remain
open; neither these source labels nor logging agreement establishes full parity.


## Vehicle heading and ride observation follow-up

Heading provenance INFO now agrees with the existing 1024-case native track
oracle on 24776 source samples, including zero-count empty clouds. Counts are
computed before decimation using the same film-heading/velocity eligibility as
published headings. Roll is counted as unpublished mode only on velocity fallback.

Ride-resolution INFO/WARN agrees with 512 native combined cases (345 INFO,
75 WARN, 167 silent). The native silence gate is episodes == 0 AND gap fallback
== 0, whereas its warning gate is episodes > 0 AND named == 0. All eight INFO
attributes are retained. The document emits coverage, heading, then ride logs;
its absent vehicle layer uses empty source/tally inputs as the native assembly.
Every prior fixture field and publication assertion is unchanged. See the new
fixture provenance notes for reproduction. Source scan/death diagnostics and
shared FilmContext profile/observer behavior remain independent open gates.


## Vehicle death observation follow-up

The successful vehicle death-read diagnostic now projects the existing shared
march facts: total deaths, ti=40 deaths, ti=40 record/clean/mask/desync counters,
event and located packet counts, retained-default calibration flag and match ID.
Only a positive ti=40 desync count selects WARN. 512 native logger cases compare
all fields and level (256 INFO, 256 WARN), including missing ti=40 keys while
other archetypes have losses. Source integration is after successful vehicle
scanning, never after an absent band or creation/position failure, and adds no
second traversal. Broader source failure and profile/observer-context paths
remain open; this is not the full build_vehicles.go acceptance gate.


## Vehicle source scan follow-up

The observed source entry point now emits the absent-ti=40 INFO at the census
exit, before its keyframe denominator is discarded. The map-aware constructor
emits the final vehicle scan summary after shared death observations. Standalone
scan calls retain their existing signatures and do not emit replay-stage logs.

Actual native decodeFilmVehicleScan fixtures exposed a semantic omission: no
biped band makes native occupant aim unavailable, not a successful empty read.
Rust now retains this in issues and emits the native warning when source
observation is enabled. The failure remains additive. Forty-eight native cases
check source success flags and all ordered vehicle logs: 30 absent-band exits,
eight successful scans without bipeds, ten successful scans with bipeds. Counts:
66 INFO and eight WARN. Keyframe-only cases exercise census/slot counts, not
nonzero delta-derived summary counters; those and other source failure/context
gates remain open. See fixtures/vehicle-source-logs-v41.md.


## Vehicle source delta-bearing validation

Expanded source fixture preserves its first 48 cases and adds 32 delta-bearing
cases using independently generated native creation and position inputs on their
matching maps. Eighty cases now compare source success, complete creation records
and stats (89 records, 103 anchors), world-position projections (122), complete
events (64), aim output and ordered logs (129 INFO, nine WARN). This closes the
prior zero-only evidence gap for creation/position/event summary fields.

Shared native march facts are input to this source composition test, not derived
by Rust. A first fixture pass incorrectly used the already-filtered vehicle deaths;
retaining all native deaths fixed it and explicitly covers one nonvehicle death.
Native generation asserts repeated shared-march stats agree under the same MPP
context. Production code is unchanged. Nonzero source aim/vehicle-death/occupancy
summary evidence and remaining failure/context paths are still separate gates.


## Vehicle MPP restoration and creation-failure audit

The source oracle now varies inherited and supplied calibration widths; all
existing records/logs remain unchanged because v41 format 27 declares widths
9/5. Native full-profile equality is checked before/after 88 source calls,
including successful scans, no-band exits and missing-archetype creation failures.
Rust uses the same width resolver then immutable precision inputs, satisfying
the vehicle-local no-leak requirement without introducing shared mutation.

Eight truncated-registry cases now compare exact creation-failure WARN records;
Rust retains its issue and partial slot count. Other Rust creation error displays
are not claimed to equal native wording. The detailed failure reachability table
in VEHICLE_SOURCE_CONTEXT_AUDIT.md separates required map/registry/loaded-byte
inputs from lazy IO and malformed-data recovery, without declaring broader source
API/context parity complete. New totals: 88 cases, 129 INFO and 17 WARN records.


## Ability contract declaration audit (current producer paths)

Pinned `types/grammar_abilities.go` declares six contracts. The earlier manifest
note that four declarations still needed mapping is superseded by this audit.
This is a field/producer audit, not a claim that every captured mode or source
failure has been validated.

| Native contract | Rust retention and producer |
| --- | --- |
| AbilityCharge | `AbilityCharge`: Slot -> slot; Chunk/TimestampUS -> source.chunk_index/source.timestamp_us; PacketIndex -> optional packet_index assigned from native packet ordinals; Emplacement/Charges/Low -> emplacement/charges/low. Producer: ability_charges.rs. The mask has three positions; the 7-bit energy word produces bounded high/low nibbles, so u8 retains all emitted wire values. |
| AbilityChargeStats | `AbilityChargeStats`: Records/WithI56/Read/Unread/Armed/Absent/Scanned -> records/with_component/read/unread/armed/absent/scanned. All fields are serialized; existing native stats fixture covers both boolean flags. |
| AbilityImpulse | `AbilityImpulse`: Slot -> slot; Chunk/TimestampUS -> source fields; PacketIndex -> optional native ordinal; Predicted -> predicted. Producer: ability_states.rs retains tag-one emissions and keeps their predicted/non-predicted source distinct. |
| AbilityImpulseStats | `AbilityImpulseStats`: Records/WithI57/WithI59/Read/Unread/Tag1/Absent/Scanned -> records/with_predicted/with_non_predicted/read/unread/tag_one/absent/scanned. All fields are serialized. |
| AbilityRank | `BipedAbilityEmission` returned by `BipedChannels::ability_ranks`: source/slot/packet ordinal as above; Counter -> counter; Rank -> rank. The component reads Counter with 3 bits and Rank with 6 bits, so the narrower Rust integers preserve all emitted values. `ability_ranks` excludes the no-rank gate, matching native ScanAbilityRanks; `ability_emissions` separately retains it as None for equipment-change consumers. |
| AbilityRankStats | `BipedChannels::ability_stats`, a `BipedChannelStats`: Records/WithI48/Read/Unread/Gated -> records/with_component/read/unread/gated. The stats belong to the ability channel, separately from camouflage. |

All three record families have serializable source metadata; unknown packet
ordinal remains None for unframed caller-supplied data rather than becoming a
fabricated zero. The existing framed-source regression in ability_states.rs
checks impulse/grapple/charge ordinals. The full Theater run includes that test,
the component value/hook oracles and charge/impulse stats contracts. Complete
producer-to-Film acceptance still requires the captured integration gates; arbitrary
out-of-wire-range values constructed by callers are not claimed as native reads.

Related constructor audit: biped_creation.go carries a profile but readCreation
uses only fixed-width primitive reads. Rust's fixed prologue scanner therefore
does not omit a profile-dependent component dispatch on that path. Native
ScanBipedCreations and ScanHeldWeaponChanges return setup errors before their
scan counters accumulate; the existing Rust empty failure result does not discard
nonzero native counters in those setup paths. Later invalid caller-supplied anchor
ranges are a separate contract from native source scanning.

## Source player-table diagnostic retention

scan_replay_film_player_table_with_diagnostics now retains the native underlying
refusal error alongside the existing projected table. ReplayFilmPlayerTableError
distinguishes missing identity, truncated identification/table input, unknown build,
table not found and still-compressed input. No-registry refusal has no decoder
error, matching the native nil error. Unknown-build increments are retained as
per-call publication data using the native metric name and value.

The actual source oracle adds native log observations and deltas read from the
levelup expvar namespace to its 19 unchanged original cases. A twentieth case
verifies that one remaining compression layer yields the distinct compressed
error while retaining the native truncated refusal category. Error text and
counter deltas match. Film.player_table_diagnostics uses the same projection
from the existing table without rescanning; the source result and Film exports
roundtrip, including unavailable diagnostics in older Film JSON.

The targeted source/Film test passed (4.67s). This supersedes the source-error
retention gap above, but not native log delivery or process-global metric
accumulation. Full Film source fidelity, broader integration and the deferred
architecture remain separate contracts. See fixtures/film-player-table-source-v41.md.
