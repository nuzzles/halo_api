# Resolution and playback

`Film::resolve(&self)` walks the existing decoded records and returns
`ResolvedFilm<'_>`, borrowing the film. It does not decompress or parse source bytes
again. Multiple independent resolutions can borrow one film.

The resolved model orders events by microsecond timestamp and stable order within
a timestamp, indexes them by kind/category/entity/player, and accumulates entity
state with generation checks. Native source references retrieve the original typed
record. Events identify recorded reads, partial reads or packet envelopes. Derived
state changes carry previous/new shared entity states and a derivation identifier.

This is native state resolution. It does not invent player-to-entity associations
or label a control bit as an observed physical jump. Continuous component fields
remain queryable. Filtering affects the query result, not the stored events or
playback. `EventFilter` combines time, kind, category, entity and player constraints;
queries start from the smallest applicable index within the time bounds.

- `current()` returns a borrowed, already-materialized world in O(1).
- `advance_to(t)` applies intervening indexed updates; the cost depends on the
  updates and affected state. Backward movement uses the seek path.
- `seek(t)` locates a checkpoint and applies its remaining updates. Checkpoints
  are stored every 1,024 indexed events. Finding an index is logarithmic; cloning
  the checkpoint's world map scales with the number of entities. Seek is not O(1).
- Enumerating or copying the world scales with world size. Entity state uses
  shared storage, but indexing, checkpoints and changes still consume memory.

Partial component reads and incomplete NEW records cannot silently promote unknown
state to known state. Recovery candidates never create resolved entities. Seeking
and sequential playback must agree at the same timestamp, including ties and
entity generations. Interpolation and a browser renderer are separate consumers;
this API does not fabricate intermediate recorded samples.
