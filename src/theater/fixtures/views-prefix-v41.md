# Truncated message and control views

Pinned LevelUp: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
The existing `halo_rust_views_test.go.txt::TestHaloRustViewsParity` harness now
runs actual consumeVueA/consumeVueC on every byte prefix of its 2,048 input
records. All original fixture fields were compared unchanged before replacing
`views-levelup-v41.json.zlib`; only the `prefixes` arrays are new. The existing
oracle generator registration handles regeneration.

There are 5,060 prefix cases, including 2,583 control prefixes and 11 native
endpoints that consume synthetic tail bits. The native analog-pair decoder
checks 13 available bits before consuming either six-bit value or the third
analog presence bit. Rust previously advanced into that group on truncated
input; the new assertion failed at start 4, four bytes, Rust end 28 versus
native end 22.

The action block uses a different contract: after guarding its first bit, it
runs the zero-tail component reader, then checks the endpoint against the
source length. Rust now scopes padded reading to that block. DecodedFrameView
retains padded_bits explicitly (omitted when zero, defaulting to zero for old
exports). Such a view remains Truncated. The test compares native endpoints,
completion and ordered kinds, every retained field against source/padded bits,
field continuity, padding counts and JSON roundtrips.

This is native view-decoding parity, not a claim that padding is recorded data
or that the reference's unsupported control/message bodies are decoded.

## Production-frame integration

Each of the 2,583 control prefixes additionally runs through actual native
DecodeFrameViewsCurseur. A zero byte is prepended, unused leading bits are
cleared, and PacketPreambleBits is adjusted to put an empty message view and
entity End immediately before the original control start. This preserves each
control input's original bit alignment and available tail bits. Very short
inputs also exercise early termination before the control view is reached.

The world starts in view 2 with slot 50 bound to archetype 0 and position
[1,2,3]. The oracle retains completed-view counts, endpoint, current world view
and all native slot fields. Rust's NativeFrameConfig.decode_production_views
compares those outputs, absence of fabricated entity records, complete seeded
world preservation apart from the native current-view change, frame/control
padding agreement, truncation and ProductionFrame JSON roundtrips. Earlier
standalone expectations were compared unchanged before fixture replacement.

The control-action helpers at the reference pin use fixed widths here; they do
not use the frame profile's component override maps. These cases exercise
production view composition, not the separate entity-body profile callbacks.

Frame padding includes any padded entity header consumed before entering the
control view. The integration test therefore adds that earlier extent to the
control's own padded_bits when checking the total. An initial assertion wrongly
required equality of the two padding counts; the oracle endpoint and world
comparisons had passed at that case. Correcting the assertion required no
production parser change.

## Film constructor and export

The final extension supplies 1,559 nonempty control prefixes framed with the
standard two-bit preamble and an entity End. Zero-body kind-3 control records
preserve the original bit alignment, so all 11 padded control outcomes remain
present. Actual native DecodeFrameViewsCurseur and consumeVueC supply the frame
endpoint, completed-view count, control kinds and completion flag. Every prior
fixture field was verified unchanged before adding these `film` expectations.

The Rust test assembles these synthetic frames into one packet stream alongside
the captured v41 bootstrap and calls Film.try_from_chunks_with_encoding. It
checks each packet's chunk identity, timestamp, byte offset and payload size,
control outcome, explicit padding, exact unparsed tail and the entire Film JSON
roundtrip. This is constructed parser-integration evidence, not an independently
annotated gameplay recording or a semantic-action golden film.
