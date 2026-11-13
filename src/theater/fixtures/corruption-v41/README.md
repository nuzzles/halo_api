# Recorded corruption guards

`reference.json` contains 44 cases exported through unchanged guard functions
from pinned LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`. The exporter
checks six Git blobs and uses the original bit reader. It extracts
`lireUnMessage`, `descripteurDuGenre`, `chargeZoom`, `consumeCorruptionCheck`,
and `consumeFullStateDefaultBlock`, plus the original descriptor table.

The 32 event cases cover all 13 empty descriptors, zoom with both flag values,
both guard values, four sentinel words, an empty successor and a terminator.
Four component cases isolate the optional R1/R32 envelope. Eight default cases
isolate signed n1, unconditional R32 when enabled and positive, then n2=1.
The default-state callback is an explicitly zero-bit ti0 stub; Rust uses the
equivalent zero-bit ti4 default to also exercise its high-frequency component;
reference-handle callbacks panic if reached. These fixtures do not independently
certify component/default body semantics or present-reference decoding.

Rust tests compare endpoints and successor boundaries, retain guard values,
and exhaust every event-prefix and component-guard truncation with byte-aligned
source ends. A public Film/replay test uses the recognized registry plus existing
identification/options fixtures and 32 explicitly vacant bootstrap player slots.
It traverses a full native snapshot table, a zoom event and NEW/DELTA high-frequency
components with the recorded flag both enabled and disabled. Runtime action
projection excludes corruption fields from payload and unnamed body tails.

Reproduce with:

```sh
GO_BIN=/path/to/go GOCACHE=/tmp/halo-go-cache python3 export_source.py /path/to/pinned/grammar > reference.json
```

The native readers consume arbitrary sentinel words; the parser preserves those
words and does not introduce an equality check. Missing recorded flags remain
`None` through the bootstrap accessor. Public Film and replay now stop at the
first boundary that needs the unknown policy. Additional tests compare these
retained event prefixes with the original-source fixture, verify no snapshot
size/component word is consumed after an unknown guard, and exercise truncated
and unsupported bootstrap layouts through Film::parse. Empty events, empty
component masks, vacant slots and n1/n2=0 remain readable. NEW schema declarations
survive a guard-context stop so later empty DELTAs can still be read.

The runtime scoreboard's arbitrary synthetic registries now use explicitly
configured test readers. Those fixtures verify projection/grammar behavior, not
public admission of invented bootstrap layouts. Public event fixtures instead
supply the recognized registry and recorded false flag. A replay-admission test
supplies an explicit empty test roster solely to isolate guard policy from roster
discovery; it verifies that unknown policy produces a missing-context blocker,
while recorded false and true both close correctly. No inferred roster becomes
canonical evidence in that test.
