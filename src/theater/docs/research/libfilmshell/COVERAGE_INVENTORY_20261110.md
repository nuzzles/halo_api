# Retained-film source coverage inventory (2026-11-10)

The [coverage-20261110.json](coverage-20261110.json) report was generated from the 32
retained `experiments/films` manifests. The inventory used `Film::parse` and
`Packet::source_coverage`; it does not decode the packets a second time. All
sampled packet payloads have valid, nonoverlapping partitions and no replication
packet-framing bytes are unread.

The largest unparsed totals are 208,276,464 bits in type-2 keyframes and
135,321,425 bits in type-0 frames. The report attributes 207,324,603 keyframe
bits to packets whose final record stops at `RuntimeContextUnavailable` (207
vehicle snapshot stops), and 951,861 bits to one unsupported component. It
attributes 116,940,273 frame bits to `entity_end: Rejected` and 14,242,557 to
`MissingKillPlaybackContext`. The largest current type-0 gap is in the ranked
Oddball film. These are packet-level attributions: bits
earlier in a packet can have different causes, and a rejection does not prove the
rejected body is decodable with the current schema.

Type-8 roster packets retain 132,093,236 opaque bits behind their bounded
outer records. Type-10 replication packets retain 32,774,328 opaque bits and
type-9 summary packets retain 92,288,240 opaque bits. `Opaque` means the bytes
are preserved but their inner grammar is not established; it is distinct from
`Unparsed` after a reader stop. A complete coverage partition is therefore not
a complete field decode or a round-trip writer.

`ProductionEntityEnd::Rejected` occurs when a DELTA has no admitted binding or
belongs to another view. The recorded header supplies no end for its unread
body, so the canonical reader cannot advance to controls or later records on
successor plausibility alone. The report identifies a priority for tracing
binding history against prior NEW/keyframe records, not permission to weaken
canonical admission. Vehicle suffix branches and speculative schema recovery
remain runtime diagnostics until their selectors are source-supported.

The parser records the rejection reason separately from the source-read header.
In this corpus all 143,882 remaining rejected DELTAs are `Unbound`; none are
`OtherView`. Of these, 141,478 name slots 1024 or higher and 2,404 name lower
slots. Older serialized stop records without a reason deserialize as `Unknown`
rather than acquiring a guessed cause.

The first inventory had 152,516 rejections and appeared to show an unbound
slot 1024 at packet 8 in every film. That was a **reader alignment defect**:
the NEW component gate was clear on the first entity, but our reader still
consumed a component mask from the next entry. Current libfilmshell's
`payload.rs` at `a13efb323b230c96b59685defbc376720043240f` makes the mask
conditional on the gate and omits the gate for ten creation archetypes. After
matching that grammar, the retained Octagon first frame reads 207 source-bounded
entries through its recorded terminator, the same count reported by the
upstream entity view. Across the corpus the fix transfers 10,930,263 bits from
unparsed to fields in type-0 packets and removes 8,634 rejected stops. The
old slot-1024 attribution is withdrawn; it was not evidence of a missing
startup-object schema. Remaining unbound bodies still need their own binding
histories and cannot be admitted by the old mistaken explanation.

The inventory now also lists every `UnsupportedDefault` stop with its source
location and last recorded field. All 32 occur in 15 chunks of the long raid,
across apparent NEW archetypes 35–43. Each stops at `mpp.count`: fourteen read
5, nine read 6, and nine read 7. The pinned creation reader and its original
function fixtures admit at most four entries for that count. These late raid
records therefore need a preceding-boundary and context investigation; raising
the count limit would turn an invalid apparent record into a claimed native
default. No other corpus film has this stop. The additional inventory fields do
not change any prior coverage count or source partition.

A [raid slot-lineage trace](RAID_SCHEMA_DIVERGENCE_20261111.md) identifies one
failure mechanism. Chunk 140, packet 1182 stops at an unread kill-playback
branch before local entity decoding; libfilmshell's resumed walker reports a
generation-2 type-42 create there. In chunk 141, packet 22 the local reader
uses its stale type-9 slot binding, ends that DELTA 35 bits before the
libfilmshell walker, and reads the remaining bits as an apparent NEW whose
`mpp.count` is 7. The inventory now retains the four preceding local records
and immediate component spans for all 32 refusals so the other cases can be
checked rather than assumed equivalent.
