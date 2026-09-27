# Native FilmFacts cache versus recording parity

Pinned reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81.
This audit distinguishes derivative cache contracts from decoded-field retention.
It does not declare cache compatibility complete or remove pending source entries.

## What the native cache represents

replay/filmfacts.go identifies its current magic as REPLAYINPUTS25 followed by a
newline. FilmFacts embeds FilmInputs and adds Film, MapModule, AxisW and
LayoutDetected. These identify the match and the actual quantization context,
including detected widths that may differ from the catalog. They are necessary
for interpreting cached quanta; a catalog match alone cannot substitute for them.

The source explicitly says that nine short player-slot fields and the session
token do not enter this format because replay assembly does not consume them.
Its options method also deliberately omits external geometry and structure.
Consequently agreement with this cache cannot establish all-data film decoding,
canonical hierarchy retention, original-source preservation or re-encoding.
Native cache roundtrip is a different contract from a recording roundtrip.

## Checked field-retention paths

filmfacts_statsdepose.go serializes eleven EquipmentPlacementStats fields. Rust
EquipmentPlacementStats retains all eleven: scanned, calibration, lives, slots,
anchors, accepted, confirmed, placements, by_id, format_version and
format_sans_profil. Film.equipment_placements retains the containing stream.
The nested EquipmentMppCalibration retains selected/runner widths, scores,
anchors/chunks/lives and by_widths. Rust represents the struct-keyed width map
as explicit rows in native candidate preference order; the Go cache sorts map
keys for deterministic bytes. Those are separate ordering contracts.

filmfacts_mortsdobjet.go serializes deaths, thirteen statistics fields and a
frame/profile snapshot. Film.native_march_facts retains deaths and occupancy in
facts, all four per-archetype coverage maps, keyframe/delta/packet counts and an
optional calibration with selected encoding, default-retention flag and
best/runner scores. The existing source scanner/oracle checks these producer
outputs. This mapping alone does not prove equivalence of every field in the
cache's full ProfilDeBalayage snapshot: the distributed Rust profile/context
representation remains under audit. Do not promote that contract on the basis
of matching accepted death counts.

The follow-up found and closed one specific omission: FilmMarchFacts now retains
walk_policy (view limit, generation strictness and simulation completion), which
previously traveled only as scanner arguments. It accompanies the retained
calibrated encoding and survives Film serialization. Old exports and scans with
no deltas report None rather than inventing settings. The existing 256-case
native loaded-march oracle directly compares both profile flags and verifies
full JSON roundtrip plus old-export absence; 201 cases have deltas, including
100 strict-generation and 41 simulation-complete native configurations. This
does not establish the rest of the full native profile/context API.

## Packet preamble retention

FilmMarchFacts now also retains the actual packet_preamble_bits used by its
calibrated walk. The value and non-event packet start come from one shared
constant (the native FilmContext scanner uses two bits). Old exports and scans
with no delta/calibration retain None. The existing loaded-march oracle supplies
the independent Config.PacketPreambleBits expectation; this change does not
pretend that the entire native profile snapshot is already retained.

## Remaining work

Keep the cache codec entries pending until their intended compatibility scope
is explicitly resolved. Continue checking all decoded producer outputs and
profile settings independently of the cache's consumer-only projection. Do not
recreate the Go cache as a substitute for finishing native recording parity.
The faithful Film/ResolvedFilm/playback redesign remains deferred as recorded
in NEXT_PHASE.md. This document is evidence for that distinction, not a change
to the current parity acceptance criteria.

## Full profile retention follow-up

The earlier profile-snapshot caveats above are superseded for the full native
march path by `NATIVE_MARCH_CONTEXT.md`: FilmMarchFacts.native_config retains
NativeFrameMetadata, including the complete native scan profile. The 256-case
loaded-march oracle, four captured source contexts, and six integrated inherited
profile documents compare this configuration. Legacy encoding-only APIs and
older exports retain explicit absence. This closes that field-retention gap;
it does not implement the FilmFacts binary cache codec.

The raw disk-cache adapter is a separate implementation in `film_cache.rs`.
See `../fixtures/cache-partial-metadata-v41.md` for the subsequent native
partial-source selection regression and its remaining selected-integer limit.
