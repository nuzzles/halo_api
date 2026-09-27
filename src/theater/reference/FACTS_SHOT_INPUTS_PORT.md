# Cached shot attachment

`facts_shot_inputs.rs` consumes exact FactsFireEvent records. Its player index is
native i64, including negative values and values above the recording wire field's
u8 domain. The bridge does not fabricate FilmFireEvent chunk/packet fields or cast
cache identity values into a byte.

`replay_shots.rs` now shares one internal attachment calculation between recording
and cache callers. The internal result references orphan input indices; each
public wrapper returns a clone of its own original event type. Thus the recording
API retains all its existing source fields, while the cache API retains every
cache field, including inactive aim payloads. The same calculation controls
nearest-position ownership, ambiguity, absent world coordinates, time windows,
heading/coordinate rounding, weapon labels, native sort and coverage accounting.

FactsReplayShotPublication preserves the native slot-membership publication gate.
Its intentionally absurd track-frame bounds in the native fixture ensure that this
gate is not incorrectly turned into temporal containment. Rejected publications
are counted but do not become recoverable orphans. Vehicle recovery now uses attach_facts_replay_vehicle_shots and the shared
recovery calculation. Full cached combat/document composition remains pending.

The independent native oracle extends the existing shot generator to 1,024 cases
with i64 extrema, negative indices, 255/256 and values beyond 32-bit range in both
ownership and events. It compares complete shot outputs, original orphan cache
payloads/reasons and every coverage field before/after publication. Source-path
shot and vehicle-recovery tests also run to guard the shared calculation.

This is one remaining cache-to-document input path, not full BuildFromFacts.
See facts-shot-inputs-validation.json for completed checks and remaining work.


The additional 1,024-case vehicle oracle compares recovered shots, untouched
orphan evidence, all moved rejection/publication counters, optional vehicle
coverage and the conditional replacement verdict. Signed identity extrema are
present in both ownership and orphan events. Vehicle interpolation, ambiguous
rides, missing placement, unpublished riders, seat ordering and weapon exclusions
use the same already-ported code as the recording path. A recovery that moves only
unpublished counters still does not invent a replacement verdict.
