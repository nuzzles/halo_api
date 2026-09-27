# Translocator native source scanning

Pinned reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_translocator_source_test.go.txt`, registered
in generate_oracles.py.

128 synthetic source cases call actual ScanTranslocatorTeleports. A wrapper
retains chunk number, packet ordinal and payload byte range, uses the same
native decoder/sort and must project exactly to the uninstrumented scan output.
Rust compares all 1,000 admitted events in exact order, including source ranges,
wire timestamps, padded identities and complete default-box positions. There
are 32 empty-source controls. Complete results roundtrip through JSON.

Inputs cover contiguous, missing, duplicate, reordered and negative chunk
numbers; filtered packet kinds; one-byte heads; two-byte padded references;
config/family and absent-reference rejection; complete positions; type-7 chunk
termination; zero-size nonterminal rejection; oversized payload declarations;
trailing partial headers; descending/interleaved timestamps and ties. The source
manifest start deliberately differs from wire time. Tied events retain the
reference sort's actual ordering rather than assuming stable sorting.

This is source scanner parity, complementary to the direct-head and constructor
fixtures. It does not establish independently observed gameplay actions. No
production scanner changes were needed for these cases.
