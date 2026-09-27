# Stateful native reader cursor continuation

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
`halo_rust_reader_cursor_test.go.txt` calls grammar.LecteurSur directly. The
registered generator emits 78 cases: six signed starting positions crossed with
12 unsigned widths, plus one signed-variable read per starting position. Widths
include 2^32 and u64::MAX. Large reads start close to signed overflow, so the
native bit loop reaches its panic in at most 128 iterations. This fixture does
not execute billions of successful zero-padding iterations.

Each case retains the read value or panic outcome, endpoint, remaining bits, and
a subsequent SetBitPos(0)/ReadBits(16) recovery. Fifty cases panic in Go. The host
Rust comparison checks all 78, including cursor state after caught panics.
WASM checks the 28 nonpanicking cases; its aborting panic runtime cannot test
recovery after a panic.

The existing source-bits-v41 fixture supplies a second independent comparison:
1,512 sequences / 13,608 operations through NativeFilmReader, with profile and
capture-slot preservation checked at each operation. WASM executes 4,896 actual
prefix operations before each sequence's first native panic, without injecting
expected state to simulate recovery. NativeFilmBits retains its own original
fixture comparison.

The grammar reader now owns NativeFilmBits rather than an independent unsigned
cursor. Native setters, signed skips, remaining counts and direct scalar or
variable-width reads share signed 64-bit wrapping semantics on both targets.
NativeFilmBits::read_wide and NativeFilmReader::read_bits_wide expose the pinned
Go uint64 width domain; no address-sized conversion narrows it on wasm32.

Existing address-sized bit_position() is a checked projection and can panic if
the cursor is negative or outside usize. Use native_bit_position() to retain the
exact signed position. Existing read_bits/skip Option results remain Some for
successful native operations; they no longer return None solely for cursor
arithmetic overflow. Invalid native reads panic as described above.

Component/frame traversal still adapts its starting position to usize and
returns a typed signed-bit-cursor refusal if it cannot do so. Live width-adjustment
negative/overflow endpoints inside traversal remain a separate open gate. This
change does not claim those paths or the broader all-data parity goal complete.
