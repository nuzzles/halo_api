# Theater architecture

The public flow is `Film::parse(chunks)` followed, when playback is needed, by
`TheaterRuntime::load(film)`.

```text
theater/
  film/
    mod.rs                 Film and its single parse entry point
    registry/              Finished Registry values
    replication/           Finished ReplicationStream observations
    summary/               Finished Summary highlights and medal values
    catalog/                Known game assets and identifiers
  parser/
    mod.rs                 Chunk ordering, validation, inflation, version selection
    chunk.rs               Validated kinds, bounded inflation, packet framing
    context.rs             Temporary writer-side parsing context
    limits.rs              Internal decoding resource policy
    ticks.rs               Replay tick admission evidence
    packet/                Private packet envelopes and source ranges
    bits.rs                Bounded bit primitives
    v41/
      chunks/registry/      ParsedRegistry, Bond source model and bootstrap grammar
      chunks/replication/   ParsedReplicationStream, decode state and record grammar
      chunks/summary/       ParsedSummary and guarded highlight projection
      context/              Temporary writer and stream decode context
      reference/            Internal reference-reader evidence
      tests/                Structural parser and fixture checks
  runtime/
    mod.rs                 Typed state playback over Film observations
clients/hi/models.rs        Downloaded RawFilmChunk and manifest kind
```

The public `Film` stores three typed results. `Registry` has version, component
schemas, decoded game options, and map identity. `ReplicationStream` owns
source-ordered observations with timestamps and typed changes. `Summary` owns
only guarded highlights. Source packet buffers, framing boundaries, partial
reads, and parser stops live in temporary `Parsed*` values. They are consumed
before the final `Film` is returned.
The public section modules own their finished types; source-read and Bond
models are defined and imported within `parser/v41/chunks`.

`ChunkReader` sorts by manifest index, validates the registry/replication/summary
order, inflates each chunk exactly once, and chooses the v41 parser from the
registry header. The v41 reader keeps one decode context across replication
chunks. `Film::parse` checks contiguous indices and tick evidence for every
film, then projects the finished observations. Opaque
and incomplete reads stay available to parser tests but do not become typed
state changes.

`TheaterRuntime` retains the finished film in shared ownership and builds a
time index for playback. An independent cursor applies complete changes through
a requested timestamp. `seek` reconstructs state from the beginning; `rewind`
clears the cursor. Highlights remain accessible directly from `Film.summary`.
The runtime does not inspect wire packets or infer recovery candidates from
opaque bytes.

This split keeps decoding evidence separate from gameplay state. The source
research documents in this directory describe the wire grammar and its limits;
they do not define additional public `Film` fields.
