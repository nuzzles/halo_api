# Theater source trace

The event-body additions and melee-clang correction follow libfilmshell
`events.rs` at `f050371a6cda83b9887eeb83a1884336fce28e53`. The independent
162-case trace suite is documented in
[events-f050371-v41](../fixtures/events-f050371-v41/README.md). Older fixtures and catalogues retain their original pins.

The masked-property enum-width correction follows pinned LevelUp
`components_managed_property.go`, `default_state_arch.go` and the instruction
analysis in `LOTCBIS_PHASE0.md`, at commit
`413971bcc485be53a2fa8cf229e170539a5146f8`. The independent exporter executes the
original variant/default functions and verifies all 33 original captured vectors
from `ti13_vecteurs_test.go`. Source Git blobs, endpoints, provenance and limits
are documented in [property fixtures](../fixtures/property-v41/README.md).
The conflicting libfilmshell fixtures retain their original 32-bit interpretation
as explicit comparison evidence; their expectations are not rewritten.

The stat-text fixture executes pinned `src/variant_stats.rs` to export fields
8–11 by definition row. Their string-index interpretation is supported by the
captured source tables and agreement with original `variant.rs` label output;
individual field roles remain unnamed. See [stat-text fixtures](../fixtures/stat-text-v41/README.md).

The runtime reference-rule view follows pinned `src/model/variant.rs` alias/
prefix/default policies, while preserving unknown and ambiguous recorded inputs.
Its independent fixture executes original `src/variant.rs` option decoding and
original `GameVariant::rules`/`Rule::effective`; source values remain exact. See
[rule fixtures](../fixtures/variant-rules-v41/README.md)

The named runtime variant view follows pinned libfilmshell `variant.rs` and
`variant_stats.rs` paths and option-byte meanings, using the local canonical Bond
tree instead of upstream anchor searches. Original-source fixtures retain the
rounding/default differences explicitly; see [VARIANT.md](VARIANT.md).

The runtime script-candidate view adapts the filename predicate in pinned
`src/scripts.rs` and the mode precedence in `src/model/variant.rs`. It searches
only the canonical script-data blob and labels results as hypotheses, retaining
source occurrences and competing rules. The original-source fixture checks all
256 trigger combinations and both blob and whole-registry corpus results; see
[script evidence fixtures](../fixtures/script-evidence-v41/README.md).

The reference `film/catalog/medals` and `film/catalog/options` modules adapt pinned libfilmshell
`game/medals.rs`, `medals_data.rs`, `options.rs` and `options_data.rs`, including
Murmur3 hashing. Their importer checks the original data blobs; the independent
fixture executes all four unchanged source modules. See
[GAME_CATALOGUES.md](GAME_CATALOGUES.md)

The reference `film/catalog/damage` and `film/catalog/physics` modules adapt pinned libfilmshell
`game/damage.rs`, `damage_data.rs`, `sandbox.rs` and `sandbox_data.rs`. Their
generator checks both data blobs, and their independent exporter executes the
original lookup functions and helpers with exact f64-bit serialization. See
[DAMAGE_SANDBOX.md](DAMAGE_SANDBOX.md)

The reference object-tag catalogue in `film/catalog/items` adapts pinned libfilmshell
`src/game/items.rs` and `items_data.rs`, including its names, enum classifications,
parent/part links and variant hashes. The local generator checks the original
data blob, and the independent fixture runs original lookup functions. Explicit
revision selection retains the source's missing game-build provenance. See
[ITEMS.md](ITEMS.md)

The shared snapshot-refusal fixture adds trace callbacks to the original
libfilmshell snapshot/component readers. It preserves their damage-count refusal
instead of treating a shared stop as an omitted reader. Its exporter proves no
vehicle-branch probing occurred in that captured prefix.

The native 8,191-entry keyframe traversal follows pinned LevelUp
`413971bcc485be53a2fa8cf229e170539a5146f8` `keyframe_entrees.go` and
`keyframe_fullstate_loop.go`, corroborated by libfilmshell's sequential
`snapshot::walk`. The [table fixtures](../fixtures/keyframe-slots-v41/README.md)
extract the original Go readers with blob verification. Header scanning and
vehicle-branch probing remain outside the canonical table reader.

See [Validation](VALIDATION.md) for the pinned reference revisions and fixture
provenance.

The runtime engine timer conversion follows LevelUp
`413971bcc485be53a2fa8cf229e170539a5146f8` `quantize_endpoint.go` and
`vitality.go`, with exhaustive original-source evidence described in
[engine fixtures](../fixtures/engine-v41/README.md). Lives interpretation
follows pinned libfilmshell `gamestate.rs`; canonical raw words remain intact.

The direct event interpretations in `runtime/actions` follow the retained body
fields in pinned libfilmshell `events.rs` and `model/event.rs`, with the local
source-supported projectile and teleport differences retained. The independent
semantic exporter adds test serialization only; see [ACTIONS.md](ACTIONS.md).

The v41 control kind-1 command, kind-2 payload, and secondary unit-block readers
are adapted from OpenSpartan's
[libfilmshell](https://github.com/OpenSpartan/libfilmshell/tree/18067987ebc3628ad499c819c6aee814692df43b),
specifically `components.rs::control_view`, `command_record`, and `unit_block`.
The port retains bounded source fields and does not import upstream's gameplay
reconstruction or zero-tail reads. See
[control fixtures](../fixtures/controls-v41/README.md) for the independent
upstream boundary trace and its validation limits.

Additional component layouts in `components::{basic,m4b,managed,navpoint,object,scene,tacmap,vehicle}`
and the explicit snapshot point form in `components::position` are adapted from
that same pinned `components.rs::block`, `filter`, `point`, and handle readers.
The [component fixtures](../fixtures/components-v41/README.md) retain independent
upstream primitive traces and row-version/mode expectations. Fixed-width raw
fields retain all source bits even where upstream only skips them. Runtime
projections and unavailable snapshot gates are separate from those recorded fields.

Projectile/vehicle snapshot creation forms, biped reference/point/player fields,
manager property values and common-object count refusal in
`components::defaults`, plus raw snapshot media-frame words in
`components::orientation`, follow the pinned `creation.rs` and position reader.
The [creation fixtures](../fixtures/creation-v41/README.md) trace the original
upstream creation and manager grammars independently, including invalid-count
boundaries. Shared property widths and respawn/crew/flock/simulation point forms
follow the same pinned component readers; the component fixtures trace them.

Bounded transport admission/output limits follow the pinned `chunk::is_zlib`,
`MAX_CHUNK_BYTES` and `inflate::zlib` capabilities. The existing flate2 decoder
is retained. [Transport fixtures](../fixtures/transport-v41/README.md) use
independent Python streams and unchanged upstream decompression/limit checks.

The fixed bootstrap event-version/build prefix follows the pinned libfilmshell
`header.rs` layout and LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`
`vue_a_versions.go` (`FUN_1428e1c64`). The canonical reader retains all fields,
uses the verified registry endpoint, and does not import a build-string scan.
[Bootstrap fixtures](../fixtures/bootstrap-v41/README.md) distinguish independently
authored field goldens from corpus observations. Fire v3 admission now uses its
recorded version; optional map context remains separate. The 415 additional
[fire fixtures](../fixtures/events-v41/README.md) trace the original upstream
event walker, validating the already ported fire layout's conditional branches.

The following identification fields use the widths attributed to game writers
`FUN_14299b198`/`FUN_14299b278` in pinned LevelUp `film_identity.go`.
libfilmshell's unchanged `header::read` independently verifies the timestamp and
both map-variant GUID locations. The canonical model preserves zero values,
all fixed-buffer bytes and opaque-block content; the independent fixture
exporter records upstream's selective projections separately from those fields.

Conditional event bodies, event reference kinds, bounded game-event strings and
full-precision event-point layouts follow the same pinned `events.rs` readers.
The [event fixtures](../fixtures/events-v41/README.md) retain independent upstream
read traces, including the 123-entry handle table. Fixed default-box points retain
their source codes without projecting them into a map coordinate system.

The shared biped/NetworkedActionRequest action-variant reader follows pinned
`components::Reader::action` and `events::body(102)`. The
[action fixture author](../fixtures/actions-v41/README.md) supplies both
independently traced entry points, preserving prefix reads on an upstream nested
refusal without changing its branches or return values.

Default-box projectile point widths follow pinned `events::Bits::pos_codes`
at precision 15 and 12. The event fixtures trace both fixed and indexed forms;
map-coordinate interpretation remains separate from consuming their wire fields.

The generic Bond v2 wire reader follows Microsoft Bond's documented
[`compact_binary.h`](https://github.com/microsoft/bond/blob/e27170b0f768037087e5d3103b9e8035c0968626/cpp/inc/bond/protocol/compact_binary.h)
protocol rules. Valid boundaries are independently compared with the pinned
libfilmshell `variant::value`; no anchor search or rounded numeric projection is
used in the canonical AST. The sequential game-options envelope follows pinned
LevelUp `film_variante_de_partie.go`, writer `FUN_1407ec560` and reader
`FUN_1407ee138`. [Bond fixtures](../fixtures/bond-v2/README.md) distinguish the
protocol source, synthetic cases, original-reader traces and captured boundary
comparisons. Player-table projection remains in runtime, now using the canonical
boundary when complete and retaining explicit search provenance otherwise.

Script-v1 prefix admission follows `scriptDuFilm` and writer `FUN_142eec4d8` /
`FUN_1404f293c` in pinned LevelUp `vue_a_charges_execution.go`, with version 1
from `vue_a_versions.go`. It uses recorder simulation from canonical game options,
not the playback reader's current state. The independent script fixtures compare
the prefix-present branch with unchanged libfilmshell; the simulation-2 branch
is attributed to the writer analysis and synthetic coverage only.

Variant schema paths, scalar defaults and conditional events 85/116 follow the
same LevelUp pin's `film_variante_de_partie.go`, `vue_a_charges_execution.go` and
`LOT_VA_V3.md` writer analysis. The latter identifies the false defaults for
PlaybackSettings fields 0/2 and engine-type default 0. The 66 independently
exported projections execute the original variant walker with its Git blob ID
verified; negative schema/inheritance cases separately test stricter ambiguity
handling. The reference's policy of assuming unrecorded playback switches are
false is retained only in explicitly labelled runtime hypotheses, never in
canonical layout completion. See the [Bond fixtures](../fixtures/bond-v2/README.md).

The build-specific level catalogue is exported from the same libfilmshell pin's
`game/levels_data.rs` and `game/levels.rs`: all 33 levels and 65 BSP regions,
with exact floating-point bit patterns. The unchanged original `levels`,
`components::Params`, point and event readers supply 324 independent traces.
The fifth region of `mp_02` is outside their fixed four-slot reader table and is
validated separately from the source catalogue. Numeric engine-build identity
comes from the same pinned LevelUp `film_identity.go` analysis, distinct from
the film's product-version string. See [level fixtures](../fixtures/levels-v41/README.md).

Additional shared-reader evidence comes from pinned libfilmshell's attached
ObjectPosition and archetype-specific Bits8 readers, plus 75 fixed-width aliases.
Pinned LevelUp `components_frequences.go` identifies the two actual
high-frequency descriptor tables and refusal elsewhere. Its
`components_dynprec_orientation.go` supplies instruction-addressed evidence where
libfilmshell differs, and the independently executed 30/30 orientation decoder
used by engine-1 teleport. `vue_a_charges_execution.go` establishes that event's
engine-dependent call path. See the component/event fixture READMEs for exact
scope, Git blob verification and the lack of a captured engine-1 example.

Equipment tracked-stack count correction and conditional equipment-handle
verification follow the pinned libfilmshell readers and LevelUp
`equipement_porte_lecteurs.go` at `413971bcc485be53a2fa8cf229e170539a5146f8`
(blob `cc09fe6b2d984591366004954d3fb643eb05cad5`, FUN_140f72dec). The
[component fixtures](../fixtures/components-v41/README.md) retain the original
reader traces and document the advance-before-compare count rule.


Actor-state target positions and snapshot actor-action points follow the pinned
libfilmshell point reader and LevelUp `unit_control.go`,
`lecteur_position_exceptions.go`, and `bloc_action.go` at
`413971bcc485be53a2fa8cf229e170539a5146f8`. Their recorded call sites identify
FUN_14076e494's 0x10 precision argument and distinguish the historical 16-bit
compatibility skip. [Component fixtures](../fixtures/components-v41/README.md)
pin those source blobs and document the new unit/weapon trace scope and limits.


Tacmap waypoint/tether position grammar and waypoint row-version tails follow
the pinned libfilmshell readers. Display-asset, area-of-interest and POI-icon
positions additionally follow the instruction-addressed LevelUp analysis in
`lecteur_position_exceptions.go` (blob
`699712aa7591344f6b60831506ce1e74e8c14f78`). The
[component fixtures](../fixtures/components-v41/README.md) distinguish original
full snapshot traces from source-composed delta envelopes whose point subgrammar
is independently traced through libfilmshell. The legacy compatibility delta
helpers are not used as canonical oracles.

The canonical roster/bot readers follow pinned LevelUp
`413971bcc485be53a2fa8cf229e170539a5146f8` shared player-record grammar and
packet wrappers. Source-profile measurements remain distinguished from writer
evidence; see [ROSTER.md](ROSTER.md) and the independent
[roster fixtures](../fixtures/roster-v41/README.md).

Runtime roster observations consume the canonical player records. Bot team-byte
locations follow the pinned LevelUp `profile/equipe_bot.go`, distinguishing
HI_1_13_0 writer evidence from older-build corpus measurements. The optional
roster word's legacy replay-index interpretation is now exposed as an explicit
assumption; libfilmshell's first-match proximity takeovers are not treated as
recorded ownership.

The runtime HUD splash-text view follows libfilmshell `splash.rs` at
`18067987ebc3628ad499c819c6aee814692df43b` (blob
`76549535b70693bac166a8f4ee961b2b99a5800e`). It consumes canonical component
fields and preserves unknown arguments, source words and repeated observations.
The numeric convenience conversion is labeled reference behavior; grouping and
holder attribution are not imported. The [HUD fixture](../fixtures/hud-v41/README.md)
compiles unchanged original text/numeric readers for independent comparison.


Runtime listing assignments use the explicit convention in libfilmshell
`roster.rs` at `8f0f8d078e513c800baa21fac416fd4c0c127d03` (blob
`a32657daa1b5969321b96895b683c1f8c5b7df8f`). Human present-option and bot Slot
associations are compared with its unchanged scanners in the separately pinned
[assignment fixtures](../fixtures/assignments-v41/README.md). `holders.rs`'s
first-later/index fallback is not imported; absent options and incomplete or
competing populations remain explicit. Earlier fixtures/catalogues retain their
original revision scope.

The runtime unit-controller view follows libfilmshell `components.rs` at
`8f0f8d078e513c800baa21fac416fd4c0c127d03` (blob
`6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4`). Separate native fixtures execute the
unchanged UnitControl block. `owners.rs` supplies the relationship under study,
but its first-controller/first-end aggregation is not copied. Source-time roster
attribution preserves missing populations and exact event order; keyframe
namespaces remain distinct from runtime generations. See CONTROLLERS.md.

Parent/node/seat observations follow libfilmshell `components.rs` and `attach.rs`
at `8f0f8d078e513c800baa21fac416fd4c0c127d03`. The components blob is pinned by
the independent [attachment fixtures](../fixtures/attachments-v41/README.md);
attach.rs blob is `38c7f19233432de069c5f01aca840df8cd9f8a88`. The Unit-handle
base follows that reference convention. Object-handle mapping stays unresolved.
Its four-level root fallback, imputed ride positions and frame/film-end span
policies are not imported. ATTACHMENTS.md documents source gaps and lifecycle
boundaries, including generation reuse.

The source-linked ride view also references `vehicles.rs` and `assemble.rs` at
that same pin for declared biped/vehicle classes and upstream ride assembly.
RIDES.md distinguishes direct recorded links and current holders from upstream
copied poses, proximity-based part association, exit timing and aggregated AI
classification. The local join reuses attachment and controller evidence; it
introduces no second wire reader or inferred canonical fields.

Contextual replay now uses that same parent admission/value reader and keeps
its own source addresses, declaration context, path outcomes and current ownership
indices. This is a local integration of source-linked evidence, not a copy of
upstream's vehicle-position or exit-timing projection. Authored bootstrap-policy
and lifecycle model fixtures are distinguished from unmodified captured films
in RIDES.md and VALIDATION.md.

Vehicle replay tracks additionally use `snapshot.rs::raw_position` at libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03` for the three-zero dynamic-position
prefix and little-endian snapshot floats, and the existing class40/ordinary
absolute position grammar. VEHICLES.md records scope. Upstream later-tag fallback,
generation-zero filtering and proximity-based snapshot part links are not imported.

Mode-qualified zone events adapt `zones.rs` event tables and its ZONED predicate
at libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`, blob
`4701db66d587260404006301be25b4fb2ae1931f`. The independent zone fixture executes
that original source section. Strongholds' separate SH_CAPTURED broadcast is
retained as its own observation. Runtime/actions supplies the existing canonical
body interpretation. ZONES.md documents explicit mode applicability, unmodified
recipient/key evidence and the excluded aggregation/backfill heuristics..

Navpoint source views use `zones.rs` at libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03` for named groups/states, letter hashing
and the labelled progress clamp. They reuse local canonical navpoint fields and
existing independent component traces at their original `18067987...` pin.
Unlike upstream's zone projection, the view retains every conditional state/order
pair, negation, all-ones mask and raw progress code; it does not select a first
team or infer a default owner. ZONES.md records the distinction and limits.

Crate geometry uses `zones.rs::shape_at` and its Bond field mapping at libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03`. The independent geometry fixture runs
unchanged upstream source after checking zones.rs, bits.rs and calibrate.rs blob
hashes. The local view shares the existing bounded Bond reader and retains all
items, unknown codes and optional dimensions. Upstream first-item selection,
missing-dimension zeros, cylinder-right suppression, integer narrowing and default
orientation are not imported. ZONES.md and the fixture README document the
source boundaries and deliberate interpretation differences.

Crate orientation references `zones.rs::angle_at` at libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03`, while retaining the existing
instruction-backed dynamic-orientation grammar instead of the helper's raw-mode
offsets. The independent orientation fixture executes the unchanged helper.
Engine roll projection additionally references LevelUp `orientation_frame.go`
(blob `fe2f382c1bd333ebea227d205e7196cf7662537b`) and
`components_managed_object.go` (blob `8e0e8742f54a0f81fb222ebd544b3f8e97ae51ad`)
at `413971bcc485be53a2fa8cf229e170539a5146f8`, under
`apps/go-api/internal/games/halo_infinite/film/internal/grammar/`. Its fixture
executes unchanged RollAngleFromRaw/dequantMidpoint and records f32 results.
ZONES.md documents the separate reference convention and f64 projection limits.

Manager reference views use `zones.rs::objects_reader` at libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03` for the row2–17 index convention,
u32::MAX clear and labelled lossy projections. The original source's generation
removal, low16 map keys, idle-state synthesis and later state association are not
made canonical. Existing creation-v41 and components-v41 independent traces
establish the underlying fields. LevelUp `components_managed_object.go` records
the sixteen same-name descriptor rows, and `varwidth.go` documents category-zero
table writers that prevent assuming its base from width alone. ZONES.md records
these identity limits and the separate runtime/keyframe domains.

Hill properties and reference point projections use `zones.rs::hill_score_reader`
and its four property hashes at libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03`. The fixture exporter executes unchanged
reader bodies from blob `4701db66d587260404006301be25b4fb2ae1931f` for 60 arithmetic
cases. The runtime retains the reference
formula as an explicit projection while enforcing source-bound names, typed limits,
scope separation and lifecycle invalidation. It does not import the upstream's
inferred viewer/team mapping or raw-word coercion. Variant types follow the pinned
LevelUp `components_managed_property.go` described in fixtures/property-v41.

Snapshot object observations and the labelled stationary projection reference
libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`, `world.rs`
(blob `6aac5301bee54d94cf150fe2ee8abd15e20dd526`) and `snapshot.rs::raw_position`
(file blob `cd8971c6b1e53e593361fe29026324a98a861728`). The fixture exporter executes
the unchanged original world reader and raw-position helper with explicit input
adapters. Native bodies reuse the
independently traced creation-v41 fixtures. The local layer retains exact sources,
exclusions and gaps instead of interpreting repeated snapshot addresses as proven
physical lifetimes; namespace IDs are not recast as runtime generations.

Tick weapon-object history and pickup candidates reference libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03`, `inventory.rs`, blob
`c5cc80c8f3cf7093da53b73cf00ca9a4937ffc79`. The independent fixture executes that
complete file unchanged with explicit operation/event/catalogue adapters.. Local code preserves all timed positions,
unknown tags and ambiguous candidate relations instead of adopting last-position
backfill or arbitrary same-tag pickup assignment. Existing creation-v41 sources
supply the native weapon default fixtures; action/event fixtures establish code 9.

Recorded weapon rest flags reuse components-v41 original libfilmshell traces for
`item-at-rest-component` (both values, snapshot/delta, parameter 4) and the existing
canonical one-bit reader. Runtime admission is class 42, registry v41/27,
parameter 1, matching the retained Recharge registry. The independent LevelUp
413971bcc485be53a2fa8cf229e170539a5146f8 `dispatch_item.go` blob
b083f1af71c3f7bc478aa578eb07fccf27f67f6d records the same one-bit grammar at
FUN_140ff91e4. No additional upstream source or fixture bytes were copied.
The source-linked history and conservative timing/absence policies are local.

Fixed-position midpoint projections use the explicit LevelUp
413971bcc485be53a2fa8cf229e170539a5146f8 HI_1_13_0 constants:
`profile/loi_largeurs.go` (623a55741a4578c57bacb3a7abf47fffe8d5737d),
`profile/plages_quant.go` (185a6387d1852015f89e1363b9bf1475fd2b292a), and
`lecteur_position.go` (6033b0366c28cf899883ab35681ba4eb1f5f8b16).
The fixed-positions-v41 exporter executes unchanged bound/width functions and
libfilmshell 8f0f8d078e513c800baa21fac416fd4c0c127d03 `game/levels.rs`
`LevelBox::engine` (13d578d46185d47a51a6867322a0849b149c460a). It records 150
authored vectors with exact bound/width/result bits. Local output preserves the
f64 midpoint convention, not engine f32 arithmetic. No upstream inference or
fixed-domain fallback is added to canonical Film parsing.

Zero-width position axes are checked against LevelUp
413971bcc485be53a2fa8cf229e170539a5146f8 `profile/loi_largeurs.go`
(623a55741a4578c57bacb3a7abf47fffe8d5737d) and the unchanged
`lireE524Sur` / `largeursDeLaLigne` in `lecteur_position.go`
(6033b0366c28cf899883ab35681ba4eb1f5f8b16). The zero-position-axes-v41
exporter executes these functions with bounded primitive I/O and authored boxes,
retaining 80 width/read traces including R(0). This is independent source-backed
correction, not a claim that libfilmshell handles zero-width region tables.
Its first-axis-positive availability check can substitute primary widths.
Canonical explicit zero-width fields, runtime missing-field policies and
non-consuming detonation-tail shape validation are local additions.

Scoreboard update/holder joins reference libfilmshell
8f0f8d078e513c800baa21fac416fd4c0c127d03 `holders.rs`
(739462eb4fd874507459111a30ee98ba12f87df2), `gamestate.rs`
(205c0ad994014f68e86f72460f1973e47a167603), and `statlog.rs`
(516a8a12f0cc3e7436878a0cd1bd12ead431140f). Existing assignment-v41,
engine-v41 and component fixtures provide the field-level reference evidence.
The local join reuses canonical runtime histories and the roster resolver;
source-time ordering, explicit unknown/conflicting populations and refusal to
backdate identities are local policies. No new upstream bytes were copied.

Movement-input projections follow OpenSpartan/libfilmshell
8f0f8d078e513c800baa21fac416fd4c0c127d03 `control.rs`
(59aeb85cbfec1c3fee6821e58d7909e02dec1ea0). The inputs-v41 exporter runs that
complete reader and `components::control_view` unchanged on authored envelopes,
with an explicit control-start adapter.
Raw source retention, repeated observations, holder resolution and conservative
input support boundaries are local policies. Code 63 maps to +32 as implemented
by the source, despite its introductory -31..31 comment.

Reserve-ammunition histories reference OpenSpartan/libfilmshell
8f0f8d078e513c800baa21fac416fd4c0c127d03 `components.rs`
(6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4) and `ammo.rs`
(d1d7b803a5ec6eb72bc2b931a3db5b553dadb9e8). The reserves-v41 exporter
executes unchanged component parsing and the complete ammo reader on authored
slot updates with an explicit preassigned-life adapter. The existing canonical eleven-bit reader is reused; source retention,
independent selection/identity handling and local gap policies remain separate
from upstream's life attribution and deduplication.

Equipment energy/delay projections use OpenSpartan/libfilmshell
8f0f8d078e513c800baa21fac416fd4c0c127d03 `components.rs`
(6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4) and `ammo.rs`
(d1d7b803a5ec6eb72bc2b931a3db5b553dadb9e8). The equipment-energy-v41
exporter executes unchanged parsing, quantization and the complete ammo reader
with explicit parent/life adapters. Source-byte
validation, retained countdown-like repeats, conservative temporal admission
and source-time attachment/controller/holder queries are local policies.
The cached contextual replay uses the same scalar oracle and shared field
validation. Per-object sampling, separation of historical attribution from
current ownership, snapshot namespaces and gap/lifecycle bounds are local
policies, not claims about upstream's preassembled owner map.

Weapon charge and heat reference conversions use the same pinned components.rs
quantizer and ammo.rs rounding. The weapon-quantization-v41 oracle executes all
4,096 charge codes and 128 heat codes, all four flag combinations, and an
authored sparse update sequence through unchanged readers. Local native closed-gate zeros, retained raw observations, sparse sample
accessors and refusal of out-of-domain model words remain explicit local policy.

Transport-budget defaults reference OpenSpartan/libfilmshell
f050371a6cda83b9887eeb83a1884336fce28e53 `chunk.rs`, `container.rs` and
`film.rs`: 256 MiB decoded chunks, 1 GiB decoded films and 65,536 packet envelopes
per chunk. Configurable admission, separate original-byte accounting, typed
resource errors and source-preserving packet-limit stops are local policy and
implementation. The existing independent transport oracle remains pinned to its
original revision; no fixture bytes or reference results were replaced.

Secondary-control corrections reference OpenSpartan/libfilmshell
f050371a6cda83b9887eeb83a1884336fce28e53 `components.rs`
(9e7f58f027e8ed38c3aa61ebaedf97871fd3d18a), `unit_block` and its shared
primitive/handle/point readers. The controls-f050371-v41 exporter executes the
original `control_view` on 336 authored lists with primitive-only tracing.
Retained raw fields, prefix diagnostics, missing
context refusal and public Film/runtime source links remain local policy.
