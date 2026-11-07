# Controlled desired weapon selection excerpts

These 20-byte excerpts begin at the listed payload byte in the original,
decompressed replication chunks retained under `experiments/films/weapons/`.
They preserve original bit alignment; no decoded JSON supplies test inputs.

| File | Match | Chunk | Payload byte | Component bits | Desired slot |
|---|---|---:|---:|---|---:|
| br-to-shock.bin | 731ef1f7-a119-49ca-a056-4b8305aaf7c6 | 3 | 529423 | 64..71 | 1 |
| shock-to-br.bin | 731ef1f7-a119-49ca-a056-4b8305aaf7c6 | 4 | 501534 | 64..71 | 0 |
| stalker-to-ar.bin | 9ea11b0a-dddc-4061-a7f7-ac22fa62a05b | 4 | 503906 | 79..86 | 0 |

BR/Shock is `04-br-shock-rifle`; AR/Stalker is `05-ar-stalker-rifle`.
The controlled actions and packet coordinates are retained in commit
`ac426a0f286e3f1e2dcbc351c5272856ee2a2492:experiments/FILM_WEAPONS.md`.
These tests validate component consumption and selection interpretation, not
whole-record boundaries or complete replay of those clips. Their coordinates
are test inputs only; production uses the sequential contextual record reader.
The BR/Shock clip currently has no accepted complete biped NEW in the runtime,
so it does not independently validate the entire inventory/export pipeline.

The switch forms are `0010011` and `0010001`. The first R3 stays 1 while the
principal slot changes: it is a request counter, not a selected slot.
LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`,
`apps/go-api/internal/games/halo_infinite/film/internal/grammar/unit_weaponstate.go`,
`lireJeuDArmes` / `FUN_1406d01fc`, establishes principal and off-hand semantics.
The principal is desired; draw animation completion is not in this field.

SHA-256 of each retained excerpt:

- `br-to-shock.bin`: `e5dbf8b78199995e8ca40a00bfe8d715fb74082bcbb5999b1b364478a956a4ee`
- `shock-to-br.bin`: `ce86154ac4834793d858c61f667fc27d3e21eda4fbcc0ffa5ac060cbd878c652`
- `stalker-to-ar.bin`: `ad266b5fd33a17c7c2cceef1cab915799ce7c91b87ea1b64fec545175d02a7ca`
