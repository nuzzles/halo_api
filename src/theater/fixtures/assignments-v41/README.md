# Listing-derived engine-player assignments

Semantic reference: libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`.
`export_reference.py` verifies the exact Git blobs for `roster.rs` and `bits.rs`
before compiling the unchanged original `struct_players` / `bot_records` readers.
The wrapper only exposes their outputs; container/identity/highlight stubs satisfy
unused traversal code. Existing fixtures keep their earlier upstream pins.

`cases.py` independently authors eight v41/27 HI_1_13_0 packet bodies. They cover
all 32 present session-option values, all 64 scanner-admitted bot slot values,
absent/present session options, duplicate engine claims and empty populations.
The 117 records reach every entry bit alignment. All 108 present fields agree
with the native reader. Eight absent session options stay absent locally, while
the native scanner reads unrelated preceding bits as a number. Its output is
retained as evidence of that difference, not adopted as a canonical identifier.
An additional complete record has first short value 21 rather than the scanner
marker 183; the original scanner omits it, while the canonical reader retains
its explicit assignment. This reproduces a captured-film difference.

Rust tests also cover the full 32-bit bot slot without narrowing, source-field
identity, unsupported semantic formats, partial/opaque/duplicate populations,
independent human/bot domains, departures, same-time reuse, reappearance, repeated
listings, missing IDs, cross-domain and same-domain competing claims. These are
local source-preserving policies, not claims of parity with `holders::api`'s
first-later/index fallback. Canonical Film and playback remain unchanged.

Reproduce from this directory:

```sh
python3 cases.py
python3 export_reference.py /path/to/libfilmshell-8f0f8d0 > /tmp/assignments-reference.json
cmp reference.json /tmp/assignments-reference.json
```

For retained films, the existing ignored metadata test now also exports runtime
assignments from the actual listing events:

```sh
HALO_ROSTER_EXPORT=/tmp/assignments-captured.json cargo test --offline --config build.incremental=false captured_player_metadata_preserves_fields_and_packet_boundaries -- --ignored --nocapture --test-threads=1
python3 export_reference.py /path/to/libfilmshell-8f0f8d0 /tmp/assignments-captured.json > /tmp/assignments-native.json
python3 verify_captured.py /tmp/assignments-captured.json /tmp/assignments-native.json
```

The captured corpus has only empty bot populations. Nonempty bots, omitted
session options and conflict/departure policies have authored evidence. Listing
associations do not by themselves prove controller ownership, bootstrap-index
identity, attachment, or continuous lifetimes between observations. Semantic
admission is restricted to v41/27; wider metadata wire-reader admission remains
separate.


The 32-film run retains 5,347 human assignments across 509 session packets,
plus 509 empty bot packets. All 5,333 assignments emitted by the original native
reader agree. It omits 14 recorded entries whose first short is 21, not its
required marker 183. `captured-comparison.json` records source packet names,
entry/short bit ranges and payload SHA-256s for those omissions. These are not
14 additional distinct players. The optional third argument to
`verify_captured.py` checks the retained original LevelUp corpus output:

```sh
python3 verify_captured.py /tmp/assignments-captured.json /tmp/assignments-native.json /tmp/roster-corpus-go.json > /tmp/captured-comparison.json
cmp captured-comparison.json /tmp/captured-comparison.json
```

Before using that retained output, the verifier requires identical packet bytes,
format and build. All 5,347 identities/options agree with the original count-driven
Go reader's output. The Go reader was not rerun at this checkpoint; its pinned
provenance and reproduction command remain in roster-v41/README.md. The verifier
also checks each omitted record's identity, join key, option and short directly
against payload bits. No canonical record is discarded to match the scanner.
