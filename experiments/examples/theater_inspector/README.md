# Theater Lab · Recording inspector

> Active lab cleanup: 32 films are retained (natural-end and later controls,
> plus the two Arena games and an hour-long raid). Forced-end recordings and standalone probes are
> archived under `archive/cleanup-2026-09-10/` at the experiments root. Historical
> 36-film/30-clip counts below describe earlier validation runs. See the main
> experiments README for current commands. Python field readers used by the
> byte inspector now live under `examples/legacy/`.


Experiment 02 is a read-only browser for the actual decompressed recording bytes.
It shares the motion replay's list of 32 cached catalog recordings, short titles,
category grouping and CSV row order. The first catalog entry opens by default;
`?film=group/slug` selects a specific recording. The
three linked views show hex/ASCII, decoded fields and gaps, and individual bits.

## Run

From the repository root:

```sh
python3 experiments/examples/theater_lab.py
```

Open **http://127.0.0.1:8766/**. Keep the command running while using the inspector;
Ctrl+C stops it. Python's standard library is the only server dependency. It
binds to localhost, reads cached files, and makes no Halo API requests.

Use `--port 8767` to change the port, or `--corpus /path/to/films` to inspect another
cache containing the catalog's recordings and analysis exports. The server exposes
only its UI, catalog-selected film data, and the generated replay; it is not a
general filesystem server. It has no editing, upload, or write endpoints.

The Motion replay and Recording inspector tabs preserve the selected recording
when switching between experiments. Both menus use `/api/catalog` and the shared
`recordings.js` menu code; unresolved films remain visible in both. The
existing offline replay also links to the default inspector address. The inspector
uses local HTTP to page through large recordings without embedding the entire
corpus into an HTML file. The motion replay still opens directly from disk.

## Explore a recording

- Choose a recording and chunk. Every cached chunk is byte-addressable, including
  registry and summary chunks without a supported packet parser.
- Jump to a film time in seconds, or enter a decimal / `0x` chunk byte offset.
  Time navigation chooses the nearest frame. The initial view selects a nearby
  frame with exported fields so there is decoded data to inspect immediately.
- The packet list can show exported fields, all frames, or every packet kind.
  It is paginated independently of the byte view.
- Click either a hex byte or its ASCII character. Both columns highlight the
  same selection; the details panel shows its eight bits, raw interpretations,
  provenance, and related fields. Nonprintable ASCII appears as `·`.
- Click a decoded field to select its exact bits and jump to its bytes. Fields
  can cross byte boundaries. Click a bit to inspect the field covering that bit.
- Filter the decoded view to unparsed gaps or opaque fields to find research
  targets. The coverage bar uses the same precise ranges as the field list.
- Search the selected chunk for text encoded as UTF-8 or a hex byte sequence.
  **Find next** starts after the previous match and wraps through the chunk.
  A registry search for `object-body-vitality-component` is a useful example.
- **Copy page hex** copies the visible byte page. **Export page JSON** saves its
  bytes, selected bit range, packet/window coverage, annotation ranges, sources,
  and any withheld-annotation errors.

## What the colors mean

| Status | Meaning |
| --- | --- |
| Decoded · mint | Extracted value with an established interpretation, often a provisional raw integer window |
| Checked structure · blue | Recognized signature, guard, mode, or state code; recognizing a constant does not explain every bit's role |
| Opaque · amber | Field boundaries / length are checked, but its semantic value is not decoded |
| Unparsed · muted purple | No supported inspector annotation covers these bits |

Striped bytes contain more than one status. The bit view resolves the ambiguity.
Coverage is counted in **bits**, without double counting overlapping annotations.
A semantic vitality window can replace an opaque component span; it does not
make the rest of the record decoded.

The **Whole recording coverage** panel above navigation shows a compact pie chart
and percentages plus exact bit counts for all four categories across **every
decompressed chunk byte**, including packet headers, registry padding and unknown
chunk types. The denominator is the sum of actual chunk file sizes × 8, not the
compressed download size or just packets with exported fields. **Export coverage
JSON** saves the match ID, denominator, category counts, annotation basis and any
withheld-annotation issues.

The first visit scans one chunk at a time and shows progress. Percentages appear
only after every chunk is counted; unscanned chunks are not presented as unparsed.
Small per-chunk summaries are cached on the server, and completed recording totals
are cached in the page. Packet navigation does not change this chart. Switching
recordings cancels the old request and clears its statistics.

This measures the inspector's **verified annotations**, not every observation the
Rust decoder can produce. Fields without exact inspector bit annotations remain
unparsed, even when a native `SourceSpan` covers them. A broad source window cannot
prove that all of its bits are understood. Missing exports leave their fields
unparsed; rejected exports withhold their annotations and report the issue count.

The smaller coverage bar within the workspace still describes the **selected
packet, including its 16-byte header**, or the visible byte window in non-packet
chunks. Pages stop at packet boundaries so the byte and decoded views always
describe the same packet. Packet header numbers
are little-endian; bit offsets count MSB-first within each chunk byte. Ranges in
exports are half-open `[start, end)`.

## Supported annotations and evidence

`theater_inspector_data.py` checks source fields against the cached bytes before
emitting annotations. If an export disagrees, that record's annotations are
withheld and the UI reports the discrepancy. Record-length recognition alone
never paints an entire record as decoded. Unparsed also includes fields that
another probe may recognize but that this inspector has not integrated.

| Region | Annotation source |
| --- | --- |
| Replication packet headers | 16-byte `<HHIQ` layout; the second 16-bit word remains opaque |
| Frame clocks | Checked 37-bit clock layout, including all three Oddball prefixes |
| Registry component names | Fixed slot geometry used by `film.rs::decode_registry`; headers and padding stay unparsed |
| Ranked spawn / identity / positions / aim / command tick | `analysis/{bandit,oddball}/decode_evidence.json` and `delta_evidence.csv`, revalidated with the capture decoders |
| Controlled positions, aim and input axes | `analysis/all-films/{positions,aim,inputs}.csv`, checked against original signatures and values |
| Octagon spawn coordinates | `analysis/octagon/spawn_evidence.json` |
| Firing | `analysis/firing/evidence.json`; historical annotations keep the weapon window amber. The native decoder now splits its slot/guards/fingerprint; see [weapon evidence](../../FILM_WEAPONS.md). |
| Scope stages | `analysis/zoom/evidence.json`; two-bit stage, pawn identity/generation, following clock/auxiliary guards; leading bits remain unparsed |
| Reloads, magazines, weapon selection | `analysis/weapons/evidence.json`; reload/clock guards, short zero, positive amounts, selected slot; unknown leading/auxiliary bits remain unparsed or opaque |
| Grenade throws and controlled projectile paths | `analysis/grenades/evidence.json`; throw/clock windows and 15/15/17 coordinates; unknown trajectory fields stay opaque |
| Melee and companion roster identity | `analysis/melee/evidence.json`; both guarded windows revalidated |
| Bandit and Oddball body health/shields | Checked sparse delta component windows using `film_vitality_probe.py` |
| Octagon body health/shields | `analysis/vitality/samples.csv`, revalidated against isolated component and boundary guards |

Missing exports leave those regions unparsed. The inspector does not automatically
rerun probes or fetch recordings. Summary-event payloads, general entity-chain
walking, unknown components, registry metadata/padding, and unsupported input
forms remain unparsed. Both Ranked matches include checked firing, melee, grenade throw, and
vitality windows, including generation reuse. Coordinate and health scales remain
provisional; supported fields do not imply complete combat or damage coverage.

The server keeps in-memory caches of film indexes, CSV record offsets, per-chunk
evidence, and small coverage summaries. CSV indexes avoid rereading an entire
large export for each chunk; overlap counting uses sorted range boundaries, not
one entry per bit.
Restart it after regenerating source exports or replacing chunk files.

## Validation

From the repository root:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s experiments/examples -p 'test_*.py'

# With theater_lab.py running and Chrome installed:
node experiments/examples/check_theater_inspector.cjs
node experiments/examples/check_inspector_coverage.cjs
node experiments/examples/check_recording_catalog.cjs
```

The browser driver defaults to Chrome's macOS application path. Set `CHROME_PATH`
for another installation and `THEATER_LAB_URL` for another local port. Screenshots
are saved in the system temporary directory. Captured-record tests check original
bit offsets, both Oddball spawn layouts, all three clocks, ID reuse, annotation
rejection, opaque overlays, and exact coverage of unaligned fields. Browser checks
cover hex/ASCII/bit/field linking, search, navigation, actual shield values,
read-only API behavior, and desktop/mobile layout.
The focused coverage check also covers complete clip totals, coverage JSON export,
scan cancellation during film changes, cache reuse, and the pie chart on mobile.

The reload comparison is `weapons/02-reload-comparison`. Seek to 26.875065 s
(manual start), 39.337740 s (two-bit empty magazine), 39.371112 s (automatic
start), or 40.171841 s (refill to 12). See [weapon evidence](../../FILM_WEAPONS.md)
for exact packet/bit offsets and limits; Ranked ammo is not annotated yet.

For `weapons/03-br-sniper-zoom`, scope observations occur at 22.036707,
26.674662, 32.080040, 36.801706, and 41.889813 s. Click **Zoom stage** to select
the two original bits. The 0/1/2 stage is not a magnification multiplier. See
[scope evidence](../../FILM_ZOOM.md) for boundaries and partial Ranked coverage.
