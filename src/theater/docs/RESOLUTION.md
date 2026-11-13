# Typed state playback

`TheaterRuntime::load(film)` holds the finished film through `Arc` and indexes
`film.replication_stream.observations` by timestamp. The runtime does not parse
packets or inspect retained source bytes. Playback begins with an empty world.

`advance_to(t)` applies all observations through film-relative microsecond time
`t`. Moving backward resets the world before replaying through the earlier
time. `seek(t)` always rebuilds from the beginning, and `rewind()` clears the
cursor. `current()` borrows the materialized world; `timestamp_us()` returns the
requested time when the cursor has been positioned. Equal timestamps preserve
recorded observation order.

An entity is keyed by its low 30-bit slot. Runtime IDs include their generation;
a stale update or delete for a different generation is ignored. Complete NEW
observations establish a state with archetype and decoded fields. DELTA
observations replace recorded component values on the matching live entity.
DELETE removes it. Keyframes replace a baseline in the first observed namespace;
other namespaces remain separate. Controls and recorded events appear in the
observation feed but do not change entity state by themselves.

`Film.summary.highlights` and `TheaterRuntime::summary_highlights()` expose the
same guarded candidates. Each has gamertag, XUID, timestamp, and typed payload.
Kill and death candidates do not invent an opponent or weapon. The summary
record grammar is still incomplete, so absence from this list does not prove
an event did not happen.

Loading builds a time index in O(n log n). `current()` is O(1); seeking can replay
all earlier observations. Two runtimes built from the same `Arc<Film>` share the
film but have independent world cursors.
