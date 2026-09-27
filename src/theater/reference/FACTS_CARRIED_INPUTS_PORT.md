# Cached carried items and projectile publication

The cache-facing loadout, grenade-read, projectile and grenade builders reuse the
recording path's pure reductions. They consume exact cache DTOs, without creating
synthetic recording records or source byte/bit coordinates.

`build_facts_replay_loadouts` retains ordering and native family alias folding.
`build_facts_replay_grenade_reads` preserves signed i64 selection ranks. Keyframes
publish only nonnegative ranks; delta selections publish whenever read and not -1.
A present-empty delta count list produces a read with null counts, matching Go's
append-to-nil behavior. Missing counts produce no count read. ReplayGrenadeRead.g
is now Option<Vec<u32>>, and its gs is Option<i64>.

`build_facts_replay_projectiles` reads borrowed cache point sequences. Signed cache
chunk metadata stays in FactsProjectileSample and is not narrowed into a source
chunk index. `build_facts_replay_grenades` retains signed i64 player identities,
including identities beyond the wire u8 range. ReplayGrenade.film_index is i64.
The shared calculations retain birth order, nearest-birth matching, ambiguity,
rank rejection, time wrapping, publication links and coverage accounting.

Four independent pinned Go generators each exercise 1,024 cases. The new cache
host comparisons pass for all four. Source regression, Clippy and WASM results
are tracked in facts-carried-inputs-validation.json; do not infer completed checks
from API availability. These adapters do not complete BuildFromFacts, and this
lossy cache projection is not the deferred canonical Film fidelity contract.
