# Film fixtures

These seven recordings are full films, not individual packet samples. Each
`.tar` file uses Git LFS and contains `manifest.json` plus zlib-compressed
`.bin` chunks. The manifest provides only the metadata needed by both parsers.

The Aquarius, Bazaar, Octagon, Oddball, and Raid captures came from the local
halo_api Theater film corpus. Ranked Slayer on Live Fire and Ranked CTF on
Aquarius came from the local `halomagick_twin/apps/theater/fixtures/ranked_arena`
source recordings. The original local files were inflated; this benchmark stores
them recompressed once with zlib level 1. No recompression is done during timing.

These are test recordings. Do not serve them from the Pages site; the Pages
workflow uploads only `benchmarks/website`.
