# Native structure and fidelity

`Film` owns the supplied transport chunks, decompressed chunk data, manifest
metadata, bootstrap reads and ordered packet reads. Packet bodies preserve
views, records, component fields, summary records, stops and diagnostics. Native
IDs, generations, masks and quantized values are retained in the relevant reads.

The bootstrap begins with version words and a registry of archetypes and named
component slots. The captured v41 corpus has 50 accepted registry blocks and
1,067 slots. Registry reads retain the rejected terminal attempt as well as the
accepted entries. Identity and player-table reads are separate bootstrap sections;
heuristically selected player slots do not establish a canonical partition of
all bootstrap bytes.

Replication packets use 16-byte headers. The packet walker records header and
payload offsets and packet time. Nested bit offsets are relative to that packet's
payload, not the compressed chunk. Source positions refer to supplied chunk order;
manifest numbers remain separate metadata. The unwalked chunk suffix begins at
`packet_walk_end_byte`. Bootstrap chunks are not partitioned as replication packets.

Within payloads, view readers walk ordered NEW, UPDATE, DELETE and End records.
Registry-selected component readers consume masks and fields with version-specific
widths. Traversal must establish an actual terminator; payload length or padding
alone cannot prove a record chain complete. Independent event-head, fire, damage,
roster and footer reads retain their own boundaries and refusal status. They can
overlap the generic body and do not replace it or imply a single complete partition.

Some reference readers support padded lookahead. Their synthetic bits, partial
fields and read refusals remain distinguishable from backed source bits. Keyframe
anchor recovery and whole-chunk highlight scans are explicitly candidate reads;
they are separate from admitted sequential records. Unknown component layouts,
opaque bodies and gaps retain their source bytes. Absence of a decoded observation
is not evidence that an action did not happen.

## Meaning of faithful

The contract is preservation of all supplied source information plus ordered,
source-addressable native decoding where supported. The full bytes are retained
even where a typed schema is unknown. The structure is an incremental decoding
report, including overlapping reference reads and heuristic candidates; it does
not claim that every source bit already has a unique typed field. Parser settings
and supplied map bounds are labeled separately from recorded values.

Original transport bytes can be retrieved unchanged. Editing the typed structure
and byte-for-byte re-encoding it is **not implemented**. JSON is an inspection
format, not a replacement for native byte identity.

Scope is v41 behavior supported by the pinned LevelUp reader. Newer film versions,
map assets, renderer geometry, inferred physical actions and external catalog
publication are outside this module's parser contract. Quantization context may
require caller inputs; missing context is not silently invented.
