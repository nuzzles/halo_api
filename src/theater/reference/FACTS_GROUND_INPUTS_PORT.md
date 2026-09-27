# Cached ground-object assembly and lifetime resolution

assemble_facts_ground_objects and resolve_facts_ground_weapon_pickup share their
recording-path reducers. Assembly projects only consumed creation fields and
borrows ammo payloads; motion selection uses the first timestamp and last position
of nonempty tracks. Original cache provenance, masks, timestamps, chunks, and
unused samples stay in the cache DTOs. Nil and empty tracks both supply no motion
evidence, as native does, while their cache distinction remains retained.

Objectives reject before catalog membership. The low u32 of MPPVal[1] is the native
catalog ID, gated by MPPPresent[1]. Full slot/generation keys group separate lives;
creation timestamps and all player samples (including unavailable-world samples)
contribute to the native film-end denominator. Life ordering, nearest-start track
selection with wrapping tolerance, keyframe disappearance bounds, owner inference,
ammo presence and output ordering share one implementation. An inferred passage
remains distinct from a written pickup event.

The new 1,024-case native object oracle covers generations above 255, high MPP ID
bits, extreme timestamps, empty tracks, objective rejection, ammo and distance
bits. The existing independent 1,024-case native lifetime fixture also checks the
cache-facing public resolver. See facts-ground-inputs-validation.json for actual
completed validation.

## Pad clock correction discovered during cache integration

GroundPadClock.frames now retains native i64. Native pad assembly only rejects a
zero time step; zero/negative frame counts still use clampFrame(frameOf(...)).
The previous Rust zero-frame guard and unsigned saturating clamp did not match.
The shared pad layer now preserves native wrapping timestamp/frame arithmetic and
signed frame clamps, including i64::MIN. Source ground/pickup wrappers retain the
signed player clock rather than clamping/casting it through usize.

A separate 1,024-case native pad-clock fixture exercises signed frame extrema,
wide timestamps/origins/steps and actual native pad layers. Existing positive-clock
pad and corpus-derived layer fixtures remain regressions. This is a parser/replay
parity correction, not the deferred playback API or canonical Film refactor.

The cache pad-scan wrapper, signed scan coverage and ground-item publication are
now implemented; see FACTS_PAD_SCANS_PORT.md for their validation. Document-level
pad dating/attachment remains pending, followed by vehicles/objectives, complete
BuildFromFacts/capture/fallback handling, legacy domain reconciliation and the final
all-data audit. These adapters alone do not establish full v41 parity.
