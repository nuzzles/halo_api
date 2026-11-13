# Shared captured snapshot refusal

The type-2 payload at decompressed offset 343051 in Recharge Oddball
`chunk-057-type-2.bin` reaches biped slot 755 at bit 226358. The original pinned
libfilmshell walk reaches 756 physical headers /129 live records and refuses
component 7, `object-damage-sections-component`: its six-bit count is 39, beyond
the reader's 32-section limit. Its component trace consumes that count through
bit 227003. The outer upstream walk reports the component-state start on failure;
the local record retains the consumed prefix end.

The exporter extracts the original pinned commit and adds only primitive trace
callbacks and test serialization. It asserts that no vehicle occurs in this
walk, so no vehicle-layout probing can have supplied the boundary. The expected
component starts, ends and bit traces come exclusively from upstream. The local
test compares source-field coverage, refusal and the retained count; it does not
claim that the earlier state values or whole snapshot are semantically valid.

```sh
python3 export_reference.py /path/to/libfilmshell /path/to/experiments/films
```

`reference.json` retains input hashes and provenance; `packet.bin` is the exact
captured payload. Neither reader's refusal is replaced with a skip or a scan.
