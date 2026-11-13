# Movement-input reference cases

`export_reference.py` executes unchanged OpenSpartan/libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03` `components::control_view` and the
complete `src/control.rs` reader (blob
`59aeb85cbfec1c3fee6821e58d7909e02dec1ea0`).

The 57 authored controls cover an absent analog block, eight analog vectors
(including neutral and code 63), all 32 stance words, and eight combinations of
the optional 2/5/6-bit fields with absent/present baseline 127. Each case is put
in two identical type-0 envelopes. A `Payload.end` adapter supplies the control
body position; no entity traversal is synthesized. The upstream reader reduces
these repeats to one series value. Local history deliberately retains both.

The exporter archives the pinned source into a disposable directory, adds only
the harness, and resolves cached dependencies offline after removing that
directory's lockfile. The source lock references unavailable cached itoa 1.0.19;
the verified run used serde_json 1.0.154 and itoa 1.0.18. Original reader code is
unchanged. This tests control syntax/reference projection, not native preceding
frame traversal or the semantics of auxiliary words.

```sh
python3 src/theater/fixtures/inputs-v41/export_reference.py /path/to/libfilmshell > /tmp/input-reference.json
cmp src/theater/fixtures/inputs-v41/reference.json /tmp/input-reference.json
```

The ordinary local suite also authors complete frames and roster packets using
the existing fixture profile, checking gap and holder policies independently.
The captured replay audit checks exact source bits and links. See
../../docs/INPUTS.md for interpretation limits.
