# Control-list resource-boundary fixtures

`wire_cases.py` independently authors 24 terminated control lists: lengths
0, 1, 63, 64, 65 and 128, each using kind 0, kind 1, kind 2 or mixed kinds.
Optional input/command/secondary bodies are absent so the fixtures isolate the
list boundary rather than gameplay or reference state. Player indices cycle
within their recorded five-bit width; repeated indices are retained.

`reference.json` executes the original `control_view` at libfilmshell
`18067987ebc3628ad499c819c6aee814692df43b`. The existing controls exporter adds
read/skip tracing only in a disposable checkout and verifies every authored
endpoint. An optional case-file argument selects this suite. Removing the
disposable Cargo.lock permits offline resolution of cached dependencies without
altering either source checkout or any original reader. The original 17-case
control oracle still regenerates byte-identically.

The native-source description in LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`,
`frame_vue_controle.go` (blob `2a8354f18262bc3c01b36415b9e279ab3512ebb7`), places
the continuation read before the runtime budget check. It explicitly calls its
64-iteration offline cap a safety policy, not a grammar width; the game's budget
is an external `param_4`. That description is source-analysis evidence, not a
newly executed native reader oracle. Kind 3's separate source disagreement is
unchanged and is not part of this suite.

Local parsing retains a 64-record resource budget. At the boundary it reads the
continuation bit: zero closes, a missing bit is truncated, and one reports
`RecordLimit` before consuming the next kind/body. Upstream closes all 65/128
lists too; those local refusals remain explicit policy differences. No format
maximum of 64 is claimed.

Tests compare every retained primitive boundary at all eight bit alignments,
retain the complete first 64 records on budget refusal, and truncate immediately
before the terminating bit without exposing byte-padding zeros. Public Film
tests verify source coverage, unread suffixes, serialization and runtime access
to the complete record prefix. No captured occurrence of 64+ records is assumed.

The 32-film source audit at this checkpoint passes across 403,465 frame packets.
Its largest decoded control view has 20 records, with zero budget stops. These
captures verify source preservation; the authored fixtures provide positive
evidence for the changed boundary.

```sh
python3 src/theater/fixtures/control-limit-v41/wire_cases.py
python3 src/theater/fixtures/controls-v41/export_reference.py /path/to/libfilmshell src/theater/fixtures/control-limit-v41/cases.json > /tmp/control-limit-reference.json
cmp /tmp/control-limit-reference.json src/theater/fixtures/control-limit-v41/reference.json
```
