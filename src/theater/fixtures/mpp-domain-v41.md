# Native MPP width and cursor domain

Reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_mpp_domain_test.go.txt` calls actual native
consumeMultiplayerPropertiesBlock and DecodeFrameRecords. Repeated generation
is byte-identical; no Rust-produced expectation or accelerated native reader
is used.

272 direct cases cross buffers, lead/index settings, signed starts and widths.
There are 212 panics and 60 successful outcomes. The test compares panic status,
reached cursor, ordered MPP publications and reset/read recovery. Very large
active native widths are run only close enough to signed overflow to finish
quickly. Ordinary-start enormous widths are excluded from the executable oracle.

Two complete native frames separately consume a 2^32-bit lead or index field.
The public Rust reader compares completion, final cursor, record identity/type/end,
MPP publications, creation outcome and JSON retention. Field widths are u64;
source prefix coverage and raw bits discarded by the numeric accumulator are
checked directly against source bytes. Synthetic padding is not expanded.
Actual WASM uses these same two frame expectations.

Old ordinary-start negative-width refusal controls now check the mathematically
required overflow panic at i64::MIN on the host. They are explicitly analytical
controls, not claimed as native executed comparisons. See mpp-domain-validation.json.

This does not establish other raw position/profile width domains or higher-level
recovery/source-address contracts, and does not claim complete v41 parser parity.
