# Native march lazy policy and calibration availability

Reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_march_lazy_policy_test.go.txt.
Fixture: march-lazy-policy-v41.json.zlib (90 cases).

The harness calls native calibrateFrameConfig, marchStartOf, marchRecordsOf and
objectDeathHarvest using the same assembly as ScanMarchFacts, with explicit
settings. It varies ID width/base, MPP lead, MPP index, New default width and
world-index width independently over -1, 0, 1, 5, 33, 65, 4097, 2^32 and i64::MAX.
Two payload modes exercise packets with and without the event-list marker.
The remaining payload consists of End prefixes, so no unused width may reject
setup, and no death/occupancy fact may be invented. Calibration still executes
all six candidate widths and event localization before retaining its default.
Native generation passed in 0.366s.

Rust compares all march fact vectors, coverage maps, packet counts, retained
calibration/default/candidate counters, native scalar settings and full original
configuration. Full FilmMarchFacts JSON roundtrips are checked. The focused test
passes all 90 cases (0.12s). This is deliberate negative/unused-setting coverage,
not evidence of positive deaths, rides, or unusual-width captured films.

Native march locator setup now uses the contextual frame encoding, whose actual
reads carry raw native settings. Calibration export no longer fails a completed
scan solely because its checked convenience encoding cannot represent those
settings. MarchCalibration.encoding is now Option<FrameEncoding>; on conversion
failure it is None and encoding_error retains the conversion error. Complete
settings remain in FilmMarchFacts.native_config. No default substitute encoding
is exported. Rust consumers must handle the optional encoding. Existing JSON
with an encoding object still deserializes as Some, and ordinary successful
exports keep the same JSON shape because encoding_error is omitted when absent.
Legacy encoding-only calibration always returns Some of its supplied encoding.

The preceding record-ID oracle also now checks 1,440 legacy-compatible cases
through native generic and inference entry points. These paths use wrapping
native ID arithmetic even without native_id_low_bits. The bounded public header
helper remains checked; explicit overflow controls verify that distinction.
This extension reuses the original independent Go fixture without regenerating
expectations. The focused ID tests pass (0.73s).

This closes these adapter issues, not the full parser goal. Negative/overflowed
cursor continuation, wider native-output reconciliation and remaining corpus
acceptance stay open. The architecture in NEXT_PHASE.md remains deferred.

Actual wasm32 execution passes the same 90 march cases and the 1,440 legacy-ID
extensions. The combined runner reports 14,346 native fixture comparisons plus
80 existing address-domain cases; supplementary legacy-ID and MPP/ID controls
are not added to that fixture count. WASM build: 18.85s. The checked calibration
projection is available in 30 host cases and 26 wasm32 cases; all other cases
retain explicit absence/error plus the complete native configuration. Clippy
passes (17.09s), Python syntax and all 485 pinned hashes pass.

Full Theater suite: 579 passed, zero failed, 48 ignored (85.73s). The four-film
captured native-context march comparison also passes (19.83s after a 2m56s
release build), checking complete native facts, coverage, configuration and
calibration counters. Other captured pipelines were not rerun at this checkpoint.
