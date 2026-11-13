# Direct recorded events

> Historical decoder and runtime research. The source-inspection runtime APIs described below were removed by the typed-film refactor; use `Film.replication_stream.observations` and `TheaterRuntime` for the current API.


`TheaterRuntime::actions(ActionFilter)` interprets sequential canonical event
records on demand. It reuses the existing timestamp index and leaves the Film,
event ordering and playback cursor unchanged. The organization remains
film/parser/v41/runtime: Film owns exact fields, the parser establishes their
boundaries, and `runtime/actions` gives selected fields names and types.

```rust,ignore
use halo_api::theater::runtime::actions::{ActionFilter, ActionPayload};

for action in runtime.actions(ActionFilter {
    start_us: Some(start_us),
    end_us: Some(end_us),
    code: Some(36), // action_weapon_fire
}) {
    if let Ok(body) = &action.body {
        if let ActionPayload::Fire { player, weapon, .. } = &body.value {
            // Optional raw player index and weapon tag; no roster attribution.
            if let Some(word) = weapon {
                let source_field = &action.record.fields[word.field];
                assert_eq!(source_field.width, word.width);
            }
        }
    }
}
```

The filter has inclusive packet timestamp bounds and an optional raw event code.
Repeated events and timestamp ties retain their original order. Each result
borrows the original indexed `Event` and `EventRecord`; the event's `SourceRef`
can also be passed to `runtime.record()`. No copying of Film buffers or separate
event cache is required.

## Supported interpretations

The view admits v41/27 records and projects the earlier libfilmshell event
bodies, the existing translocator body, and nine bodies added at upstream
`f050371`. These are typed source fields, not gameplay attribution:

| Codes | Direct fields |
| --- | --- |
| 0, 5 | Damage tag/player/direction/secondary/small values; detonation kind, weapon specification, tag and encoded point |
| 9, 11, 21, 22 | Pickup slot/tag; empty click fields/player; zoom code; vehicle-exit seat and flags |
| 36, 38, 39, 40 | Fire prefix/player/weapon/variant; reload kind/player; grenade versus object throw; melee prefix/player |
| 64, 75, 76 | Respawn-target slot, encoded position and raw float words; AI dialogue priority/sound/line/player; 2D dialogue fields and mask |
| 82, 85, 100 | Game-message keys/values/parameters; victim/killer/assistant and exact percentage words; power-up tags/word |
| 32, 116, 117 | Teleported positions/tags; teleport-effect orientation codes and optional positions; translocator tag/positions |
| 2, 8, 12, 13, 14 | Motor-state width/value; vehicle-board seat; melee-clang players/tag/point/direction; motor-interruption width/value/flag; object-effect word |
| 80, 104, 105, 106 | Networked-AI form and zero/one/three encoded points; optional knockback vector; object-knockback tag; collision point/direction/index |

Every projected scalar is a `Word { raw, width, field }`; `field` indexes the
owning record's exact source fields. A recorded zero is preserved. Optional
players/tags stay absent when their gate is closed. Player indices are not
assumed to be roster indices, engine-player slots or XUIDs. Percentage words
are not clamped to a plausible range.

The nine f050371 body layouts consume every body field. An unexpected extra
field refuses the typed view, even if the enclosing record claims completion;
other action families may still expose documented unnamed tails.

The three outer references retain their original domain kind, optional kind-1
narrowing, relative index and generation words. They are not silently merged
into player identities or normalized using libfilmshell's mixed index domains.
The existing parser's reference-kind table is shared with the projection.

Recorded corruption guards have their own `EventFieldStage::CorruptionCheck`.
Action projection validates those fields separately and excludes them from both
payload interpretations and `remaining_fields`. A truncated guard leaves the
record incomplete, even when the preceding body fields are available.

Encoded points retain each original code, its exact width and the explicit
region, or `None` for the fixed default box. No map scaling, precision shift or
coordinate calibration is applied. Teleport effects preserve the source-selected
19/8-bit or 30/30-bit orientation form. Respawn-target float words retain the
MSB-first source integer; interpreting the source's little-endian float encoding
requires byte swapping before `f32::from_bits`. Their physical roles stay unnamed.

Game-message keys remain ordered, including duplicates. Text retains only the
recorded bytes, including NUL when present, up to the native 16-byte limit;
it is not padded or assumed to be UTF-8. Parameters preserve their kind and
24-versus-32-bit width. An absent extra block differs from a present empty one,
and an absent player parameter differs from an Empty parameter.

Indexed `ActionPoint` axes may have width zero when the canonical reader's
supplied region declares a single bin. The original field index and explicit
zero-width scalar survive; absent or unavailable fields are not synthesized.
Detonation's known suffix is now shape-checked after the point while remaining
in `remaining_fields`. This prevents a removed zero-width axis from shifting a
following direction word into the point without detection. The suffix receives
no invented semantic names or values.

## Completeness and remaining work

`ActionBody::remaining_fields` explicitly identifies the source tail after the
typed portion. Fire, damage, AI dialogue and detonation expose a semantic prefix;
unnamed fields remain in the original record. `Other { code }` means the parser
bounded a layout for which this view supplies no semantic body. Neither case
discards source fields or claims complete semantic decoding.

Incomplete records remain in the default query with `IncompleteRecord`, and
their known prefixes remain inspectable. References may be complete when the
body is not. Field width, stage, contiguity, bounds, scalar representation and
the recorded end are checked before body projection. Malformed field storage
produces an indexed error; unsupported film versions refuse projection. No
zero padding, fallback scan or runtime event-layout assumption is introduced.
For example, a kill prefix whose playback context is unavailable stays
incomplete in this API even if a research decoder can read that prefix.

The upstream public `model::Event` also includes derived attribution and mode
interpretations: headshot timing, fatal weapon, damage amounts, player/holder
joins, HUD messages, flag/zone events and nearby equipment ownership. These
are not completed by the direct body view. Existing summary and replay APIs
remain separate. Full runtime parity and canonical format decoding remain open
in [LIBFILMSHELL_ROBUSTNESS.md](LIBFILMSHELL_ROBUSTNESS.md).

## Source-time kill and game-event holders

`TheaterRuntime::action_holders(AssignmentSchema::Reference8f0f8d0, filter)`
adds explicit participant interpretation beside the unchanged direct action.
Pinned upstream `assemble.rs` maps all three kill IDs through `holders.api` and
does the same for game-event recipient bits and player-typed arguments. The
local view follows those domain meanings using the existing recorded population
resolver, with source-time boundaries shared by the scoreboard holder view.

```rust,ignore
use halo_api::theater::runtime::{
    actions::{ActionFilter, holders::ActionHolderBody},
    roster::AssignmentSchema,
};

for event in runtime.action_holders(
    AssignmentSchema::Reference8f0f8d0,
    ActionFilter { code: Some(85), ..Default::default() },
) {
    if let Ok(ActionHolderBody::Killed(kill)) = &event.body {
        if let Some(holder) = &kill.victim.holder {
            // Unique human/bot identity, unassigned, ambiguous or unknown.
            let status = holder.status();
            let claims = holder.claims(); // exact listing and member sources
        }
    }
}
```

`ActionHolderBody::Killed` retains victim, killer and assistant independently.
Each includes the original optional `Word` and an optional `HolderResolution`.
A recorded absent index has neither word nor resolution. A present zero is
engine ID zero and still resolves; unknown or unassigned is not converted into
absence. Damage percentages and suicide bits remain in the direct action.

`ActionHolderBody::Game` retains every set recipient bit as its own entry, in
MSB-first engine-ID order (0 through 31), with the original 32-bit mask word.
It also retains each player-typed extra argument's original list position and
optional player word. Empty and word-typed arguments are not player claims;
they remain available in the direct action. Repeated player arguments are not
deduplicated, and resolved recipients are not folded into a new 32-bit mask.
This avoids conflating recorded engine IDs with participant identities.

Both queries use the latest independent session/bot populations no later than
the action's exact timestamp and stable event order. Missing IDs, conflicting
claims and unavailable populations stay explicit. Unframed chunks, unread
chunk suffixes and discontinuous source sequences clear both populations;
fresh listings restore each domain independently. An opaque body within an
otherwise framed packet does not conceal a separate type-8/type-12 listing.
Filtering the actions does not filter the population history used for joins.
No bootstrap, first-later-listing, raw-index fallback or life-path ownership is
used. The shared cursor lives under `runtime/roster`; no canonical reader or
Film field changes.

Partial or unsupported action bodies retain the same error and source in this
view. `Other` means this schema has no participant interpretation for that body,
not that it has no participants. Other body player fields, damage/weapon
attribution and mode-specific role meanings require their own evidence. The
schema choice remains explicit and does not expand format/build admission.

## Event handle addresses and controller evidence

`ActionReference::runtime_entity_id()` maps recorded kind-0 object references
and unit references (kind 1 with its unit bit, kinds 4 and 6) to their numeric
runtime entity IDs, retaining the two-bit generation. Other kinds return `None`
because their short indices do not establish the same object address. The
source reference still carries its original kind, index and generation words.
This mapping follows current libfilmshell `events.rs::Handle::entity`; an
address is not proof of a live entity or an actor.

`runtime.action_reference_owner(action.event, slot, schema)` rereads the three
references from the borrowed event's canonical record and queries controller
ownership at that **exact event**, excluding later same-time updates. It
distinguishes absent references, unmapped kinds, a mapped address without
controller evidence, and the controller/holder status already provided by
`runtime/controllers`. Foreign events and invalid slots are errors. The query
is on demand; it does not scan for nearby entities, infer a life, or assign a
participant from the handle alone. Authored packet tests cover all three
address outcomes and foreign-event rejection, while the independent event
oracle checks numeric mapping over its recorded reference kinds and generations.
An integrated controller/listing/event packet sequence additionally checks that
two equal-time handles see different holder states across the listing boundary;
a handle naming another generation keeps the explicit other-generation refusal
and no inherited holder.

Five native-packet tests cover zero/31/absent IDs, exact-time order, repeated
events and slot reuse, complete recipient masks, argument positions, human/bot
conflicts, partial populations, source-gap restoration, unsupported/incomplete
bodies, source pointers and unchanged Film/cursor state. Their synthetic context
supplies listing build and disabled kill-tail settings; this is not new public
bootstrap admission evidence. Existing original-reader action and assignment
oracles remain unchanged. The retained capture audit compares the advancing
cursor with an independent bounded listing scan and checks source words, claims,
uncertainty and exact source boundaries. Results are in VALIDATION.md.

## Evidence

An independent exporter executes unchanged libfilmshell `events::walk/body` at
`18067987ebc3628ad499c819c6aee814692df43b`. Its 1,475 retained semantic outputs
cover all 18 upstream body families: 1,473 existing complete cases and two
authored kill boundary cases. The local test compares the upstream subset
exactly while retaining additional local fields, point codes and text length.

Additional tests cover source-supported projectile tails, engine-1 teleport
widths, translocator points, exact game-message strings/duplicates/parameters,
malformed fields, truncated records, missing kill context, public queries,
source pointers and unchanged Film/cursor state. The exporter adds only
serialization derives and a test harness to a temporary pinned Git archive;
it does not alter reader decisions. Fixtures and reproduction instructions are
in [events-v41/README.md](../fixtures/events-v41/README.md).

The captured check passes for Octagon (2,084 complete /96 incomplete records,
58,398 exact source scalars) and Recharge (28,626 /246, 543,432 scalars). Complete
counts include Other bodies. Both feature configurations pass 273 ordinary
tests and ten doctests; strict Clippy/Rustdoc, WASM and all 58 manifest hashes
also pass. Full checkpoint details are in [VALIDATION.md](VALIDATION.md).
