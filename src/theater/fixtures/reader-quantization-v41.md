# Stateful quantized-vector parity

Native target: grammar.Lecteur.ReadQuantizedVec3 at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

NativeFilmReader::read_quantized_vec3 reads from its current signed cursor. It
uses the native mid-bucket float evaluation order, with a uint64 shift scale
(which is zero for widths >=64). It does not emit component observer callbacks.
The existing bounded read_native_quantized_vec3 helper remains atomic on
truncation; the stateful native method retains consumed axes if a later read
panics. No player action or position interpretation is inferred by this method.

The existing independently generated quantize-v41 fixture checks 4,096 vectors,
float bit patterns (NaNs by classification), and endpoints. A new native harness,
halo_rust_reader_quantization_test.go.txt, adds 72 cases crossing six signed
boundary starts with 12 widths, including 2^32 and u64::MAX. Fifty-seven panic;
all host cases check endpoints and a recovery scalar read after repositioning.
Large native reads are arranged to reach overflow within 128 bit iterations.
WASM checks all 4,096 ordinary cases and the 15 nonpanicking boundary cases;
post-panic recovery cannot be checked with its panic=abort runtime.

The new method was identified by resolving types returned through facade
function signatures, beyond the facade's explicit type aliases. This closes the
missing stateful API, not the remaining component/frame cursor restrictions or
whole-parser acceptance gates.
