# Native translocator zero-tail contract

Pinned LevelUp: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_translocator_padding_test.go.txt`, registered
with the oracle generator. Calls actual decodeTranslocHead and replays its
reader stages to record the logical end offset.

2,048 payloads cover every second-byte value and lengths zero through seven,
with absent/present finite map context. Native accepts 384 heads; 64 have an
incomplete unit reference and 120 traverse synthetic tail bits. Rust compares
native admission, slot, complete-position availability and logical end offsets.
Existing 632 map/profile cases plus captured teleports also exercise the native
entry point for full position values and refusals.

The legacy bounded decoder rejected incomplete references, unlike the native
zero-padding reader. It remains a bounded API. decode_native_translocator_head
now returns the native outcome separately with source_bits and padded_bits.
unit_reference_recorded requires all 20 prefix/reference bits. Padded identities
are native decoder output, not fully recorded actors. Map vectors still fail
when their final bit exceeds the physical payload, exactly at the native guard.
Partial position information before a later refusal remains inspectable; only
positions() returns a usable complete jump.

Film.native_translocations retains these reads with source packet and ordinal,
using native source selection/head admission and native timestamp sorting. All
three constructors and Film JSON roundtrip retain a two-byte accepted native
head while the bounded translocations list remains empty. The map constructor
resolves positions using its supplied map. This does not turn padding into
semantic replay events or begin the deferred architecture.

The production decoder continues to reject nonfinite map bounds as invalid;
normal catalog loading rejects those too. Arbitrary injected NaN map contexts
are not covered by this fixture. Source-error API and captured full-film
translocation publication remain separate audits.
