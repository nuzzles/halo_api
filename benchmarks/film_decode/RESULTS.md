# Film decode benchmark — 2026-11-18

Run from the repository root:

```sh
cargo run --release --manifest-path benchmarks/film_decode/Cargo.toml -- \
  tests/fixtures/theater/bazaar_idle \
  tests/fixtures/theater/octagon_ar_kill
```

The two larger recordings below are local and ignored by Git. To repeat those
runs, supply your own film directories containing `film.json` or `manifest.json`
and the referenced chunk files.

The baseline used halo_api `13028a74`, libfilmshell `031e2179`, Rust 1.98.1,
arm64 macOS 26.7.1, release `opt-level=3`, without LTO. Source files are read
before timing. Each decode receives a fresh owned copy of every compressed
chunk. Already-inflated local files are recompressed once before timing. The
median is reported over 15 runs for small films and 5 runs for larger ones.
Set `FILM_BENCH_RUNS` to override the run count.

| Film | Compressed input | halo_api `Film::parse` | halo_api inflation alone | libfilmshell `Film::from_chunks` | halo_api / libfilmshell |
| --- | ---: | ---: | ---: | ---: | ---: |
| Bazaar idle | 0.52 MiB | 79.7 ms | 12.3 ms | 27.3 ms | 2.92× |
| Octagon AR kill | 0.51 MiB | 81.1 ms | 15.4 ms | 25.9 ms | 3.13× |
| Bandit Evo | 8.30 MiB | 2902.4 ms | 126.5 ms | 812.6 ms | 3.57× |
| Ranked Oddball | 15.36 MiB | 5069.8 ms | 232.1 ms | 4551.2 ms | 1.11× |

These are whole-film entrypoint timings, not equal-work microbenchmarks.
halo_api materializes timestamped ECS observations (973,283 in Ranked Oddball);
libfilmshell derives player/world tracks and events. Neither benchmark includes
network download or a `FilmRuntime` seek. The inflation-only result shows where
halo_api spends relatively little time.

After changing `RawBits::from_source` and `RawBits::low_u64` to work by byte
rather than by bit, the same harness measured:

| Film | halo_api before | halo_api after | libfilmshell after | halo_api improvement |
| --- | ---: | ---: | ---: | ---: |
| Bazaar idle | 79.7 ms | 70.8 ms | 29.1 ms | 11.2% |
| Octagon AR kill | 81.1 ms | 74.5 ms | 26.8 ms | 8.1% |
| Bandit Evo | 2902.4 ms | 2738.5 ms | 830.7 ms | 5.6% |
| Ranked Oddball | 5069.8 ms | 4914.5 ms | 4558.6 ms | 3.1% |

Single-run temporary phase timing on Bandit Evo attributed about 2.17 s of
2.96 s to replication decoding, 236 ms to replication projection, and 127 ms
to inflation. Those measurements identify where to profile next; they are not
stable benchmark medians. An in-bounds fast path for the reference bit reader
did not improve a single-run whole-film check, so it was discarded.

Removing redundant component-field and component clones from the entity-record
decoder produced another improvement with the same run counts:

| Film | halo_api after both changes | libfilmshell same run | halo_api vs baseline |
| --- | ---: | ---: | ---: |
| Bazaar idle | 64.0 ms | 28.1 ms | 19.7% faster |
| Octagon AR kill | 71.8 ms | 26.4 ms | 11.5% faster |
| Bandit Evo | 2126.3 ms | 824.1 ms | 26.7% faster |
| Ranked Oddball | 3766.9 ms | 4593.6 ms | 25.7% faster |

The Ranked Oddball result now leads libfilmshell on this whole-film comparison.
The smaller recordings still trail it; matching speed across the corpus needs
more work in the replication record decoder. Different output models remain the
principal limit of a direct performance comparison.

Moving component fields from the temporary record buffer into their component
reduced another copy. The parser retains fields that also belong to record
scope. With `FILM_BENCH_FINGERPRINT=1`, streaming JSON hashes of the complete
finished `Film` matched before and after this change on all four recordings.
Fingerprinting runs separately from the timed decode.

| Film | halo_api after field move | libfilmshell same run |
| --- | ---: | ---: |
| Bazaar idle | 61.9 ms | 28.6 ms |
| Octagon AR kill | 72.2 ms | 26.9 ms |
| Bandit Evo | 1981.4 ms | 853.1 ms |
| Ranked Oddball | 3451.8 ms | 4668.8 ms |

The fingerprint comparison covers every serialized finished-film field in
these recordings, rather than only counting observations. It does not prove
equivalence for recordings outside this set. Bandit Evo had one slow outlier
(4407.6 ms), so its median should be repeated on a quiet machine before using
small differences to guide further changes.

The same field-transfer rule was then applied to keyframe components. Complete
film fingerprints matched again, and the median timings were:

| Film | halo_api final | libfilmshell same run | halo_api improvement vs baseline |
| --- | ---: | ---: | ---: |
| Bazaar idle | 51.6 ms | 28.6 ms | 35.3% |
| Octagon AR kill | 59.7 ms | 27.0 ms | 26.4% |
| Bandit Evo | 1967.2 ms | 865.3 ms | 32.2% |
| Ranked Oddball | 3313.8 ms | 4636.6 ms | 34.6% |

The preserved whole-film fingerprints were `9ebb6a12e814cc91` (Bazaar),
`37ecdf4a27f86c9d` (Octagon), `cf47e261e88055a5` (Bandit Evo), and
`09f3de8ebe6fb0e7` (Ranked Oddball). These 64-bit hashes are a within-toolchain
regression aid, not a stable external film identifier.

The final bit-copy change stopped zero-filling output bytes that are immediately
overwritten. All four complete-film fingerprints still match. Bazaar and Octagon
were effectively unchanged (51.4 and 59.9 ms); Bandit Evo improved to 1827.5 ms.
The Ranked Oddball run was too noisy for a useful comparison: its inflation-only
median rose from 236.8 to 344.6 ms and both decoders slowed. A later single run
with normal inflation time still parsed the same film in 3580.3 ms. Repeat this
case under stable load before attributing a change to the parser.

The final `low_u64` change uses 64-bit arithmetic for the trailing value
instead of accumulating into a 128-bit integer. It includes the preceding byte
when the final byte contains padding. Exhaustive bit-range tests and the same
four complete-film fingerprints passed. A fresh run measured:

| Film | halo_api median | libfilmshell median | Runs |
| --- | ---: | ---: | ---: |
| Bazaar idle | 48.3 ms | 27.3 ms | 15 |
| Octagon AR kill | 56.2 ms | 25.7 ms | 15 |
| Bandit Evo | 1776.9 ms | 837.5 ms | 5 |
| Ranked Oddball | 3154.3 ms | 4721.0 ms | 5 |

The small-film and Bandit gaps remain. The Ranked result has normal inflation
time in this run, but a single five-run comparison cannot isolate a small
change from system noise.

The final change delegates ordinary zlib header and Adler-32 validation to
`flate2` instead of scanning inflated bytes in Rust. The existing raw-DEFLATE
path remains for the supported empty preset-dictionary header. Unit tests
cover valid data, corrupt and truncated checksums, the dictionary header, and
the exact output limit. A proposed `SmolStr` field-name change was discarded
to keep the public `ReplicationField` API stable.

With five runs per film, the final branch measured:

| Film | halo_api median | libfilmshell median | halo_api vs initial baseline |
| --- | ---: | ---: | ---: |
| Bazaar idle | 40.3 ms | 27.5 ms | 49% faster |
| Octagon AR kill | 43.7 ms | 25.9 ms | 46% faster |
| Bandit Evo | 1610.1 ms | 814.2 ms | 45% faster |
| Ranked Oddball | 2869.8 ms | 4511.8 ms | 43% faster |

All four complete-film fingerprints still match the pre-optimization values
above. halo_api is faster than libfilmshell on Ranked Oddball in this
whole-film comparison, but still trails it on the other three. The decoders
produce different output models, so this is an entrypoint comparison rather
than an equal-work measurement.
