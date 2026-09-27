# Native record-ID widths and slot arithmetic

Reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_record_id_policy_test.go.txt.
Fixture: record-id-policy-v41.json.zlib (8,640 cases).

Native readRecordID consumes the low field only for positive IDLowBits. It
truncates that read to uint32, adds IDBase with uint32 wrapping, masks the slot
to 30 bits, and ORs the separately read two-bit generation into bits 30 and 31.
End never reads the ID. Nonpositive widths therefore have a different policy
from the signed-to-unsigned MPP width conversion.

The oracle varies 16 signed widths (-128, -1, i64::MIN, 0, 1, 5, 30, 31, 32,
33, 64, 65, 130, 4097, 2^32, i64::MAX), five bases (0, 1, 0x3fffffff,
0x40000000, 0xffffffff), five prefix forms (End, New, Delete and both Delta
encodings), eight alignments and three low values. Consumed positive widths
above 4097 are not run in Go. There are 1,920 End cases, 2,400 nonpositive-width
cases and 3,360 consumed widths above 30. These groups overlap.

Each case independently records the header kind/ID/endpoint, then reruns the
native complete frame loop with known empty-component archetypes, disabled New
default deserialization and a seeded world. Ordered records, masks, completion,
frame endpoints and all world slots are retained. Rust compares those values,
including generation/base overflow, and roundtrips complete returned views.
Native generation passed in 0.448s. The two focused Rust tests passed in 0.71s.

FrameEncoding.native_id_low_bits carries the original signed setting through
native contextual setup; None retains the checked legacy header contract.
Native pre-reads in inference, production, resync, chain and harvest use the
same header policy. NativeFrameConfig's explicit legacy frame adapter and the
bounded public decode_record_header helper remain checked. Native march still
uses that legacy adapter; its admission and calibration export need separate
reconciliation before claiming all native entry points are migrated.

Explicit Rust-domain controls cover a reached ID exceeding the signed/platform
cursor domain and a generation read after an ID reaches the maximum cursor.
They run with and without the optional 32-bit prefix. Generic readers return
the width error at the reached cursor; inference/resync preserve structured
width refusals. ProductionFrame.header_diagnostics retains refusals before any
complete EntityRecord exists. JSON roundtrips and old-export absence are checked.
These controls do not claim native huge-width execution or signed-overflow
continuation. The actual WASM runner includes the same oracle and controls.

The broader parity audit and NEXT_PHASE.md remain unchanged in scope. No new
canonical hierarchy, resolved-event model or playback refactor is introduced.

Actual wasm32 execution passes all 8,640 new native cases and the four additional
ID-domain controls. The combined runner passes 14,256 native comparisons plus
80 existing address-domain cases, with additional MPP/ID controls counted
separately. WASM build: 21.95s. Clippy passes (20.27s), Python syntax checks pass,
and all 485 pinned source hashes match. The existing copied-test dead-code
warning in the WASM runner is unchanged.
