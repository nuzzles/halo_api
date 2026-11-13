# Capability audit beyond component names

**Current target:** broad coverage of the latest libfilmshell capabilities and
a stronger source-backed canonical structure. The gaps listed here distinguish
known limits; exhaustive upstream parity and complete 1:1 decoding are outside
the revised completion criteria. See [the current scope](LIBFILMSHELL_ROBUSTNESS.md).

Latest source refresh: [f050371 audit](LIBFILMSHELL_UPDATE_F050371.md). It adds
event readers, resource policies and further component/runtime changes. Nine
local event bodies now have new-pin traces; remaining new policies/corrections
must be audited separately. Earlier tables and oracles retain their stated pins.

Reference: [OpenSpartan/libfilmshell at
18067987ebc3628ad499c819c6aee814692df43b](https://github.com/OpenSpartan/libfilmshell/tree/18067987ebc3628ad499c819c6aee814692df43b).
This audit compares its public `src/film.rs` surface and owning readers with
halo_api's existing film/parser/runtime split. It identifies remaining work;
it does not claim complete parity or a fully decoded film format.

**Upstream refresh:** HEAD advanced to `8f0f8d078e513c800baa21fac416fd4c0c127d03`
on 2026-11-10. The table below describes the original pin. Listing-based engine
IDs, controller ownership and parent-state rides now have additional upstream
evidence; see [the update audit](LIBFILMSHELL_UPDATE_20261110.md). Existing
oracles/catalogues retain their old pin. The older proximity-only descriptions
below must not be read as claims about the refreshed HEAD.

The corresponding local additions are now listing-derived assignments and
exact-event holder queries ([ROSTER.md](ROSTER.md)), source-linked unit controllers
([CONTROLLERS.md](CONTROLLERS.md)), and parent observations, snapshots, root paths
and direct attachment histories/spans ([ATTACHMENTS.md](ATTACHMENTS.md)). Controller-holder
continuity queries now combine direct control with roster/gap changes, and contextual
replay consumes the same readers/resolver. Source-linked [ride histories](RIDES.md)
now join direct seats, ancestor changes and current owners. Source-linked
[vehicle tracks](VEHICLES.md) now retain runtime positions and separate snapshots;
full life attribution and seated pose reconstruction remain open. Contextual replay now exposes ride observations
and sampled support using its own decoded parent bodies and source gaps.

## Wire format and context

| Capability | Local evidence | Remaining work |
| --- | --- | --- |
| Keyframe slot table | Sequential 8,191-entry native table, all fixed headers retained, vacant slots skipped without scans. Twenty-four original-Go traversal/guard cases plus full source coverage. The snapshot-only audit identifies 207 unrecorded vehicle-context stops and one damage-count refusal shared with upstream; that refusal now has a captured independent fixture. | Vehicle construction context remains unavailable. Physical slot order does not prove ID ownership or runtime clearing. The shared refusal is not evidence of a missing damage reader. Historical chain comparison remains a separate oracle. |
| Components/default state | All 286 upstream dispatch aliases have fixture evidence; 12,011 main component cases plus separate original-source suites. Creation fixtures cover all 23 upstream nonempty types. | Zero-bit position axes now follow the original width law and retain explicit named empty fields; following-field and truncation boundaries have independent source traces. Per-branch/build admission is distinct from name coverage. Snapshot vehicle context remains unavailable. Preserve source-supported disagreements with upstream. |
| Events | 1,494 main traces cover 42 upstream body codes. Script has separate recorded-context fixtures; kill has historical and variant/playback-context tests. 144 additional original-source projectile traces validate the conditional 19-bit tail. | The 44 dispatch routes do not establish all versions. Snapshot/full-precision projectile event admission remains unresolved. Missing kill playback settings still require refusal or explicit runtime assumptions. |
| Header/registry/variant | Canonical fixed bootstrap, recorded event versions, sequential identification fields, game-options envelopes and generic Bond values are retained in `film/registry`; parser context selects established script/fire/variant branches. | Identification internals and broader schema/build support remain incomplete. The generic Bond model retains values that lack named semantics. |
| Container/packet framing | Bounded packet/chunk reads, original source coverage, RFC zlib window admission and explicit inflated-output limit. Types 8/12 have source-backed player metadata readers; type 9 retains the supported summary layout. A terminal type-3 summary/highlights chunk is required for a complete film. | All retained local corpus films contain a summary. Zero-size/type-7 legality is not established beyond the corpus. Remaining packet layouts and versions are not made canonical by upstream's envelope recognition or scans. |
| Lossless representation | Raw fields, ranges, unknown values and explicit partial stops are retained. Source preservation passes the existing corpus at prior checkpoints. | Byte preservation is not complete structural decoding. No full field-driven film encoder/round-trip proof exists. Historical appearance comparison is still unresolved. |

Upstream `payload.rs`, `snapshot.rs`, `roster.rs` and `highlights.rs` combine
sequential readers with scanning or inference. The initial study documents those
algorithms. These paths must not promote candidate boundaries to recorded film
structure. Local unbound-DELTA search is already isolated in runtime and reports
its evidence; this does not certify every other recovery path.

## Runtime projections

The public names differ intentionally. A missing convenience API below does not
mean its source bits are missing: `runtime/resolved/world.rs` already accumulates
raw components from established NEW/DELTA/keyframe records, with generation,
source and completeness information. Conversely, retaining those bits alone
does not implement the corresponding semantic projection.

| Upstream surface / owning source | Local counterpart | Gap or condition |
| --- | --- | --- |
| `header`, `clock`, `players`, `player`; header/clock/roster/assemble | Canonical bootstrap and packet timestamps; an on-demand `packet_clock()` now exposes source-linked counter, game-tick and timestamp-aligned highlight-tick estimates for fully framed replay. Count-driven type-8 roster/type-12 bot records retain explicit partial stops and build context (see ROSTER.md); runtime player-table and identity evidence. `roster()` now exposes exact listing projections, independent human/bot populations and presence/absence/unknown history; replay consumes these canonical fields through current controller/holder resolution, with separate stable participant indices. | Compare captured clock/death alignment and complete header/roster semantics separately. Upstream roster searches and bot-takeover proximity are inferred evidence. |
| `variant`, `mode`; `variant.rs`, `game/options*` | Canonical Bond plus parser context for engine type, killcam and play-of-game. The explicit runtime schema exposes 24 settings, ten flag meanings, loadouts, exact option rows, stat definitions, scoreboard-column ID matching, the spawn-settings override, and indexed localized strings with reference-convention name/description, all with source links. Bounded script filename candidates and source-backed mode hypotheses preserve competing rules. Reference rule views retain alias matches, uncertain overrides and explicit default provenance. Stat-text views follow recorded indices, retain shared references and require agreement for consensus; see VARIANT.md. | Individual text-field roles, script archive/load/default applicability and stat-value presentation remain incomplete. Missing values, duplicates and unknown payloads stay explicit. Script defaults are not recorded overrides. |
| `level`, `coordinates`; level/calibration code | All 33 levels /65 regions bound to the recorded build; contextual position conversion. | Do not import inferred coordinate calibration as recorded map context. Broader build catalogues remain unverified. |
| `lives`, `lives_of`, `sample_at`, `alive_at`; replication/assemble | `PlayerReplay` lives, poses and observations now include unique-life `life_at`, strict `sample_at`/`alive_at`, and replay-set `samples_at`, alongside the original raw `at` lookup. Source-linked gaps interrupt transient bindings and all held channels after readable prefixes, including untimed transport loss. Visibility interruptions are distinct from observed deaths. | Replay integrates recorded controller/holder evidence, bots and reassigned IDs. Ownership interval starts are distinct from observed spawns; complete-life reconstruction and vehicle pose attribution remain open. API similarity alone proves no parity. |
| `state`; `model/state.rs`, stat/vital/weapon readers | Team, camouflage, vitality, grenade, weapon and equipment samples with source observations. Movement inputs now retain complete kind-0 controls, signed analog and optional raw stance, exact source-time holders and explicit sampling gaps (INPUTS.md). | Complete controls before unread suffixes remain inspectable; opaque packets, population/source loss and replay lifecycle clears end sampled support. Reserve-ammunition history now retains every complete source reading and uses independent registry slot rank; active/secondary samples follow existing sparse weapon presentation (REPLAY_RESOLUTION.md). Source-linked equipment energy/delay observations, histories and exact-event holder queries now exist over canonical records. Cached contextual PlayerReplay energy samples retain each supported object and distinguish original scalar attribution from current ownership, with explicit gap/lifecycle boundaries and diagnostic-only snapshots (EQUIPMENT.md). Delay units and complete recharge timelines remain unproven. Continuous key-held interpretation needs separate proof. |
| `grenade_flights`; `grenades.rs`, `assemble::pair_grenades` | Source-linked projectile tracks, generation-aware references, positions, deletions, interruptions and detonation events in `runtime/replay/projectiles.rs`. Singletons, zero-length support and equal-clock observations are retained. Later sparse updates can resume bounded sampling; actual source gaps cannot be crossed. | Upstream pairs throws with nearby creations by time and distance. That inference is not a recorded ownership relation and needs separate provenance if adopted. |
| `vehicles`, `rides`; `vehicles.rs`, assemble | Vehicle/default/component bits and raw world snapshots; source-linked `rides()` histories, direct seat links, full ancestor outcomes and current human/bot holders (RIDES.md). Contextual replay adds separate attachment/ride diagnostics, per-player observations and `ride_at()` samples through current ownership. Vehicle tracks retain ordinary absolute world positions, separate snapshot floats and lifecycle/source-gap support (VEHICLES.md). | Vehicle default-world and parent-local codes now have explicit fixed-domain reference midpoint conversion and sparse read queries (VEHICLES.md). Predicted/delta/writer-specific branches and seated poses remain open. Source intervals do not supply vehicle-center poses, inferred exit times or proximity-based part associations. A known direct ride can have an unresolved root or participant. |
| `world_objects`, `ground_weapons`; `world.rs`, assemble | `runtime.objects()` now retains source-checked keyframe tags/positions and explicit stationary reference candidates over complete snapshots, with original samples, exclusions, confirmation events and gaps (OBJECTS.md). | Repeated snapshot addresses and matching positions do not prove continuous rest or a physical lifetime. Tick weapon-object histories now retain timed positions, tags, boundaries and all same-packet/tag pickup candidates (GROUND_WEAPONS.md). Recorded item-rest flags now retain true/false/missing/error evidence with source timing. Explicit fixed-domain midpoint projection now handles Default/Local codes with separate domains and pinned constants. Physical rest-onset, automatic fixed-bound admission, parent transforms and pickup attribution remain open; a candidate is not a proven ownership relation. |
| `zones`, `hill_score`; `zones.rs` | Mode-qualified source-linked code-82 zone events now expose reference meanings, raw recipient indices, unique typed zone references and every keyed position (ZONES.md). Unknown IDs, duplicate keys and partial records remain explicit. Navpoint observations now retain exact docking filters, raw progress, default/per-clause visual states and order values with source/schema checks. | Bounded crate geometry now retains all items, shape codes, optional dimensions and map-team integers from the existing Bond reader. Crate orientation now retains all canonical modes and offers separately labelled compatible reference angles and source-backed raw/config-default-up heading projections. Manager target handles and all sixteen navpoint words now have source-checked, lifecycle-aware state; lossy index projections retain discarded identity bits. General orientation reconstruction, proven manager/zone/viewer identity joins, ownership/contest timelines and team hill-score attribution remain open. Source-bound hill inputs now expose typed limits, duplicate refusal and explicit per-scope reference points (ZONES.md). Explicit mode selection is an applicability assumption; no scanned location, recipient/team aggregation or later-target backfill is made canonical. |
| `scoreboard`; `gamestate.rs`, `model/scoreboard.rs` | `TheaterRuntime::scoreboard()` now exposes source-linked statborg identities, current/finalized values, extras, outcomes, history and timestamp queries. All 64 bits, initial zeros and repeated values are retained. Owner transitions, partial records and packet gaps stay explicit; see SCOREBOARD.md. | Source-time statborg/engine update joins now expose recorded roster holders, including unknown/conflicting claims, without relabeling old values after slot reuse. Engine history now exposes phase, round, exact-endpoint timer conversion and raw respawn/lives/active state with complete-default identity and shared gaps. Timer roles and final-score recovery remain pending. Upstream `stat_value` projects at most 32 bits; later-snapshot ownership and implicit zeros are not source evidence. |
| `events`, `kills`, `medals`, `highlights`; events/highlights/assemble | Structural `RecordedEvent` records, interpreted summaries and `TheaterRuntime::actions()` expose the earlier direct body families, translocator fields and nine f050371 additions, with exact scalar/source links and explicit unnamed tails where applicable. See ACTIONS.md. `runtime.hud()` adds source-linked static splash-text observations with localization keys, ordered typed/unknown arguments and explicit absent/error states; see HUD.md. | Kill victim/killer/assistant IDs and game-event recipient/player arguments have source-time holder joins with explicit claims, unknowns and source gaps. Kind-0/unit event handles now expose numeric runtime addresses and on-demand exact-event controller evidence, retaining absent/unmapped/no-controller outcomes (ACTIONS.md). Direct mode-qualified zone messages have a source-linked view (ZONES.md); proving event actor roles, weapon/damage attribution and remaining mode-event interpretation is still open. HUD viewer identity/counts, screen duration and suffix meanings remain unresolved. Upstream also adjusts/attributes some events by proximity and highlight timing; separate direct decoding from those hypotheses. |
| `raw`; `format.rs` | `Film`, packet source coverage, event/component fields and exact retained source references. | Preserve the more complete local representation. Upstream raw iteration can recover by marker search and does not retain every consumed field or original compressed chunk. |

`TheaterRuntime::motion_encoding_evidence(player_count_hint)` now exposes the
latest upstream first-three-chunk marker vote as a source-linked diagnostic:
bot and multiplayer vote counts, bounded examples, incomplete input positions
and the resulting optional label. It does not select a canonical packet/body
reader or turn a size-based heuristic into a recorded format discriminator.
The [current-source oracle](../fixtures/encoding-a13efb/README.md) records
upstream labels for all 32 retained films without a player-count hint.

Contextual replay can now join a supported ride to the carrier's sparse
source-backed vehicle position through `ride_carrier_position_at`; direct and
part carriers remain distinct. This exposes the vehicle center that upstream
uses when splicing ride motion, while leaving the biped pose unknown until a
seat transform is established. It refuses ended tracks and missing joins.

Weapon charge and heat now expose explicit pinned reference fractions and rounded
percentages while preserving their raw words. The exhaustive scalar oracle covers
all 4,096 charge codes, all 128 heat codes and every two-bit heat flag combination;
sparse native updates additionally validate independent slot ranks and source
locations. Closed charge gates retain native zero semantics instead of adopting
upstream's absent fraction. This does not establish heat-flag gameplay meanings,
engine f32 rounding, remaining-battery interpretation or broader continuity.
See [REPLAY_RESOLUTION.md](REPLAY_RESOLUTION.md#weapon-charge-heat-and-defaults).

Opaque non-frame packet bodies and unread control suffixes now interrupt all
contextual replay channels, including ownership, vehicle/ride state and older
weapon/vitality/pose channels. Raw source evidence remains available; fresh
controller/holder evidence is required before player support resumes. Repeated
unknown intervals retain every gap without redundant per-player/ride clears.
This conservative continuity policy does not decode the opaque bodies or prove
they actually changed state. Capture coverage can decrease substantially.

## Game catalogues

`game/levels*` has been imported with build restrictions. `film/items` now retains
all 2,001 upstream tag entries, 40 variant hashes, names, classifications and
parent/vehicle-part links. Queries require an explicit reference revision because
its source does not establish a game build. These data do not select parsing
layouts or infer instance ownership; see [ITEMS.md](ITEMS.md). `film/weapon` already
classifies known film IDs and retains unknown IDs; its standard magazine/heat
defaults are separate from recorded inventory. `film/damage` and `film/sandbox`
now retain all 468 effects, 151 firing entries, 294 projectiles and 64 vehicle
render-model bounds, with explicit revision selection and exact upstream f64
constants; see [DAMAGE_SANDBOX.md](DAMAGE_SANDBOX.md). They do not establish actual
hit damage, player vitality, inventory, instance geometry or vehicle physics type.
`film/medals` now retains all 157 reference medal definitions, including index,
category, difficulty, personal score and sorting weight. `film/options` retains
98 hash/name entries, seven scripts /104 options, every alias and exact literal
default. Both require explicit catalogue revision selection and preserve unknown
indices/hashes. See [GAME_CATALOGUES.md](GAME_CATALOGUES.md). The existing 124
empirical summary pairs remain unchanged; no service-NameId equivalence is assumed.
Option-script defaults remain separate from recorded Bond values. The runtime
reference-rule view connects their aliases and defaults to recorded rows with
explicit source and uncertainty; it does not establish actual script applicability.

The generated data headers identify game-file origin but do not by themselves
prove applicability to every recorded build. Pin extraction provenance and test
unknown/build-mismatch behavior before using these tables to admit parsing or
label runtime data.

## Next implementation boundaries

1. Continue the wire/context audit where an upstream capability actually adds
   established decoding: build/version admission, snapshot context and remaining
   packet grammar. Do not substitute scans for missing record boundaries.
2. Extend typed semantic views over already established source records.
   Statborg, engine/player history and direct event body views now exist; next
   independently establish roster-holder links, timer roles and mode events.
   Keep their models and transformations in `runtime`; retain source locations
   and unknown values. Reuse parser output instead of unbounded byte re-reads.
3. Add the catalogue coverage needed by those views, with namespace/build tests.
   Then address vehicle/ride, ground-object and zone timelines. Test ambiguous
   ownership, generation reuse, partial updates and film-start observations.
4. Compare runtime results independently; do not use derived accuracy, inferred
   ownership, adjusted event times or a retained opaque suffix as canonical
   format completion evidence.

The [robustness ledger](LIBFILMSHELL_ROBUSTNESS.md) remains the work tracker.
This first pass resolves the ambiguity between missing wire readers and missing
runtime APIs; field-by-field runtime parity and full canonical decoding remain
unfinished.

## Recorded corruption policy

Known-bootstrap guard policy now reaches Film and replay profiles. Nonempty
message trailers are retained separately from event bodies, and native empty
messages omit them. Existing NEW/DELTA/snapshot component guards and the distinct
snapshot-default word have original-source envelope tests plus public integration.
Missing-bootstrap corruption policy is now strict: both Film and replay stop at
the first affected body boundary, retain exact prefixes, and mark incomplete
sections ContextUnavailable. Empty events/masks and guard-free snapshot entries
still complete. Internal configured research fixtures do not change public
admission. This closes the corruption-policy fallback gap; broader build,
snapshot physics and other runtime-context admission remain unfinished.

## Event table and reference-configuration admission

Both Film and replay require the complete native 123-word version table for the
established v41/27 bootstrap before reading a nonempty message's references/body.
Missing/mismatched entries and configuration zero retain the message header and
an explicit stop; no following entity/control view is inferred. Empty lists can
still close. The supported table is independent of later header completion and
map/variant context, which retain their own admission boundaries.

An independent original-Go oracle covers 503 table classifications and 87 frame
cases. Canonical parsing deliberately refuses upstream's shorter-table prefix
hypothesis. This does not add historical builds, variable-cardinality bootstrap
layouts or configuration-zero global reference ranges. The 32-film corpus uses
only native tables and configuration one; negative cases have authored evidence.

The flag now also reaches shared component/default handles and control actions,
including the legacy position handle tail. Configuration-zero handles retain
known gates/probes and stop before their unknown-width value/generation; affected
sections carry ContextUnavailable. Fixed-width posture words, absent handles,
empty masks and vacant slots remain readable. Replay records control-stop reasons
at exact source boundaries. Native category-table behavior has 49 original-Go
cases; configuration-zero evidence verifies prefixes/refusal, not successful
global-range decoding. Establishing that range and wider writer layouts remains
open; a profile width cannot stand in for recorded evidence.

Runtime component-check suffix handling now accepts both source-preserved
keyframe and DELTA spellings through one strict validator. Native checked packets
cover controllers, attachments, crate geometry/orientation, navpoints and vehicle
snapshot positions; manager rows share the same validation. Missing/extra or
malformed suffixes refuse, and checksum words do not become semantic payloads.
This closes the runtime naming mismatch, not the unestablished checksum algorithm
or the broader full-format proof.

Navpoint history now retains independently supported docking, progress and all
eight visual groups, with timestamp and exact-event queries. Existing occurrence
iteration is preserved. Clears, partial reads, keyframes, deletes, generation
changes and packet/chunk gaps remain explicit evidence; runtime generations and
keyframe namespaces stay separate. This supplies lifecycle state for later zone
work but does not establish viewer filtering, manager/zone identity joins or
ownership/contest interpretation. See ZONES.md and VALIDATION.md.

The hill-score prerequisite audit corrected a native property-width regression:
masked tags 11–15 use a four-bit enum payload. libfilmshell's fallback used 32 bits
for 12–15; the earlier parity-driven import followed it. Original instruction-
addressed Go traces and 33 captured vectors now establish the corrected component
and default-state widths; original upstream oracles retain explicit disagreements.
See fixtures/property-v41. This does not establish hill-score attribution or the
separately unresolved class 13 NEW envelope.

`runtime.properties(PropertySchema::V41Registry27)` now provides source-linked
property defaults, name updates, scalar values and all 32 viewer rows, plus
lifecycle-aware timestamp/exact-event queries. It retains raw variants, domain
absence and explicit decode/gap evidence. Names and earlier value samples keep
separate source observations. The hill-score layer now binds names to subsequent values, preserves ordered resets,
refuses duplicate named entities, and projects points only with typed limits in
the same explicit scope/namespace. Timestamp/exact-event queries exclude future
inputs. Viewer/team identity and team score timelines remain unresolved. See
PROPERTIES.md and ZONES.md; the layer does not expand parser admission.

Snapshot object observations and stationary reference candidates now cover the upstream
world-object presentation layer. Native source checks retain signed-zero/raw fields,
unknown tags, partial records and excluded observations. The reference comparison
requires two complete snapshots within the first-anchor tolerance, with explicit
confirmation time, class/identity ambiguity refusal and source-gap boundaries.
Unread tick-entity tails remain attached warnings because they cannot hide a
separate snapshot; this does not establish continuous rest. Tick weapon-object histories and indexed, nonexclusive pickup candidates now
supplement this layer, including recorded item-rest flags. Physical rest-onset and pickup attribution remain open.
See OBJECTS.md, GROUND_WEAPONS.md and VALIDATION.md.

Scoreboard source-time holder joins now cover statborg and engine-player updates
through the shared recorded assignment schema. They preserve duplicate/unknown
claims, exclude future same-time listings and require fresh independent
populations after source gaps that can hide roster changes. They do not merge
participant totals, relabel old values on slot reuse or establish retrospective
finalized-round ownership. See SCOREBOARD.md and VALIDATION.md. Timer roles and
recovered final-score coverage remain open.


Replication type-7 admission now distinguishes the established empty end marker
from a nonempty unsupported body. Nonempty payloads retain all source bytes and
produce the same replay interruption as other unread packets. This closes an
admission/continuity gap without changing transport framing or interpreting the
unknown payload. All 451 captured replication end markers remain on the empty
branch; nonempty behavior has authored parser and runtime integration coverage.


Control lists now read the continuation bit at the 64-record resource boundary.
An exactly-full terminated list completes; a missing bit is truncated; another
record retains its presence bit before an explicit budget refusal. Independent
traces confirm larger upstream lists but do not turn the local policy into a
format maximum. See control-limit-v41 and the latest validation checkpoint.
