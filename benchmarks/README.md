# Benchmarks

| Benchmark | Contents |
| --- | --- |
| [`001_film_parse`](001_film_parse/) | Seven complete Theater film fixtures and the halo_api/libfilmshell decode harness, with single- and multi-threaded halo_api runs. |
| [`website`](website/) | Static GitHub Pages chart generated from the harness JSON output. |

The site publishes a measured snapshot. It does not run benchmarks in a GitHub
Actions runner: timing depends on the machine, and every CI run would download
the Git LFS recordings. See the benchmark README for the command and methodology.

[View the published chart](https://nuzzles.github.io/halo_api/).
