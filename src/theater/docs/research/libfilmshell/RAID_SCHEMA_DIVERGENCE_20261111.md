# Raid schema divergence after an unread kill event

The retained raid gives a concrete lineage for one of the 32 apparent
`UnsupportedDefault` records in [coverage-20261110.json](coverage-20261110.json).
These are reader outcomes, not evidence that the film writer emitted an
`mpp.count` greater than four.

Contextual replay now exposes its own assumed NEW reads in
`ReplayDiagnostics.assumed_creations`. With `KillPlaybackDisabled`, it reaches
one complete, closed-frame type-42 NEW for full ID `2147484981` in chunk 140,
packet 1182, at relative bits 5,021–6,001. Libfilmshell reports the index
field three bits into that header (relative bit 5,024). The two views agree on
the create location and type under their respective recovery policies;
canonical `Film` still refuses the packet before the entity view.

In `raids/01-hour-long-raid`, chunk 140, type-0 packet 1182 has a 13,088-bit
payload. Canonical `Film::parse` reads the event prefix, then stops at
`MissingKillPlaybackContext`; its entity frame is
`Refused(IncompleteMessageList)`. The local source-linked slot-history example
shows no subsequent NEW for slot 1333 before chunk 141, packet 22. The last
local declaration for that slot is archetype 9, generation 1, in chunk 1;
the last reached keyframe header is also archetype 9 in chunk 65.

Current libfilmshell's entity walker, using the verified current `payload.rs`
blob `f2bd27fb808c15bcc364b6e587f7eb303e11f59c`, reports a generation-2,
type-42 create for slot 1333 in chunk 140 at absolute bit 7,400,920
(packet payload starts at byte 924,487, so relative bit 5,024). Its tag is
`0xfd98554c`. This observation comes from libfilmshell's resumed walk after
an unread section; it is a recovery interpretation, not a sequential local
canonical read of the skipped event tail and entity prefix.

In chunk 141, packet 22, libfilmshell reports slot 1333 as a type-42 update
starting at absolute bit 4,538,523 and the next update at 4,538,706.
The local parser starts the same slot's DELTA at relative bit 2,266 but
uses the stale type-9 schema: four managed-player components end at 2,415.
It then encounters an apparent archetype-38 NEW at 2,415 and refuses its
`mpp.count` value 7. The upstream next entry starts at relative bit 2,450,
35 bits later. The apparent NEW is inside the preceding type-42 update on
the upstream path. This establishes a specific stale-schema failure mode;
it does not establish all 32 raid refusals as identical.

The local lineage was inspected at slot 1333 in chunk 140, packet 1182,
and chunk 141, packet 22.

The coverage inventory now retains the four preceding local records and the
last one's component spans for each default refusal. It changes no source
partition or existing count. The next parser step is to establish the kill
playback branch with actual runtime context, or add explicitly labeled
post-stop recovery without publishing the resumed entries as canonical.
The `mpp.count <= 4` guard remains correct on the pinned creation oracle.

Contextual replay now also tries its existing two-record, unique-schema
recovery when a DELTA's full generation ID differs from an occupied slot's
binding. It does not alter canonical `Film` traversal or treat a generation
change alone as proof of a new archetype. A synthetic test proves that a
corroborated replacement can be found while retaining the original-source
record and successor spans. In the long raid, contextual replay already holds
the type-42, generation-2 binding from its assumed chunk-140 creation; the new
generation-change recovery hook is not needed for this particular continuation.
A release probe confirms that chunk 141, packet 22 reads the slot-1333 DELTA
at relative bits 2266–2449, with the next DELTA header at 2449 and its index
at 2450, matching libfilmshell's reported next-index location. The contextual
frame reaches an entity terminator, 29 records and an end at bit 4306 of its
4312-bit payload. This is source-position agreement under the explicit
`KillPlaybackDisabled` assumption, not a canonical proof of the missing kill
tail or all fields in that frame.

The public `Film::parse_with_context` entry point now accepts an externally
established event-85 tail branch. A prior probe with
`HALO_KILL_TAIL=absent` on this raid reaches the same complete type-42 NEW at
chunk 140, packet 1182 (bits 5021–6001) and type-42 DELTA at chunk 141,
packet 22 (bits 2266–2449). Its chunk-141 frame has 29 records, an entity
terminator and end bit 4306, with six zero padding bits in the 4312-bit payload.
The default `Film::parse` continues to stop at the unrecorded playback branch.
This captured continuation corroborates field boundaries conditional on the
supplied absent-tail setting; it cannot prove that setting from film bytes.
