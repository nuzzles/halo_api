# Bounded zlib transport fixtures

`wire_cases.py` independently uses Python's zlib to encode the 30,720-byte
payload `b'abcde0123456789' * 2048` with fixed and dynamic Huffman codes. It checks
both block types and Python decompression before writing the two tiny streams.
Their long backreferences exercise repeated decoder calls around 8 KiB output
boundaries. No film or player data is included.

`export_reference.py /path/to/libfilmshell` archives upstream commit
`18067987ebc3628ad499c819c6aee814692df43b`, appends only a test caller, and invokes
the unchanged `inflate::zlib`. It verifies full output bytes and TooLong versus
successful completion for 16 stream/limit pairs. No upstream decoder logic is
modified. `reference.json` retains those results; local tests consume it.

Additional independently authored RFC stored-block tests cover all eight window
headers, empty streams, exact limits, 8 KiB boundaries, truncation, checksums and
missing dictionaries. Local limit outcomes intentionally retain a bounded prefix
and original compressed source with `ZlibLimitExceeded { limit }`; upstream
returns TooLong without the prefix. Existing partial-stream source retention is
preserved. The production limit matches upstream's 256 MiB ceiling. Small injected
limits test the same code without allocating a 256 MiB test buffer.

This verifies valid streams and selected malformed/limited inputs. It does not
establish full RFC conformance, packet terminal semantics, or film format coverage.
Original LevelUp fixtures are unchanged.

```sh
python3 src/theater/fixtures/transport-v41/wire_cases.py
python3 src/theater/fixtures/transport-v41/export_reference.py /path/to/libfilmshell > /tmp/transport-reference.json
cargo test --offline --lib transport::compression
```

Review the independent upstream export before replacing `reference.json`.
