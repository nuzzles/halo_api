# Captured dynamic-position branch comparison

Two 160-bit snippets are copied from the retained raid film's type-0 packet
payloads, at the payload-relative component starts recorded in `cases.json`.
`verify_captured.py` checks the full chunk hashes and re-extracts both snippets.
The source film has recorded build 269225 and branch `HI_1_13_0`; the snippets
are evidence for those exact bytes, not universal writer admission.

The independent `reference.json` runs unchanged libfilmshell component dispatch
at `f050371a6cda83b9887eeb83a1884336fce28e53` with only primitive tracing
added in a disposable archive. Its source is `src/components.rs` blob
`9e7f58f027e8ed38c3aa61ebaedf97871fd3d18a`. The exporter validates its
captured primitive extents and can regenerate the file. The existing local
reader differs deliberately:

| Prefix | Native-source local read | Latest libfilmshell read |
| --- | --- | --- |
| `11`, chunk 140 | 99-bit keep-baseline body | Stops after 2 bits |
| `10`, chunk 163 | 99-bit keep-baseline body | 101-bit raw float body |

The original LevelUp `components_position_i0.go`, pinned at
`413971bcc485be53a2fa8cf229e170539a5146f8`, names those first bits
`bUsePred` and `bDelta`. Its `bUsePred=1` path reads the third handle gate,
three raw words as a baseline copy, then the gated handle tail regardless of
`bDelta`. In the two snippets the handle gate is clear. The 96 source bits
are retained, but their numeric interpretation as a new world position is
not established. The current local read matches that source sequence.
This comparison does not prove every later record in either packet or every
position branch; it prevents the new upstream raw/relative labels from silently
replacing a separately supported native layout.

A prior corpus inventory found 2,534,142 complete ordinary `00` reads, 21 complete and one truncated
`01`, 14 complete `10`, 14 complete `11`, and 4,990 complete snapshot `00`
reads across 32 films. The non-`00` prefixes all occur in the raid film.
These are counts of reached local components, not independently proved whole-
packet boundaries. The two captured snippets are representative evidence,
not a 50-case endpoint oracle.

```sh
python3 src/theater/fixtures/position-prefix-f050371-v41/verify_captured.py
python3 src/theater/fixtures/position-prefix-f050371-v41/export_reference.py /path/to/libfilmshell > /tmp/position-prefix-reference.json
cmp /tmp/position-prefix-reference.json src/theater/fixtures/position-prefix-f050371-v41/reference.json
cargo test --all-features captured_prediction_prefixes_follow_native_reader_without_upstream_raw_relabeling
```
