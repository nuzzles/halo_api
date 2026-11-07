# v41 grenade inventory component evidence

Original decompressed bytes from Oddball match
`4031c1db-2fd5-4a2a-91a1-bed5d0ad3e73`, retained under
`experiments/films/ranked-arena/02-oddball/`. Each excerpt starts at the byte
containing the component's first bit and ends after its last bit. Adjacent bits
are retained to preserve alignment. `evidence.json` records original chunk,
payload byte, bit bounds, full entity ID, clock, raw values and SHA-256.

The existing sequential contextual parser established these component locations;
production does not search for a 35-bit count pattern. The component is R3 count,
then count R8 values. The supported count is four.

The pinned reference is LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`:

- `film/internal/grammar/unit_weaponstate.go`: component byte-count grammar.
- `film/internal/grammar/grenade_events.go`: `GrenadeTypeIDsByRank` order is
  fragmentation, plasma, dynamo, spike. Spike is index 3, not index 2.
- `film/internal/grammar/inventory_decode.go`: `DefaultGrenadeMax = 2`, the
  standard per-type admission bound. Our explicit `grenade_count_max` context
  allows a caller to supply a known custom-mode limit, including zero.

`frag-plasma.bin` carries [2, 1, 0, 0]. The other two excerpts expose suspicious
readings [16, 8, 4, 0] and [144, 8, 8, 0] for the same player entity. They are
retained as raw evidence and rejected by the standard-mode admission bound;
they do not establish those actual inventory quantities. Their upstream decoding
cause remains unresolved. We neither clamp them nor retain the previous inventory.

Tests read all three through the existing component grammar, verify bit ends,
reject truncated inputs, and distinguish raw readings from admitted counts.
Whole-fixture tests check player binding, lifespan, original source ranges and
causal sampling. This is regression evidence, not independent visual validation
of spike or dynamo counts. No nonzero spike observation is recovered in either
bundled fixture; its column mapping is reference-backed and separately tested.
