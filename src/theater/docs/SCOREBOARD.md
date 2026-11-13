# Source-linked scoreboard history

> Historical decoder and runtime research. The source-inspection runtime APIs described below were removed by the typed-film refactor; use `Film.replication_stream.observations` and `TheaterRuntime` for the current API.


`TheaterRuntime::scoreboard()` projects statborg and engine/player records
already established by canonical parsing. It preserves the film/parser/runtime organization:
`film` owns the source fields, `parser/v41` reads their wire layout, and
`runtime/scoreboard` owns this interpretation. It does not scan packet bytes or
reuse runtime schema-recovery candidates.

```rust,ignore
use halo_api::theater::runtime::scoreboard::{StatOwner, Statistic};

let board = runtime.scoreboard();
let snapshot = board.at(timestamp_us);
let kills = snapshot.statistic(StatOwner::PlayerSlot(3), Statistic::Kills);
let signed_value = kills.and_then(|v| v.integer.signed());
```

`PlayerSlot(3)` is an engine player slot, not a proven roster index, gamertag or
XUID. The explicit source-time holder view below supplies population links. Team owners use
the low sixteen bits of the entry index; player slots use its high sixteen.
Both the complete index word and type byte remain available in `StatIdentity`,
including unknown kinds and the other half of the index word.

## Supported evidence

The projection requires v41/27 and the recorded archetype-6 sequence of 58
version-1 components: 28 current-round pairs, 28 finalized-round pairs, outcomes,
then entry identity. `schema_supported` is false for missing or mismatched
schemas; an empty result is not proof of an all-zero scoreboard. This admission
is a schema check, not a claim of support for every game build.

Each current pair exposes both five-bit round counts, both mandatory values and
both independently gated extra values. Every integer retains its bit position,
width and raw word, with signed conversion across 8/16/32/64 bits. A 64-bit value
is never truncated to the low 32 bits. Finalized pairs preserve the 32-bit mask,
each selected round and each recorded absent/present value. The 32 two-bit
outcome codes remain numeric.

`Statistic` names fourteen conventional slots; all 56 current-value slots
remain queryable numerically. This does not replace mode-specific stat labels
or reinterpret the variant's rules. Every projected row and identity has a
`StatSource` pointing to a canonical record and component index. Use
`TheaterRuntime::record` to inspect the original fields. Projected integer bit
positions are relative to that record's owning packet payload.

`ScoreboardSnapshot::finalized_value(owner, slot, round)` and
`finalized_statistic(owner, statistic, round)` expose one explicitly selected
finalized-round side with its source. A present mask bit whose side is marked
absent returns `Some(FinalizedScoreValue { integer: None, .. })`; an unset bit,
incomplete row, unknown owner or competing owners returns `None`. Both outcomes
remain distinct from numeric zero. The latest snapshot is a source-time view,
not proof that every round or the whole film is complete.

`finalized_sum(owner, slot, first..=last)` adds only a caller-selected, inclusive
span of finalized rounds from one snapshot. Every round mask bit and numeric
side must be present; missing owner/row, duplicate owner, absent side, invalid
integer and invalid range are separate errors. The result retains each source
word and uses `i128`, so 32 signed 64-bit rounds cannot overflow. It is an exact
sum of those recorded sides, **not** an automatic whole-match total: callers
must establish round coverage and owner lifetime separately.

## Timeline and uncertainty

`updates()` retains every projected record, including equal consecutive values
and the first observed zero. Updates use the established event timestamp and
order. `at(time)` folds these updates without moving the playback cursor and
returns the latest established state at that time. Its cost is linear in the
preceding updates and gap markers.

- NEW observations replace that entity's previous rows. A keyframe packet resets
  the whole accumulated view before its records establish new baselines; entities
  omitted from the table cannot survive from the preceding snapshot. Partial
  tables still expose their readable prefix before the unread-suffix gap. A
  keyframe observation has no fabricated runtime generation or creation time.
- An ordinary DELTA replaces only its recorded rows. A partial or unsupported
  row invalidates the old value; its known raw prefix stays in the Film.
- Identity changes invalidate old rows before applying same-record rows.
  Identity learned later cannot assign an earlier unowned statistic. Missing or
  unknown identities remain inspectable without a player/team attribution.
- Two entities claiming the same owner make an owner query ambiguous, even
  when one lacks the requested row. A stale-generation delete does not remove
  a newer established entity.
- Unread frame entity regions, incomplete keyframe tables, opaque packets,
  incomplete transport, unframed chunk tails and discontinuous packet/chunk
  sequences yield explicit `gaps()`. Unread prefixes/suffixes are ordered using
  the shared source-continuity visitor. Queries clear accumulated state at a
  gap; earlier observations remain in history. A later row without newly
  established identity stays unassigned. Valid keyframe resets have their own
  `KeyframeBoundary` reason.

Unknown values return `None`; they are not silently zero. These rules expose
the consequences of existing canonical stops. They do not recover unread
records or infer omitted entity deletions. Unread chunks with no framed packet
now invalidate state using their original untimed source reference. `at(u64::MAX)` is the
latest established view, not a guaranteed final match scoreboard.

## Source gap provenance and ordering

`ScoreboardGap.source` is a `ScoreboardGapSource`: either a packet SourceRef or
an unframed chunk's canonical manifest index (`chunk_index`). The latter refers directly to the
chunk in Film, preserving access to inflated bytes and manifest metadata without
creating a packet. `timestamp_us` is optional: packet boundaries retain their
recorded timestamp; unframed chunks have None. Manifest offsets never supply a
replacement packet clock. Complete empty clear/zlib chunks stay distinct from
unread data.

`before_event` identifies the gap's position in the full runtime event array.
An index equal to the array length means after its last event. StatUpdate and
EngineUpdate now retain `event_index` into that array; their existing `order`
remains the tie-break order within a timestamp. Event order resets at a new
timestamp and cannot serve as a global index.

The materialized scoreboard and engine histories share both gap records and an
immutable event timestamp index. Queries select the event prefix by time and fold
gaps in absolute index order. Timed boundaries beyond the requested time are
excluded even when their index immediately follows the current prefix. Untimed
chunks conservatively bound uncertainty between neighboring source data; reversed
source clocks use the earlier bound. A later valid baseline or identity-plus-row
update can establish new values without recovering the skipped history.

This shared handling covers statborg scores and engine/player fields, including
phase, round, timers and lives. PlayerReplay retains separate lifetime/state
machinery and remains subject to its own source-gap audit. Sequential admission
still refuses a NEW for an occupied slot; a gap does not manufacture a DELETE.

## Reference and differences

Owner halves and slot mappings follow pinned libfilmshell
[`statlog.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/statlog.rs)
and
[`model/scoreboard.rs`](https://github.com/OpenSpartan/libfilmshell/blob/18067987ebc3628ad499c819c6aee814692df43b/src/model/scoreboard.rs).
The corresponding captured registry and independent component traces verify
the local source layouts.

Upstream `gamestate::stat_value` reads at most 32 bits into its projected signed
value while advancing over a 64-bit field. `statlog::reader` may associate an
earlier update using a later chunk's ownership map, suppresses initial zeros
and repeated values, and uses holder inference. Those choices are not copied
into this source-linked history. Direct values and relationships must remain
distinguishable from inference.

The engine view below adds source-linked timer, phase and player-state history.
Source-time roster-holder resolution is available below. Assigning timer roles and
recovered final-score coverage still require implementation/audit.
At pinned f050371, upstream `Scoreboard::total` sums last observed current values
between round starts inferred from simultaneous spawn-count clears, or from a
clock fallback when no spawn updates exist. Its `row_cleared` rule substitutes a
prior value after three zero changes at one tick. The local finalized-round
records are a stronger direct source for selected round sides, but neither their
mask nor the current pair establishes complete lifetime coverage across gaps,
owner changes and missing rounds. We therefore do not report a total or silently
substitute zero for a missing round. The clock fallback also cannot be adopted
without resolving the timer-role conflict in the captured films.
Recorded variant stat-text references are now available separately; see VARIANT.md.
The broader tracker remains [LIBFILMSHELL_ROBUSTNESS.md](LIBFILMSHELL_ROBUSTNESS.md).

## Engine clocks and player state

`runtime.scoreboard().engine()` exposes `EngineHistory`, with independent
`schemas()`, `updates()`, `gaps()` and `at(timestamp_us)` queries. It works even
when the statborg schema is unavailable. Within v41/27 it admits uniquely named
version-1 phase, current-round and round-timer rows in archetypes 0/1/2, and
respawn-timer, lives-remaining and active-in-game rows in archetype 5. Missing,
duplicate or unsupported fields are not replaced with defaults.

```rust,ignore
use halo_api::theater::runtime::scoreboard::engine::{EngineField, EngineValue};

let history = runtime.scoreboard().engine();
let state = history.at(timestamp_us);
let lives = state.active_lives(3); // engine-player slot, not roster index
if let Some(engine) = state.engine() {
    if let Some(EngineValue::RoundTimer(timer)) =
        engine.rows.get(&EngineField::RoundTimer).and_then(|r| r.value)
    {
        let seconds_in_source_order = timer.seconds();
        // timer.quanta and timer.state retain all recorded words.
    }
}
```

`RoundTimer::seconds()` uses the original endpoint quantizer: 0 maps to 0,
65,535 maps to 36,000 seconds, and interior values use 36,000/65,534 with
separate float32 operations. Neither word is labeled elapsed or remaining;
those roles need temporal evidence. The captured [timer audit](research/libfilmshell/TIMER_AUDIT.md)
shows increasing second words in Octagon and decreasing ones in Recharge;
all changing pairs cross known source gaps. The five-bit state stays numeric. This
differs from libfilmshell's linear 65,535 divisor. The
[independent timer oracle](../fixtures/engine-v41/README.md) checks every u16
quantum against the original source with x86 scalar rounding.

Respawn timers keep the active flag and both ten-bit integer words without
inventing a unit. Phase remains the raw three-bit code. `Round(None)` is
recorded absence and differs from an unknown/incomplete row. Lives retain the
seven-bit code; `count()` follows the upstream interpretation (raw minus one,
with -1 meaning unlimited). `active_lives()` requires explicit active state.

Player ownership comes only from the complete six-bit `index` in that entity's
NEW/keyframe default-state body, with a source reference and bit position. A
later baseline cannot assign earlier observations. DELTAs retain established
identity, but cannot restore it after a gap. NEW/keyframes replace prior rows
and identity; partial rows invalidate old values. Generation-safe deletion
follows the existing canonical event changes. Competing engines or players
remain inspectable in `entries`, while convenience queries return `None`.

Engine and statborg history share the same gap markers. The engine view obeys
the packet-prefix and equal-timestamp ordering described above. It preserves
observations before a gap without claiming they are current after it. Raw
Film data and playback position are unchanged.

## Verification

Seven ordinary tests include synthetic complete Film keyframe/delta packets,
full-width signed boundaries, initial zeros/repetitions, both extras, sparse
finalized rounds, outcome codes, unknown/changed/duplicate owners, generation
reuse/deletion, partial baselines and packet gaps at equal timestamps. All 216
version-1 independent current/finalized component fixtures also project back
to their exact source values. Mismatched schemas and malformed duplicate/raw
fields refuse projection.

An additional ignored captured-film check follows every projected integer to
the original packet bits in Octagon and Recharge and requires explicit gaps.
It checks source fidelity, not full semantic agreement with a final scoreboard.

The latest captured check passes with 1,160 updates /64,616 exact integers /35,486 gaps
for Octagon and 12,128 updates /214,517 integers /110,664 gaps for Recharge.
The gap counts describe existing canonical unread regions; they are not newly
recovered events or a completeness percentage.

Six additional engine tests cover all 65,536 timer conversions, actual Film
NEW/keyframe/DELTA/deletion bytes, both default-version gates and six-bit slot
boundaries, absent rounds, lives limits, explicit active state, competing
identities/engines, generation reuse, partial defaults/rows, schema refusal,
equal-timestamp gaps and unchanged canonical bytes/playback cursor. The timer
oracle executes the original reader under all 32 tail states as well.

The captured test additionally checks every engine row and default identity
against its canonical record and packet bits. Octagon supplies 1,239 updates,
2,693 rows and 704 identities; Recharge supplies 5,385 updates, 9,419 rows and
1,952 identities. Each captured round timer also matches the exhaustive
float32 oracle. These are source-observation counts, not full-match coverage.
The same test now checks HUD component observations; see HUD.md. The HUD change
does not modify scoreboard state or gap calculation.

```sh
cargo test --offline --all-features --lib scoreboard::tests
cargo test --offline --all-features --lib captured_scoreboard_values_keep_canonical_sources_and_expose_gaps -- --ignored --nocapture
```

## Recorded holders at update time

`runtime.scoreboard_holders(AssignmentSchema::Reference8f0f8d0)` provides
`stat_updates()` and `engine_updates()`. Each observation borrows its original
update and exact runtime Event, retains the established statborg identity or
engine-player default field, and supplies a `HolderResolution` for player slots.
The resolution retains every source-linked claim and uncertainty from the latest
independent human/bot lists at that event. Later same-timestamp listings are
excluded. Teams, missing/partial identities, unknown identity kinds and removals
remain visible without a fabricated participant.

```rust,ignore
use halo_api::theater::runtime::roster::AssignmentSchema;
let holders = runtime.scoreboard_holders(AssignmentSchema::Reference8f0f8d0);
for observation in holders.stat_updates() {
    if let Some(holder) = &observation.holder {
        println!("{}: {:?}", observation.event.timestamp_us, holder.status());
        // Only observation.update.rows were recorded at this event.
    }
}
```

The iterator walks updates, gap markers and the roster event index in order;
identity state is bounded by live entities. Each player update projects the
latest populations through the existing roster resolver. Gaps, NEW/keyframe
resets and removals clear identity support. A complete bot listing no longer
erases raw slot scores; incomplete human or bot lists still emit the conservative
`IncompleteRosterPopulation` gap. The legacy `UnresolvedBotPopulation` enum value
is retained for compatibility but is no longer emitted.

Missing packet/chunk sequences, unread chunk tails and unframed chunks can hide
population changes. At these boundaries the join forgets both populations;
each domain requires a fresh complete listing before it can establish a unique
holder or known absence. The original gaps remain available on the scoreboard.
Keyframe resets and unread entity suffixes clear entity identity support without
discarding population packets whose source sequence is still established.

These are links to **recorded population observations**, not proof of continuous
ownership between listings. Previously accumulated rows
are not reassigned when a slot changes holder. Initial zeros, equal updates and
duplicate statborg claims are all retained; there is no participant-total merge
or unique-score convenience query in this view. A listing change by itself does
not manufacture a score update. Finalized-round values remain source values
observed at this event; the join does not prove who earned them in earlier rounds.

This follows the recorded relationship used by upstream `holders.rs`,
`gamestate.rs` and `statlog.rs` at
`8f0f8d078e513c800baa21fac416fd4c0c127d03`. First-later-listing,
identity-equals-slot and later-snapshot identity fallbacks are deliberately
excluded. Engine timer roles, score aggregation and final-score coverage remain
separate obligations.
