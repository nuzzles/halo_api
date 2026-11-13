# Managed-player creation participant

`wire_cases.py` independently authors all 64 six-bit values at all eight bit
alignments, with both absent and present version words: 1,024 creation bodies.
Each includes the following six-bit secondary word, one-bit flag and a distinct
`0xd6` successor byte. The original OpenSpartan/libfilmshell
`creation::managed_player_participant` at
`f050371a6cda83b9887eeb83a1884336fce28e53` supplies `reference.json`. It
admits exactly values 0–31 and returns `None` for 32–63. No field is inferred
from a record index, roster position or later event.

`export_reference.py` archives the exact revision into a disposable directory,
adds only a test harness calling the unchanged upstream function, and resolves
cached dependencies only in that archive. The original six-bit word remains in
canonical `DefaultState.fields` even when the typed participant accessor refuses
it. Rust tests check all 1,024 source words and following-byte boundaries, plus
complete NEW and keyframe records at values 0, 31, 32 and 63 in both version
forms. Partial records cannot publish a participant.

The older `creation-v41` fixture remains unchanged. Its four type-9 bodies use
raw participant value 51; previously replay could treat that as a player ID.
The new accessor and replay reject it while keeping the raw field.

Reproduce from the repository root:

```sh
python3 src/theater/fixtures/managed-participant-v41/wire_cases.py
python3 src/theater/fixtures/managed-participant-v41/export_reference.py /path/to/libfilmshell > /tmp/managed-participant-reference.json
cmp /tmp/managed-participant-reference.json src/theater/fixtures/managed-participant-v41/reference.json
```

This proves the pinned
function's value admission and the local creation boundary on authored bodies;
it does not establish a managed object's lifetime owner by itself.
