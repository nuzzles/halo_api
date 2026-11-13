# Canonical player metadata packets

> Historical decoder and runtime research. The source-inspection runtime APIs described below were removed by the typed-film refactor; use `Film.replication_stream.observations` and `TheaterRuntime` for the current API.


Type 8 (`RosterPacketBody`) and type 12 (`BotsPacketBody`) now own a
`PlayerMetadataRead`. The `film / parser/v41 / runtime` organization is retained:
Film owns exact fields and partial stops; the v41 parser reads sequentially;
identity, population history and player ownership interpretations belong in runtime.

Both packet types begin with a 32-bit count. Type 8 entries carry the format-21
optional five-bit word and a byte-copy identity. Type 12 entries carry three
32-bit words: absolute index, slot and bot ID. Both then use the shared player
body: mask, byte/word lists, fixed blobs, variable UTF-16BE name, representation,
join key, six short words, build-sized personalization and a 44-byte block.
Neither includes the bootstrap table's leading 85 bits or trailing 32 bits.

Every field retains its original payload bit position, width and MSB-first bits.
Names retain consumed NULs and unpaired surrogates. Identity and join key are
separate raw words; their byte-copy interpretation requires reversing bytes of
the MSB word. Mismatched keys are not normalized. The five-bit option remains a
raw canonical field; the explicit reference assignment view below interprets its
present value as an engine-player ID. Opaque blocks remain opaque in source coverage.
The 44-byte block retains each byte, including partial prefixes; team offsets
are build-dependent; the runtime projection below retains their provenance.

The recorded registry format controls the type-8 gate. An established complete
BuildBranch field selects the personalization width; later identification or
options sections need not complete. Unknown builds stop at personalization,
retaining the preceding fields. Unsupported/missing formats stop before the
first entry. An empty population needs neither context to read its count.
The internal reader admits formats 20/21/24/25/27; these unit-tested branches do
not expand the enclosing parser's supported registry/bootstrap layouts.

Personalization profiles follow pinned LevelUp: HI_1_12_0/HI_1_13_0 = 1,852 bytes;
HI_1_10_0/HI_1_11_0 = 1,492; HI_1_8_0/HI_1_9_0 = 1,312;
HI_1_4_1 = 2,052. HI_1_13_0 comes from the disassembled writer; other widths are
upstream corpus measurements. Unknown builds never inherit a nearby profile.

The first failed field ends traversal. Complete earlier entries and the partial
entry remain available. The byte-list limit is 2,048; the word-list limit is 192;
the bot-count limit is 64. These are explicit reader admission limits. Type 8
allocates only as entries are consumed, never from its untrusted announced count.
No plausible-name, marker, fixed-stride or successor search supplies boundaries.

Type 8 only checks bounded reading: arbitrary trailing bits remain unparsed,
with their exact range retained. Type 12 requires fewer than eight trailing bits,
which may be nonzero and are classified as padding only after successful closure.
Longer tails receive `TrailingData`. Original packet timestamps are unchanged.

Complete human and bot packets preserve raw engine-slot scoreboard state.
`scoreboard_holders(schema)` joins each statborg/engine update to the recorded
assignments at its exact event; see SCOREBOARD.md. Incomplete packets in either
population create `IncompleteRosterPopulation` scoreboard gaps. A complete empty roster needs
no parser gap. Replay now consumes the canonical fields through the same runtime
projection described below. Replay now uses the explicit assignment schema and
current controller/holder evidence, supports engine-ID reuse and bots, and keeps
stable output indices separate from engine IDs. The legacy
`roster_index_assumptions` output has been removed; see REPLAY_RESOLUTION.md.
An authored bot-only cold-start sequence verifies that a complete bot listing
creates bot participants without a human bootstrap table. Its controller stays
`Unknown` until a complete empty human listing rules out a competing claim;
the subsequent source-ordered ownership observation identifies the bot. An
unobserved human domain is not silently treated as empty.
The same native sequence now checks one ownership-started bot life and its
recorded controller-clear endpoint. The endpoint is an ownership change, not
an inferred death or proof of all bot lives.

The structural sources are LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`
`roster_type8.go`, `bot_metadata.go`, `player_table_record.go`, and
`profile/build_profile.go`. They establish count-driven boundaries beyond
libfilmshell's roster/name scans. Fixtures and reproduction instructions are in
[roster-v41/README.md](../fixtures/roster-v41/README.md).

## Runtime population observations

`TheaterRuntime::roster()` borrows the existing runtime. `listings()` walks an
index of packet events for types 8/12, including identical repetitions, in the
runtime's timestamp and stable event order. Each listing retains the original
Event and canonical metadata read. No frame counting, timestamp adjustment,
name scan, proximity rule or same-number owner fallback supplies membership.

`listing.members()` projects a complete unambiguous population. Human identities
require matching nonzero outer/inner identity words; bots use their recorded bot
ID in a separate namespace. Duplicate participants, conflicting keys, missing
fields, inconsistent counts, unsupported formats and partial lists refuse the
population. A refusal cannot establish that any player left. Raw partial and
conflicting fields remain available in `listing.read`.

Members borrow their canonical records and retain original word fields. UTF-16
units remain exact; `gamertag()` returns an error for invalid UTF-16, without
invalidating otherwise established membership. Bot team and mirror bytes use
pinned `profile/equipe_bot.go`: offsets 37/38 for HI_1_13_0 (writer evidence),
4/6 for HI_1_10_0 through HI_1_12_0 (upstream measurements). Other builds have no
team projection. Signed values, including out-of-domain values or disagreements
between the two bytes, are preserved rather than clamped or voted away.

`at(timestamp_us)` selects the latest listing in each independent domain. A
complete empty list is known absence; an incomplete latest list is unknown until
a new complete list arrives. Before the first listing the domain is unobserved.
This is a snapshot of the latest **recorded population observation**, not proof
that no unrecorded change occurred between listings. Existing bootstrap identities
remain separately accessible through `player(index)`; they are not backdated
packet observations.

`history(ParticipantId)` retains presence, absence, reappearance and unknown
observations in that participant's domain. Human XUIDs and bot IDs never collide.
Same-time listings retain packet event order. Queries leave canonical Film and
playback time unchanged. The index stores only event offsets; projection happens
on demand. Listing traversal costs O(listing count); `at` projects only the latest
human and bot lists, without visiting all frame packets.

```rust,ignore
use halo_api::theater::runtime::roster::ParticipantId;

let snapshot = runtime.roster().at(timestamp_us);
match snapshot.membership(ParticipantId::Human(xuid)) {
    Ok(Some(member)) => println!("{:?}", member.gamertag()),
    Ok(None) => println!("absent from the complete recorded list"),
    Err(reason) => println!("population unknown: {reason:?}"),
}
```

Replay has no second roster wire reader or zero-padding restriction. It consumes
canonical population fields through the shared assignment/holder resolver below.
Missing or ambiguous engine IDs block ownership; invalid display names remain
optional without destroying otherwise established participant identity. Dynamic
engine-ID reuse is supported with source-ordered control intervals. Bootstrap
indices remain output metadata and never establish an engine assignment.
Scoreboard source-time holder joins now share this resolver. Final participant
score aggregation and full-life reconstruction remain separate work.

## Reference engine-player assignments and holder history

`listing.assignments(AssignmentSchema::Reference8f0f8d0)` interprets complete
v41/27 listings under the separately pinned upstream convention. A session's
present five-bit `Option` supplies its engine-player ID; a bot's full 32-bit
`Slot` supplies its ID. Absolute index, bot identity and bootstrap order remain
separate. Each assignment retains the schema, original listing Event, member,
entry range and exact source word. Repeated entries/claims are not collapsed.
An absent option stays `None`; no preceding-bit scan supplies its value.
Older wire formats are not automatically admitted to this semantic convention.

`roster.at(time).holder(schema, EnginePlayerId(id))` considers the latest
independent human and bot populations. Its status is `Unique`, `Unassigned`,
`Ambiguous` or `Unknown`. All competing claims and missing-population/ID evidence
remain inspectable. A missing ID can prevent uniqueness even when another member
explicitly claims the queried slot. A failed latest population invalidates that
domain's earlier claims until a complete listing returns. It never proves a
departure. An unobserved bot domain is uncertainty, not an empty population.

Complete lists establish observed departures and slot changes. A newer list
replaces only its own domain: it cannot erase a competing claim from the other
domain. `holder_history(schema, id)` emits an observation after **every** listing,
including identical repetitions and same-time changes in stable event order.
No first-later listing, identity-equals-index fallback, bootstrap backdating,
takeover proximity or inferred departure time supplies a holder. These remain
observations of listed populations, not continuous-lifetime proof.

```rust,ignore
use halo_api::theater::runtime::roster::{AssignmentSchema, EnginePlayerId};
let schema = AssignmentSchema::Reference8f0f8d0;
let holder = runtime.roster().at(timestamp_us).holder(schema, EnginePlayerId(7));
println!("{:?}", holder.status());
for observation in runtime.roster().holder_history(schema, EnginePlayerId(7)) {
    println!("{}: {:?}", observation.listing.event.timestamp_us,
        observation.resolution.status());
}
```

Queries preserve Film and playback. They use the existing roster-event index;
history projects the latest populations at each observation without storing a
second film representation. The [assignment fixtures](../fixtures/assignments-v41/README.md)
pin original readers to `8f0f8d078e513c800baa21fac416fd4c0c127d03` and separate
native field agreement from local absence/ambiguity policies. Controllers,
life ownership, attachments and consumer migration remain subsequent work.

The 32-film comparison agrees with all 5,333 assignments returned by the original
scanner and retains 14 additional recorded assignments that scanner omits when
the first short is 21 rather than 183. Identical-payload original-Go outputs
corroborate all 5,347 local identity/option pairs. These are record occurrences,
not additional distinct players. See VALIDATION.md for source hashes and limits.

Type-9 managed-player creation now exposes its separately recorded six-bit
participant only when the complete NEW/keyframe default is in the upstream
0–31 domain. Replay uses that value for managed team-controller registration;
values 32–63 remain raw source evidence and cannot become engine-player IDs.
This does not equate the outer managed entity ID, bootstrap order or roster
position with a participant, and it does not prove a continuous holder between
population observations. See the managed-participant-v41 fixture.

`at_event(&Event)` additionally selects populations at the exact borrowed runtime
event, using stable event order to exclude later same-timestamp listings. Foreign
and cloned events return `RosterError::ForeignEvent`. The controller view uses
this method for source-time attribution; see CONTROLLERS.md. `at(timestamp_us)`
continues to include all events through that timestamp.
