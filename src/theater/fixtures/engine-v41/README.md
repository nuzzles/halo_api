# Engine timer conversion oracle

`timer.f32le` contains 65,536 little-endian IEEE754 float32 words, one for each
u16 quantum in order. The output comes from unchanged LevelUp source at
`413971bcc485be53a2fa8cf229e170539a5146f8`:

| Source | Verified Git blob | Extracted definitions |
| --- | --- | --- |
| `quantize_endpoint.go` | `5d7c33a94b6a73012a07d6b74c5da7bfa2469a2c` | `DequantEndpoint`, `RoundTimerMax`, `roundTimerBits` |
| `vitality.go` | `2caa36888f53b767db48bf16b5a391fe7fa4b205` | `RoundTimer`, `decodeGameEngineRoundTimer` |
| `lecteur_minuteur.go` | `dc382adaa1bbfe38e4f8004f094aaf04da636bd6` | `Minuteur`, `lireMinuteur140d580d0`, `largeurQueueMinuteur` |

The exporter supplies bounded MSB-first bit I/O and runs both timer words
through every quantum and all 32 tail states (2,097,152 bodies). It verifies
the exact 37-bit boundary, original raw words and tail. This certifies the
extracted source's conversion, not a run of the game executable or a complete
match-clock interpretation. The source explicitly leaves elapsed/remaining
roles unassigned. Respawn timer integers have no established time unit.

The original quantizer has exact endpoints 0 and 36,000, with interior step
36,000/65,534 and float32 operation ordering. The exporter pins Go to amd64/v1,
which preserves separate scalar rounding. On ARM64 Go contracts multiply/add
into FMA; this produced 21,018 different words, first at quantum 8. These were
investigated before retaining the oracle. No reference function was patched.
ARM macOS needs x86 execution support (Rosetta) to reproduce this fixture.

```sh
GO_BIN=/path/to/go GOCACHE=/writable/cache \
  python3 export_timer_source.py /path/to/pinned/grammar > /tmp/timer.f32le
cmp timer.f32le /tmp/timer.f32le
```

The Rust test compares both projected words for all 65,536 quanta bit for bit.
Separate packet tests exercise default-state identities, phase/round gates,
raw respawn/lives/active values, lifecycle changes, partial baselines and gaps.
The manifest in `docs/fixtures.json` records the binary's length and SHA-256.
