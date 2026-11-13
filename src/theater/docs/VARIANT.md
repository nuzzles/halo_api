# Recorded variant runtime view

> Historical decoder and runtime research. The source-inspection runtime APIs described below were removed by the typed-film refactor; use `Film.replication_stream.observations` and `TheaterRuntime` for the current API.


`TheaterRuntime::variant(VariantSchema::ReferenceLatest)` borrows a named view
of the canonical variant Bond tree. The existing film/parser/runtime split is
unchanged: Film retains syntax and bytes, parser/v41 establishes boundaries,
and this runtime module supplies explicit reference-schema interpretations.
Schema selection cites libfilmshell `a13efb323b230c96b59685defbc376720043240f`;
it is not a claim that the mapping applies to every game build.

```rust
use halo_api::theater::runtime::variant::{Setting, VariantError, VariantSchema};

fn inspect(runtime: &halo_api::theater::TheaterRuntime) -> Result<(), VariantError> {
    let variant = runtime.variant(VariantSchema::ReferenceLatest)?;
    let respawn = variant.setting(Setting::RespawnSeconds)?;
    let options = variant.options()?;
    println!("{respawn:?}: {options:?}");
    Ok(())
}
```

The view requires a v41/27 bootstrap, a complete canonical variant Bond message,
and exactly one selected optional instance. It does not search for strings or
markers. A complete message is usable even if unrelated fields after it are
truncated; this does not relax the canonical parser's separate context policies.
All returned values retain their original BondValue and registry-relative bit
ranges. Neither loading nor querying the view changes Film or the playback cursor.

The current view exposes 24 named settings, ten flag meanings, all loadout/weapon
entries and all option rows. Settings retain signed/unsigned integers, booleans
and exact f32/f64 words. Float units are not converted to libfilmshell's rounded
integer thousandths; for example, SpawnProtection retains the float that its
`spawnProtectionMs` convenience projection scales. Last-chance duration is
retained independently of the enabling setting, rather than presented as active.

`mode_icon()` reads the selected variant header's Bond path `[0, 10]`. A
recorded integer keeps its exact `BondValue` and must fit in `u8`. A complete
header with no icon field returns `SerializerDefaultZero`, labeled as the
reference writer's default rather than a recorded zero; a missing header
returns `None`. Duplicate, wrong-type and negative values refuse the projection.
The latest upstream source locates the header by scanning backward from a
string path; our view uses the already parsed Bond hierarchy. The
[current original-reader oracle](../fixtures/variant-icon-a13efb/README.md)
matches all 32 retained headers: 30 omitted zero defaults, one recorded eight
and one recorded two. The
older f050371 variant fixtures remain historical oracles for unchanged paths;
only the latest schema is selectable.

Loadouts remain separate, with recorded identifiers, weapon tags/random values,
equipment and localized-name indices. Grenade definitions remain available as
canonical syntax. Missing tags are distinct from a recorded zero tag. Option
hashes use the native signed i32 wrapper reinterpreted as u32. Every option keeps
all eight payload bytes: exact f32 or bool interpretation, type discriminator,
and the three trailing bytes. Unknown types retain their raw data. Entries stay
in source order and duplicate hashes remain separate; a hash and value can only
belong together within the same row.

`Ok(None)` means that the queried field/container is absent. A present empty
container is `Some(empty)`. Duplicate field IDs in the same inheritance layer,
wrong shapes and invalid values return explicit errors. Earlier inherited IDs
cannot fill missing derived fields. Recorded complete flag lists/sets can prove
that a named flag is absent; an omitted container remains unknown. Script and
serializer defaults are not inserted into this view.

The implementation follows the named-path evidence in pinned `variant.rs` and
`variant_stats.rs`, with the canonical optional-instance structure already
established by the existing LevelUp variant fixtures. See the independent
[fixture documentation](../fixtures/variant-view-v41/README.md). Corpus validation
currently covers 32 variants: 221 named setting occurrences, 325 option rows,
32 loadouts and 96 weapon entries. Original upstream traces match all 325 option
hashes and eight-byte values in source order and all base-structure endpoints.

## Stat definitions and scoreboard columns

`stat_definitions()` exposes every definition at its original list ordinal,
including the unsigned StatId, exact name bytes, base entry type, aggregation,
sort direction, display type and decimal precision. Numeric wire signedness and
width remain available through the source value. A missing attribute remains
None; a present zero is recorded as zero. Invalid UTF-8, embedded NULs, empty
names and absent names remain distinguishable. No name is hashed to invent an ID.

`scoreboard_columns()` retains original column order and repeated IDs. Its
DefinitionMatch distinguishes a unique slot, duplicate candidates, unmatched
IDs, absent definitions, a missing column ID, and incomplete matching when some
definition IDs are unknown or malformed. Even one known match is insufficient
to claim uniqueness when another definition's ID is unavailable. Shared match
arrays avoid multiplying memory use for repeated columns; indexing avoids a
full definition scan per column. Each row's canonical source remains available.

`StatDefinition::lower_is_better()` projects only sort code -1 as true;
`display_kind()` maps codes 0/1/2 to Number/Time/Percentage and retains other
signed codes. `shown(stored)` uses the current reference precision law for an
explicit i32 stored value, clamping precision to 0–18. All three methods retain
missing and malformed Bond fields as unresolved. The independent
[f050371 presentation oracle](../fixtures/stat-presentation-f050371/README.md)
checks the original display mapping and exact shown-value bits at the i32 and
precision boundaries. The source values remain available unchanged.

`spawn_settings()` exposes the recorded respawn-settings override tag without
inserting a global default. These APIs do not rewrite statborg observations,
aggregate rounds, infer player ownership, or recover final scores.

The expanded corpus comparison covers 1,324 definitions and 130 column references.
Every recorded property and column match agrees with the original upstream
projection. Another 5,876 local fields remain absent where upstream supplies
zero or empty defaults. This distinction is retained in the comparison output.

Individual text-field roles beyond the reference conventions,
script archive/declaration semantics,
actual script/default applicability, time/percentage formatting and broader writer/build
semantics remain incomplete. This view does not claim full variant/runtime parity
or complete 1:1 film-format coverage.

## Indexed localized strings

`localized_strings()` reads the string table at `[3,0]`, with its byte blob at
`[3,0,0,0]` and entry list at `[3,0,1,0]`. Each entry retains its field-0 bool
without assigning it a meaning, plus field-1 signed-i32 offsets in recorded
language order. The generic lookup imposes no fixed language count. `text(entry,
language)` returns exact bytes, the recorded offset value, and separate source
ranges for the text and NUL terminator. Invalid UTF-8, empty text, shared/suffix
offsets, omitted fields and malformed values remain distinct. No preceding-NUL
rule or fallback language is assumed. Accessors borrow cache dependencies
immutably; the terminator index scans bytes monotonically and allocates only
for distinct valid referenced offsets.

`english_name()` and `english_description()` select the first language of rows
0 and 1 under the explicitly selected reference convention. Both agree with the
unchanged upstream text reader on all 32 captured films. This is a convention
validated for this schema/corpus, not a recorded language identifier. Other
string roles are not inferred from row number.

The captured tables contain 1,639 entries and 29,502 localized values. All offset
and flag traces match the original native reader; every returned byte and NUL
range matches the Film source. All 1,323 label texts emitted by upstream occur
in the tables. That initial check established text availability; the linked
reference view below additionally verifies definition-to-text association.
For example, the first stat label is at row 9 in the standard Slayer capture
and row 5 in the raid. The Alpha row is recorded too: all 18 offsets refer to
one shared string. The indexed view needs no special skip for it and does not
treat it as unreferenced text. Neither text scanning nor the upstream fixed
41-label loop establishes canonical boundaries.

## Script filename evidence and inferred mode

`script_data()` exposes the recorded byte array at `[3,2]`. Its internal archive
and bytecode grammar remains opaque. `script_name_candidates()` searches this
bounded data for upstream's slash-prefixed ASCII basenames ending in `.lua`.
The enclosing value's boundaries come entirely from canonical Bond parsing.
Matches are filename candidates, not proof of script declarations, execution,
or dependency loading. No parser admission or engine setting uses these guesses.

The result preserves every occurrence in source order, exact spelling, basename
and preceding-separator bit ranges, and the following byte when present. Its
separate name index supplies the original lowercased alphabetical name set;
case-insensitive lookup returns all occurrences. Missing data, a present empty
blob, a malformed field and an unrecognized filename set stay distinguishable.
The search requires lowercase `.lua` and upstream's ASCII basename characters;
it excludes Unicode/hyphen/dot basenames. Like upstream it accepts a prefix of
`.luac`, so `following_byte` remains explicit. A complete `.lua` at the blob end
is retained even without a following delimiter. Bytes outside the blob cannot
extend its basename or supply its separator.

`infer_mode()` returns reference-choice and all matching rules with their source
evidence. Its precedence follows pinned libfilmshell f050371, including
Attrition/Elimination (`attritionevents.lua`), Extraction (`eevents.lua`) and
Escalation (`gungameevents.lua`); competing rules remain visible. The upstream
classifier groups Attrition and Elimination under one `Attrition` result. When
no rule matches, the choice is None; it does not imply PvP.
Firefight requires its trigger even when the Firefight-KOTH helper appears.
These are hypotheses based on filename presence; engine type and recorded mode
settings remain separate. The view does not apply script defaults.

Independent fixtures run unchanged upstream script/mode code across 272 older
authored cases, including all 256 prior trigger combinations. A separate f050371
oracle checks all 2,048 combinations of the eleven current triggers against its
original classifier. On all 32 captured films the
bounded name sets and mode choices match both the original blob search and the
original whole-registry search: 3,336 per-film distinct names and 4,463 retained
source occurrences. No names were found exclusively outside the blob in this
corpus. The corpus contains Slayer, Strongholds and Oddball choices; other rules
have authored/native-code evidence, not captured-mode validation. Script archive
structure, actual loaded-script/default applicability and full film
format parity remain incomplete.

## Reference option rules

`rules()` borrows a view of all 12 reference Options-script groups and 247 rules
from the sole f050371 catalogue. It retains the original option
rows, every alias in precedence order, matching row ordinals and source values.
Group evidence keeps script filename candidates separate from recorded hashes
whose reference names share the group's prefix. `has_evidence()` reports that
presence only; false also covers unavailable inputs, which remain inspectable.
Neither result establishes script loading or active rule applicability.

The first matching alias has reference precedence. All later matches remain
visible. Case-only aliases share the same match array and do not make a single
row ambiguous. Repeated rows for one alias remain ambiguous even if their bytes
are equal. Missing/malformed values and unknown discriminators prevent evaluation;
they are not skipped to try another alias. A row without a readable identifier
could match any rule, so every override retains shared incomplete-match evidence.
The immutable view keeps row indices and cached matches consistent.

`Rule::evaluate()` distinguishes a recorded value, ReferenceDefault,
NoLiteralDefault and Unresolved. A recorded value retains all eight bytes and
its exact f32 word or bool meaning. A reference default keeps the catalogue's
f64 value or bool and is explicitly conditional on that reference rule applying.
Only a complete present option list with no matching override can reach this
default path. Missing/malformed containers, incomplete IDs, duplicate matches
and unreadable values stay Unresolved. No default enters Film or the ordinary
recorded option view.

Aliases absent from the reference hash/name lookup can still match an override
when inspecting a group. They do not manufacture prefix applicability evidence:
all reference groups remain available even without such evidence. Callers can
inspect unrecognized hashes, raw malformed rows and the catalogue independently.
The view does not infer game-build applicability from a catalogue revision.

The f050371 original-model oracle validates all 486 aliases, all 247 default
metadata entries and reference evaluation on well-formed inputs. Across the 32
films, 465 evidenced rules match upstream: 285 recorded overrides, 60 hypothetical
reference defaults and 120 absent literal defaults. Original decoder outputs are
compared separately to avoid rounding the retained local float words. See the
[rule fixtures](../fixtures/variant-rules-v41/README.md) for reproduction and the
intentional distinctions on missing/ambiguous data. Actual loaded-script/default
applicability and complete film-format parity remain open.

## Recorded stat text references

`StatDefinition::text_index(StatTextField::Field8)` and its Field9/10/11 variants
expose the recorded signed-i32 fields. Their individual label roles are not yet
established, so the API names the wire IDs explicitly. The reference interpretation
as string-table indices is supported by the original numeric reader and by their
agreement with all native labels in the 32-film corpus. It does not establish
short/long/plural/display semantics for any one field or applicability to all builds.

`stat_texts()` binds definitions to the string table of the same selected variant.
`text(slot, field, language)` follows the chosen recorded index; `consensus(slot,
language)` resolves only when all four indices are present, valid and equal.
Distinct indices remain conflicting even when their text bytes happen to agree.
Each result retains its definition, index value(s), string-entry offset value,
exact text bytes and separate NUL range. Missing/ambiguous tables, missing or
wrong-type fields, negative/out-of-range indices and missing languages stay
explicit. No stat ordinal, name, ID, marker search or fixed-label count supplies
a missing index. Missing or duplicate stat IDs do not erase independently readable
text references; association follows the original definition row.

All 5,296 references in 1,324 captured definitions match the independent original
reader. Their 23,832 localized values resolve to the recorded entries, with source
ranges checked against Film. All 1,323 English labels emitted by the native scan
match. One additional association comes from Oddball slots 45 and 46 sharing
entry 55, Carry Time: their distinct definitions, SkullHeldTime and
SkullScoringTicks, remain separate. Equal labels do not establish equal units or
precision. The original scan emits the shared text once, leaving slot 46 unlabelled.
See [stat-text fixtures](../fixtures/stat-text-v41/README.md) for evidence and
reproduction. The view does not establish the remaining field roles, numeric
stat presentation, final-score recovery or complete canonical-format parity.
