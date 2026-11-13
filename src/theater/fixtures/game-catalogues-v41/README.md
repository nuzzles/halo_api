# Medal and option catalogue reference

This independent fixture validates the local `film/catalog/medals` and `film/catalog/options`
catalogues. It does not establish applicability to any recorded game build or
complete named variant/summary interpretation.

The exporter verifies four blobs from libfilmshell
`f050371a6cda83b9887eeb83a1884336fce28e53`, compiles the unchanged source modules
in a temporary standalone Rust harness, and invokes their original public APIs.
It adds no dependencies and does not use halo_api code. The enum lookup is
exercised at all indices 0–65535 and at u32::MAX. It exports all 157 medals,
including native index, enum name, display name, kind, difficulty, personal score
and sorting weight. Index holes remain absent.

The options output retains 98 hash/name entries, 12 mode scripts and their
247 ordered option entries, including every key alias and optional literal
default. Numeric defaults retain their exact f64 bit patterns; booleans and
missing defaults remain distinct. Every name is checked through the original
lookup and lowercased murmur3 hash. Another 261 deterministic binary vectors
exercise the original hash function at lengths 0–260.

Reproduce from the repository root:

```sh
python3 src/theater/fixtures/game-catalogues-v41/export_reference.py /path/to/pinned/libfilmshell > /tmp/game-catalogues-reference.json
cmp src/theater/fixtures/game-catalogues-v41/reference.json /tmp/game-catalogues-reference.json
```

Medal indices are not service medal NameIds. Personal score is not sorting
weight. Script defaults are not recorded Bond values or overrides. The
local catalogue retains explicit source revision/build provenance and these
namespace distinctions. It must not expand empirical summary admission or select
wire layouts merely because a catalogue lookup succeeds.

Reproduce the adapted local data separately:

```sh
python3 src/theater/fixtures/game-catalogues-v41/generate_data.py /path/to/pinned/libfilmshell
```

An optional second argument writes to an alternate film directory for comparison.
The importer verifies the two original data blobs and formats the adapted Rust
files. It does not use this JSON fixture to generate the implementation. Tests
compare all local metadata/lookup results against the independent export and
check the existing runtime empirical admission boundary. See
[GAME_CATALOGUES.md](../../docs/GAME_CATALOGUES.md).
