# Fresh reader lifecycle with seeded traversal worlds

Pinned reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81.
The existing position-hook-inference-v41 and context-resync-frame-v41 oracles
now include two additional lifecycle passes per case. All prior fixture fields
were checked for exact equality and remain unchanged.

Native generators remain TestHaloRustPositionHookInference and
TestHaloRustContextResyncFrame, already registered in generate_oracles.py.
Both passed together in 0.469s. The added passes seed positions on bound slots,
reuse one world and one observer, and create a fresh reader for each call. They
do not install the native reader's optional position accumulator. Inference starts
at the existing fixture's bit 13; raw resync starts at zero and accepts each clean
target candidate. World binding changes persist across calls.

There are 1,024 inference passes and 256 raw-resync passes, containing 4,758 and
194 returned records respectively. Cumulative callbacks across each pair are
copied once per pass. Total deliveries are 1,366 inference positions
(1,210 Absolute,84 DeltaAxis,54 Delta8,18 AbsoluteFallback) and 12 raw-resync
positions (10 Absolute,2 AbsoluteFallback). No Baseline position is published.

Rust checks ordered records and endpoints, inference outcomes, resync acceptance
callbacks, ordered typed position callbacks, complete world snapshots and result
JSON roundtrips after every pass. The checks test the distinction between a
traversal world that contains positions and a reader with accumulation explicitly
attached. They do not claim arbitrary private-reader accumulator injection is
supported by every wrapper, or independently annotate physical gameplay actions.

Source evidence: DecodeFrameInfer, DecodeFrameResync and DecodeFrameViews create
fresh LecteurSur readers. position_capture.go initializes the optional accumulator
to nil and documents that production has no installer. The Rust fresh-reader paths
therefore must not automatically attach the supplied traversal world. Explicit
accumulation on NativeFilmReader.read_frame_records is covered separately by the
generic/shared accumulator oracles.

Both new Rust tests passed in 0.90s. The original inference and resync fixture
tests also passed (0.64s and 0.06s). Clippy passed (30.07s), as did format/diff,
generator syntax and all 485 pinned-source hashes. No parser implementation
change was necessary. Results are tracked in reference/CURRENT_VALIDATION_GAPS.md.
