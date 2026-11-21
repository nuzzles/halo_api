# Common name pools and incremental projection

Measured on Apple M1 Max (10 logical CPUs), macOS/aarch64, with rustc 1.98.1. Baseline parser: `77e50d2`; optimized parser: `29c2345`. Both binaries use the benchmark harness from `29c2345`, the same locked dependencies and release profile (`opt-level = 3`, `debug = 1`), without custom target CPU flags.

Each cell is the median of 25 timed parses following one warmup. Files are loaded before timing; timing includes cloning compressed inputs, inflation, parsing and projection, and excludes destruction of the returned film. Each fixture/version/mode runs in a separate process. Baseline and optimized processes alternate order between fixtures; see [execution order](execution-order.json). No builds or tests ran concurrently with these measurements. These are local measurements on one machine, not cross-platform performance guarantees.

| Film | Single before → after (ms) | Time reduction | Multi before → after (ms) | Time reduction |
|---|---:|---:|---:|---:|
| Aquarius Idle | 26.6 → 25.4 | 4.4% | 16.8 → 16.5 | 2.2% |
| Bazaar Idle | 20.7 → 20.3 | 1.7% | 14.2 → 13.6 | 4.3% |
| Octagon | 155.4 → 133.9 | 13.8% | 52.0 → 46.3 | 10.9% |
| Ranked Slayer | 449.5 → 379.0 | 15.7% | 348.5 → 316.1 | 9.3% |
| Ranked Oddball | 1,316.8 → 1,227.5 | 6.8% | 385.3 → 288.3 | 25.2% |
| Ranked CTF | 732.5 → 677.2 | 7.6% | 221.5 → 157.6 | 28.8% |
| Clan Raid | 8,778.0 → 4,049.9 | 53.9% | 2,496.4 → 2,531.5 | -1.4% |

All seven whole-film JSON fingerprints match across both revisions and threading modes. The additional packet-boundary regression test also checks full `Film` equality after re-chunking a captured film in both modes.

## Peak resident memory

Three fresh-process Clan Raid runs per revision, single-threaded, each with one warmup plus one timed parse. `/usr/bin/time -l` measures the entire benchmark process, including fixture loading, fingerprint serialization and output destruction; this is not an isolated parser allocation count.

| Revision | Median peak RSS (GiB) | Individual peaks (GiB) |
|---|---:|---|
| baseline | 7.953 | 8.247, 7.946, 7.953 |
| optimized | 4.770 | 4.804, 4.770, 4.712 |

Median whole-process peak RSS decreased by 40.0%. Raw byte counts and parse timings are in [memory.json](memory.json).

## Reproduction and raw data

Build the baseline at `77e50d2` with only `benchmarks/001_film_parse/src/main.rs` copied from `29c2345`, and the optimized binary at `29c2345`. Keep separate copies of the executables. For each fixture and each mode:

```sh
FILM_BENCH_RUNS=25 /path/to/binary --halo-only --threading single --output result.json ranked_slayer
FILM_BENCH_RUNS=25 /path/to/binary --halo-only --threading multi --output result.json ranked_slayer
FILM_BENCH_RUNS=1 /usr/bin/time -l /path/to/binary --halo-only --threading single clan_raid
```

The harness resolves fixture paths from its compile-time Cargo manifest directory, so keep the fixture directories available. The baseline build includes the updated harness only; no parser changes. Captured reports explicitly record each binary’s parser revision because the harness normally reads Git HEAD at runtime. `benchmark_harness_commit` and `binary_sha256` record the common harness and the exact executable used. Each report retains ordered raw samples and whole-film fingerprints.

- [Baseline single](baseline-single.json)
- [Optimized single](optimized-single.json)
- [Baseline multi](baseline-multi.json)
- [Optimized multi](optimized-multi.json)

These focused comparisons do not replace the separately measured libfilmshell website reports.
