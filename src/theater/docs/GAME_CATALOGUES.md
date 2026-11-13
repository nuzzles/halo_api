# Medal and option reference catalogues

`film/catalog/items`, `film/catalog/medals`, and `film/catalog/options` use one
bundled reference catalogue, adapted from OpenSpartan/libfilmshell. The data
does not establish an applicable game build. Looking up reference
metadata does not select a parser layout, establish writer compatibility, or
alter canonical Film or replay.

```rust
use halo_api::theater::film::catalog::{Catalog, medals::MedalIndex, options::OptionHash};

let catalog = Catalog;
let medal = catalog.medal(MedalIndex(0)).unwrap();
assert_eq!(medal.name(), "Double Kill");
assert_eq!(medal.personal_score(), 50);
assert_eq!(medal.sorting_weight(), 100);
assert!(catalog.medal(MedalIndex(14)).is_none());

let hash = OptionHash::from_name("SlayerOptions_scorePerKill");
assert_eq!(catalog.option_name(hash), Some("SlayerOptions_scorePerKill"));
```

## Medals

All 157 native definitions retain index, enum identity, English name, category,
difficulty, personal score and sorting weight. `medals()` visits index order
and omits holes. `MedalIndex` can represent any u32 without truncation; lookup
returns None for unknown indices. This namespace is distinct from Halo service
NameIds. The metadata enum is also separate from `film::summary::medals::Medal`, whose
124 empirical summary pairs remain the only admitted identities in that API.

Every existing empirical pair agrees with the native sorting-weight/index pair.
The 33 additional native definitions are available as reference metadata, without
expanding summary admission. Reference personal score is not observed player score
or sorting weight; using it as the summary type code misidentifies even Double Kill.

## Options

The original 98 hash/name entries, 12 scripts and 247 ordered option entries
retain all aliases and optional literal defaults. `OptionHash::from_name` applies
ASCII lowercase then Murmur3 x86 32-bit seed zero; `options::murmur3` hashes arbitrary
bytes verbatim. Unknown hashes remain representable and lookups return None.

`mode_options()` returns reference script definitions. Numeric defaults retain
their original f64 bits, boolean defaults remain boolean, and missing/nonliteral
defaults stay None. Defaults do not populate or override recorded Bond messages.
The parser's registry game-options envelopes and Bond values remain the source
of recorded data. Named variant interpretation still
needs verified Bond paths and applicable writer/build context.

The native hash/name table does not contain every script alias. The local lookup
preserves this behavior; all alias strings remain available in `ModeOption::keys`.
It does not silently add new hash resolution or merge conflicting option entries.

## Reproduction and validation

[The independent fixture](../fixtures/game-catalogues-v41/README.md) executes
unchanged original Rust APIs after checking all four source blobs. Local tests
compare every metadata field, every alias in order, and exact numeric default
bits. Medal lookup is checked at 0–65535 and u32::MAX, including absent indices.
The original 261 binary hash vectors cover lengths 0–260. Runtime tests check all
65,536 byte-sized summary pairs, confirm the same 124 admitted pairs and retain
33 additional catalogue-only definitions.

The separate `generate_data.py` importer verifies the two original data blobs
and adapts module paths, serde derives and single-catalogue lookup.
It does not generate the independent fixture from local code.

This closes reference metadata parity for these two pinned catalogue surfaces.
It does not establish full film-format coverage, named variant/runtime parity,
recorded scoring,.
