# Packet types 6 and 10: body evidence and admission boundary

The [measured inventory](packet-bodies.json) covers all 32 retained films. Its
[generator](packet_bodies.py) walks only the recorded 16-byte packet envelopes,
requires exact chunk closure and records source hashes and byte-exact examples.
No signature search, guessed fields or candidate body parser is used.

| Packet | Occurrences | Payload bytes | Distinct complete payloads |
| --- | ---: | --- | ---: |
| 6 | 451 | 4 | 1 |
| 10 | 403,465 | 5, 6, 8, 9, 10, 17, 18, 19 | 1,028 |

All observed type-6 bodies are `00000000`. Every one occurs between a type-2
keyframe table and a type-8 population packet. Its timestamp is 104–305 us after
the preceding packet. This does not establish whether the four bytes are a
count, flags, reserved storage or something else, nor whether nonzero bodies
are valid.

Type-10 body-size counts are 146,485 at 5 bytes; 29,054 at 6; 29 at 8; 37,086 at
9; 72,346 at 10; 45 at 17; 118,169 at 18; and 251 at 19. There are exactly as
many type-10 packets as type-0 packets in this corpus, but adjacency is not a
universal rule: 402,988 follow type 0 and 477 follow type 12. All are followed by
type 0. The recorded timestamp difference from the immediately previous packet
ranges from 51 to 247,832 us. Neither equal counts nor common prefixes proves
a body schema, a simulation tick duration, or absence of state effects.

## Source check

libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03` `src/container.rs`
names type 10 as a per-tick companion and type 6 as bookkeeping. It frames
their payloads but supplies no body reader. No native type-6/type-10 decoder
was found in the pinned LevelUp source files inspected below, either. Their
packet types retain an uninterpreted byte slice after the envelope.

LevelUp pin: `413971bcc485be53a2fa8cf229e170539a5146f8`.

| Inspected source | Git blob |
| --- | --- |
| `film/internal/source/source.go` | `f6c033dd51387d4392b79dc02280341865e17f61` |
| `film/internal/source/doc.go` | `299229a6ab37931e5ea89acf9517ca064a9cc0ca` |
| `film/internal/source/rev.go` | `a0bafbe97240d09205e556dfa4ec859f11644d0a` |
| `film/types/source.go` | `69a38acd1573a24f95dc440a64b893e0ccbbc67c` |
| `film/internal/grammar/film_packets.go` | `f9f2141d44712944ffd597f465fb86453edb38c1` |
| `.ai/V7.5/MESURES_CUISSON_PERF.md` | `44d0dda5c37448e6b9d51c270c8a8f7e857127d1` |
| `.ai/V7.5/PLAN_CUISSON_PERF.md` | `109a11b85d4af48819587e9ba6fe45ad3a31d594` |
| `.ai/V7.5/film_re/NOTE_ENVELOPPE_EVENTS_2026-08-30.md` | `ed0fec5995973b0f1047c52a25d589df45a1d596` |

The film source paths above are relative to
`apps/go-api/internal/games/halo_infinite/`. The last note describes embedded
event reference envelopes, not these outer packet bodies. The performance
notes establish framing/end-marker behavior, not state-neutrality for types
6/10. Corpus regularity alone is insufficient to weaken canonical admission.

## Implementation decision

Keep both bodies source-preserved and opaque, with the existing explicit
continuity gaps. Adding an integer to the four-byte body or ignoring type 10
would turn an unverified hypothesis into a canonical field/state assertion.
Completing this part requires native serializer/deserializer or playback
dispatcher evidence, including the nonzero/variable branches and their effects.
This is a source-evidence limitation for these packet bodies, not a claim that
all remaining libfilmshell capabilities are blocked.

An independent remaining capability has a concrete source path: upstream
`control.rs` exposes movement-input series from complete kind-0 control records,
using the two six-bit words minus 31 and the optional five-bit stance field.
The local canonical control reader already preserves those fields, but
`PlayerReplay` has no equivalent input observations/series. Continue that
runtime work using exact recorded indices and holder evidence, without making
the type-6/type-10 bodies disappear or claiming continuous state across them.

## Reproduction and verification

```sh
python3 src/theater/docs/research/libfilmshell/packet_bodies.py experiments/films
```

The report includes all contributing manifest/chunk SHA-256 hashes, header flag
distributions, neighbor counts and up to three distinct source-linked examples
per payload size. Counts agree with the earlier independent packet inventory.
Regeneration is byte-identical. No parser/runtime behavior or fixture bytes were
changed; the preceding 500-test/ten-doctest checkpoint remains the production
verification. No commit or push was made.
