# Film detection and replay coverage

This document describes the v41 implementation at `d824422` (2026-09-30).
It covers both the canonical parser and `TheaterRuntime`, and should be updated
when a decoder, semantic mapping, or independently validated replay feature lands.
Other film versions are unsupported.

We can preserve and inspect substantial replication data today, accumulate raw
entity/component state, and query typed summary interpretations. We do **not**
yet provide a complete player/camera/projectile replay or a named event for every
player action. A decoded field is not automatically a validated gameplay fact.

## How to read the coverage

- **Raw fields:** the canonical parser reads the supported wire layout and keeps
  values, ordering and source ranges. Quantized values remain quantized.
- **Interpretation:** the runtime exposes a labeled higher-level read or guarded
  association. Inspect its completeness, padding, refusal and derivation metadata.
- **Typed feature:** a consumer-facing semantic API exists. This still carries
  the feature's documented provenance and limitations.
- **Not established:** no validated semantic mapping or dedicated implementation
  exists. Related fields may be present, but their existence does not prove the
  recording contains the requested input/action.

All raw coverage is conditional on reaching the record, having the required
schema/context, and completing the relevant read. Unsupported layouts can stop
an enclosing stream before later supported records are reached. Missing data is
not evidence that an action never happened.

## Player movement, view and input

| Feature | Available now | Remaining work |
| --- | --- | --- |
| Player 3D position tracking | **Raw fields.** Object-position components retain absolute, predicted, baseline and delta branches, including quantized coordinates and precision gates. Raw component updates are accumulated by the runtime. | Resolve a player to the correct biped/entity over time; establish map/region bounds, units, delta baselines and prediction semantics; expose typed positions and validate trajectories. Parser-side dequantization hooks were disabled and removed; there is no public position track today. |
| Player body orientation and velocity | **Raw fields.** Forward/up, angular velocity and translational velocity component layouts are supported. | Decode packed vectors into documented coordinates; resolve reference frames/parents and player entities; validate orientation and velocity across updates. |
| Player aim tracking | **Raw fields + interpretation.** Desired-aim components retain yaw/pitch fields; actor/action blocks retain packed directions. Selected fire reads expose optional aim evidence. | Establish units, coordinate conventions, which vector describes which actor, and continuous aim state. Aim evidence at a shot is not a complete camera timeline. |
| Player camera tracking | **Not established as a camera track.** Aim/orientation fields and selected zoom reads provide related evidence. | Identify actual view transforms, camera origin, offsets, FOV, zoom transitions and first/third-person modes. Do not substitute body orientation or tactical-map heading for the player's camera. |
| Analog movement input | **Raw fields.** Supported control entries retain player index, baseline, two analog scalars, optional third scalar, extra fields and flags. | Validate axis meanings, normalization/deadzones, baseline accumulation, and player attribution; expose typed input snapshots. |
| Player jump input | **Not established.** Generic control/action and biped-action fields are retained, but no validated jump-button mapping or typed jump-input event exists. | Identify the recorded input bit/state with controlled recordings, including held/repeated/released inputs and negative cases. A rising position or airborne state must remain a separate derived physical-action observation. |
| Player crouch input | **Not established as input.** `unit-crouch-component` decodes a flag and progress; control/action fields are also retained. | Distinguish crouch state from the button press; validate toggle/hold behavior, timing and attribution. Expose crouch-state changes and crouch inputs separately. |
| Player grenade input / throw | **Not established as a throw event.** Grenade counts and desired grenade mask/selection are decoded. Generic action fields are retained. | Map throw input, windup/release and cancellation; connect inventory changes and projectile creation without treating an inventory decrement as proof of a button press. |
| Player pinging | **Not established.** Waypoint/navpoint layouts are decoded, but no validated player-ping event is exposed. | Identify the ping-specific wire action and its actor, target/location, type and lifetime. Distinguish player pings from objective markers, waypoints and other UI markers. |
| Player AI scanning | **Not established.** There is no dedicated validated AI-scan input/effect mapping. | Obtain scan/no-scan recordings, identify the action and any revealed targets/radius/duration, and validate attribution. Removed legacy highlight/diagnostic scans were decoder searches, not the player's AI-scan ability. |
| Sprint, slide and posture | **Raw fields.** Slide, posture, mobility and biped-action layouts are supported. | Establish the semantic values and transitions, player ownership, and timing. Separate physical state from input intent; expose typed events only for validated meanings. |
| Melee input / impact | **Not established as typed melee events.** Generic action and damage evidence can be inspected. | Identify the input, attack phase, target and impact outcome independently; verify misses and canceled actions. |

## Weapons, projectiles and combat

| Feature | Available now | Remaining work |
| --- | --- | --- |
| Weapon firing | **Interpretation.** Selected type-0 packet heads produce `FireRead` evidence, including optional shooter index, unit reference, weapon ID and aim. This is exposed through `interpretations()`, not a dedicated `EventKind::Shot`. | Validate complete source-backed reads and shooter/weapon associations; unify validated fire evidence into typed events. A fire read does not prove a hit, damage or victim, and absence of a selected head does not prove no shot occurred. |
| Ammo, reserves and heat | **Raw fields.** Supported weapon components retain magazine/fraction, inventory rounds, ammo scalars, heat and flags. | Validate the meanings of less explicit scalars, associate weapons with players, and expose typed state. Ammo changes alone do not identify firing, reload or pickup causes. |
| Weapon switching and reload input | **Not established as input events.** Desired weapon-set, ammo and reference fields provide related raw data. | Resolve equipped/held weapons over time and validate switch/reload inputs, phases, cancellation and outcomes. |
| Zoom / scope | **Interpretation.** Selected packet heads expose a slot and zoom level, with padding/provenance metadata. | Establish actor linkage, exact levels/FOV, state transitions and complete temporal coverage. |
| Projectile tracking: grenades | **Raw fields, not typed tracks.** Supported projectile defaults retain conditional position, direction, speed, target and reference fields. Projectile components retain rest state, command tick, tether and deceleration flags; shared object components can supply additional updates. | Identify grenade type, owner and entity lifetime; resolve coordinates/velocity and trajectory updates; decode bounce, fuse, detonation and destruction evidence where recorded. Validate each grenade family independently. |
| Projectile tracking: weapon projectiles, including Skewer | **Raw fields, not weapon-specific tracking.** The same supported projectile/object grammars are available when the corresponding entities are recorded and reached. | Establish projectile-to-weapon/shot/owner linkage and per-weapon archetype or asset identity, including Skewer. Validate spawn, travel, impact and removal. No tested guarantee of a complete Skewer trajectory exists today. |
| Hitscan weapons | **Interpretation.** The runtime retains selected weapon-damage reads and source references. | Resolve attacker/victim/weapon and impact meanings, validate shot-to-hit relationships, and distinguish misses. Do not invent a persistent bullet entity for an instantaneous hitscan result. |
| Health, shields and regeneration | **Raw fields.** Body/shield vitality components retain health/shield scalars, flags and conditional regeneration fields. Maximum-vitality and damage-section layouts are also supported. | Establish scales and maxima, player linkage, regeneration/damage semantics, and typed health/shield state/events. Do not assume the recorded integer is already a percentage. |
| Damage, assists and outcomes | **Raw fields + interpretation.** Supported event-list layouts retain recorded fields; selected damage reads provide additional labeled evidence. | Validate damage amounts/types, source/target relationships and assist meanings; account for incomplete/opaque bodies. Join with summaries only when source evidence supports the association. |
| Equipment, abilities and powerups | **Raw fields.** Equipment deployment, energy, charges, creator/activation fields, ability energy/selection/state and active-camo layouts are supported. Selected pickup and translocator packet heads have runtime interpretations. | Resolve equipment/ability identity, ownership, duration and effects; validate each powerup/ability separately. Translocator reads may need external map quantization context, which normal runtime loading does not currently supply. |

## Identity, events and replay infrastructure

| Feature | Available now | Remaining work |
| --- | --- | --- |
| Player identities | **Interpretation + typed summary actors.** Bootstrap/player-table reads expose roster evidence. Summary actors contain source-read XUID/gamertag and explicit unique/missing/ambiguous roster linkage. | Establish a complete time-varying player-to-controlled-entity relationship, including respawns, bots, joins/leaves and ownership changes. A control index or entity slot is not itself a gamertag. |
| Entity lifetimes and component state | **Runtime support.** NEW/DELETE/delta/keyframe records are indexed; supported state accumulates with source references and generation checks. Partial reads and separate keyframe namespaces remain explicit. | Add typed component semantics and relationships; preserve uncertainty around incomplete NEWs, keyframe-only observations and unsupported updates. An entity NEW is not automatically a player spawn. |
| Kills, deaths and medals | **Typed feature with derived provenance.** `summary_events()` and `query_summaries()` expose guarded v41 summary interpretations, including actors, timestamps and known/unknown medals. | Establish a complete sequential summary grammar. Opponent/weapon relationships are not fabricated from opaque kill/death bodies. Count agreement alone does not establish complete decoding. |
| Objectives and mode events | **Raw fields + typed summary category.** Several game-engine/team/objective components are decoded; summary payloads include mode events. | Map mode-specific payloads to concrete actions/outcomes, participants, scores and object relationships; validate each mode. A mode category is not a fully decoded objective action. |
| Ordered event log and filters | **Runtime support.** Source-linked structural, lifecycle, state, control, recorded-event and summary entries have chronological ordering. Filters cover category, kind, entity/player index and time; summaries additionally support XUID/medal queries. | Add validated named gameplay events and per-player associations. `RecordedEvent` currently means a supported record read, not a guarantee of an understood physical action. |
| Pause/advance/seek state | **Runtime support.** The runtime exposes a materialized raw-component world, forward advancement, rewind and checkpointed seeking. Seek/sequential consistency is tested. | Build a playback clock and typed world suitable for a renderer. `current()` is O(1) borrowed access; seek is not O(1). See [resolution costs](RESOLUTION.md#costs-and-playback). |
| Browser 3D replay | **Not implemented.** The source-linked runtime is a foundation for it. | Supply map geometry/assets and coordinates, typed transforms and camera state, a browser rendering pipeline, visual interpolation, playback controls and a synchronized filtered log. Keep interpolation separate from recorded state and show gaps. |

## Structural limits that affect every feature

- Registry, packet envelopes, supported datum/keyframe/frame layouts and raw
  component fields are decoded. Unknown chunks/payloads, unread suffixes and
  explicit stops remain inspectable; retention is not semantic decoding.
- Replication type 8 is acknowledged, but its body has no decoder. Other
  unsupported packet types retain their source rather than acquiring a guessed layout.
- The sequential event reader stops when the code-15 runtime setting is
  unavailable. We no longer infer this gate. Other unknown/unsupported event
  layouts and opaque bodies also limit coverage.
- Control handler kinds 1/2 and the secondary control block are unsupported.
  The supported kind-0 control path does not imply complete input-stream decoding.
- Canonical summary parsing reads a count and retains an opaque stream; runtime
  summary candidates are guarded interpretations, not canonical record boundaries.
- Some pinned readers model zero-tail/padded reads. Padding and partial results
  must not be materialized as recorded gameplay state.
- Parsing uses the current explicit v41 profile. Film/map-dependent precision,
  bounds and runtime settings still require evidence; supported component names
  do not establish universal correctness across every possible v41 layout.

## Remaining work, in dependency order

1. **Make raw coverage measurable.** Report reached/complete/partial/opaque/refused
   packets and records, unsupported components and stop reasons per film. Preserve
   all remaining bytes and avoid treating a missing action as a negative result
   when the relevant stream was not decoded.
2. **Establish player/entity/asset relationships.** Resolve controlled bipeds,
   weapons, equipment and projectile identities over time with source references
   and explicit missing/ambiguous cases.
3. **Resolve geometry and view state.** Establish map bounds and quantization,
   baseline/prediction/relative transforms, vector encodings and camera semantics.
   Produce typed recorded positions/aim before adding visual interpolation.
4. **Map inputs with controlled films.** Prioritize jump, crouch, grenade, ping,
   AI scan, fire, reload, switching and melee. Record isolated actions, holds,
   releases, canceled attempts, repeated inputs and no-action controls. Validate
   physical outcomes separately from input intent.
5. **Resolve combat and projectiles.** Validate grenade and weapon-projectile
   families separately; link spawn/travel/impact/removal, owners and shots only
   where supported. Add typed health/shield/equipment changes.
6. **Close structural gaps as evidence becomes available.** Decode unsupported
   control/event/packet bodies and the sequential summary grammar through known
   layouts, without speculative recovery.
7. **Build the replay consumer.** Add assets/rendering, clock/pause/resume, seek,
   interpolation and log synchronization over the typed runtime state.

These are follow-up tasks, not claims that the recording necessarily contains
all requested inputs or enough information for every visual effect.

## Evidence required before promoting coverage

For each named feature, keep versioned expectations for actors, exact wire
fields/timestamps, expected actions/outcomes and negative cases. Expectations
must come from independent annotations, controlled recordings and/or a pinned
reference; they must not be generated by the code under test.

- Compare ordered fields and boundaries with the pinned decoder and retain unknown data.
- Compare typed events with independent annotations. Wire timestamps compare
  exactly; define and justify a separate tolerance for observed physical-action times.
- Assert player/entity linkage and supported world state at selected times.
- Require checkpoint seeking and sequential playback to produce the same state.
- When browser replay exists, verify rendering/log synchronization, filtering,
  pause/resume and backward seeking.

The current suite covers 32 captured films and 403,465 reference contexts,
3,667 summary interpretations plus independent service counts, structural
boundaries, negative/refused cases and runtime seek consistency. This does not
establish semantic correctness for every action above. A recording named
`jump`, `ping` or `grenade` in the corpus is useful source material, not proof
that a corresponding typed event is implemented or independently validated.
See [validation](VALIDATION.md) for exact scope and provenance.

## Implementation pointers

- [Canonical format](FORMAT.md), [architecture](ARCHITECTURE.md),
  [runtime resolution](RESOLUTION.md).
- Raw coordinates: [position decoder](../parser/v41/chunks/replication/components/position.rs),
  [orientation decoder](../parser/v41/chunks/replication/components/orientation.rs).
- Inputs: [control views](../parser/v41/chunks/replication/replication_stream/frame/controls.rs),
  [action blocks](../parser/v41/chunks/replication/components/control.rs),
  [unit state](../parser/v41/chunks/replication/components/unit.rs).
- Projectiles: [default states](../parser/v41/chunks/replication/components/defaults.rs),
  [projectile components](../parser/v41/chunks/replication/components/projectile.rs).
- Runtime: [world accumulation](../runtime/resolved/world.rs),
  [event models](../runtime/resolved/events.rs),
  [interpretation dispatch](../runtime/resolved/interpretation/mod.rs).
  Public access is through `TheaterRuntime` and its `runtime` model modules.
