# Upstream refresh: 2026-11-10

GitHub HEAD was checked through the GitHub API and a separate source checkout:
[`8f0f8d078e513c800baa21fac416fd4c0c127d03`](https://github.com/OpenSpartan/libfilmshell/tree/8f0f8d078e513c800baa21fac416fd4c0c127d03),
merged 2026-11-10 07:21 UTC. The old
`18067987ebc3628ad499c819c6aee814692df43b` checkout, fixtures, catalogues and
original-source oracles remain pinned to their original revision. This update
does not silently retarget those comparisons. The source diff spans 34 files.

## New evidence that changes implementation priorities

| Source | New behavior | Local action and required boundaries |
| --- | --- | --- |
| `roster.rs`, `holders.rs` | Engine-player assignments now come from type-8/type-12 listings, replacing takeover proximity. | Implemented listing-linked assignment/holder views in `runtime/roster/assignments.rs` using canonical entry fields under an explicit v41/27 reference schema; see ROSTER.md. Session present options and bot `Slot` words are already retained. Keep missing IDs, conflicts across human/bot listings, departures, reused slots and unavailable populations explicit. Do not backdate the first assignment or return the queried index as an invented identity. |
| `components.rs`, `owners.rs` | Unit-control publishes its optional controlling player; ownership constrains life assignment. | The local wire reader already retains `control.present`, `control.index`, the optional secondary index and reference. Implemented controller observations and exact-event listing attribution in `runtime/controllers`, retaining clears, changes and source independently of default-state ownership; see CONTROLLERS.md. Controller-holder histories, spans and exact-event/time queries now invalidate gaps and react to roster reassignment. Full life/replay attribution remains open. Do not permanently assign the first controller or carry identity across incomplete updates/lifecycle boundaries. |
| `components.rs`, `attach.rs` | Parent state now retains handle generation, parent node and optional seat. Rides follow attachment/detachment/deletion. | Local source fields already retain these bits: `parent` handle fields, `word`, `tail_sign`/`tail_value`. Added typed parent observations and attachment snapshots/root paths with explicit generation, namespace, source gaps, lifecycle reuse and missing-state outcomes; see ATTACHMENTS.md. Direct attachment histories/spans are implemented; ride projections remain open. Separate an absent parent from an unknown parent. Resolve chains with explicit missing-link/cycle outcomes. |
| `motion.rs`, `assemble.rs`, `model/objects.rs` | Parent-based rides splice vehicle positions into biped paths; ride lifetimes follow attachment spans. | Build on the attachment history. Keep observed attachment distinct from inferred ride/position recovery; preserve local offsets, missing vehicle tracks, baselines and partial reads. A copied vehicle center is not an exact biped pose. |
| `gamestate.rs`, `assemble.rs`, `model/scoreboard.rs` | Snapshot scoreboard rows, max-per-round life counts and grouped spawn-counter reset inference. | Local statborg history already retains all 64 bits, current/finalized values and gaps. Do not copy zero-row carry-forward, max-per-round aggregation or simultaneous-reset heuristics into canonical state. Compare semantic results against recorded round/finalized fields before adding final-score recovery. |
| `container.rs`, `lives.rs`, `model/time.rs` | Highlight clock derives from packet microseconds when available, with a 59.94 Hz conversion and monotonic clamping; inferred hitch-adjusted tick fallback remains. | Keep recorded timestamps and highlight alignment evidence separate. Local runtime already uses packet timestamps. No fixed frequency, fitted highlight zero, clamped timestamp or tick fallback should replace the source clock. |
| `model/event.rs`, `assemble.rs` | Game-event text arguments are now included in the public model. | Local `ActionPayload::Game` already preserves ordered `extra` arguments and raw types/words. Player resolution must use the assignment view, and meanings must stay specific to the game-event argument domain rather than borrowed from HUD arguments. |
| `game/items_data.rs` | Adds Pelican tag `0x000026f0`. | Add only with a new catalogue revision or explicit supplemental provenance. The 2,001-entry old catalogue must continue to mean its old pin. This entry does not establish physics type. |
| `analysis.rs` | Exposes accuracy tick inputs and the default spawn-scoring model. | These remain derived analysis/model parameters, separate from recorded hit accuracy, spawn decisions and actual variant applicability. Their public surface belongs in the wider capability audit. |
| `format.rs`, `testing.rs`, tests/CI | Public encoding labels, research feature, and tests loading films as downloaded. | Lobby-size encoding labels are descriptive hypotheses, not canonical packet discriminators. Local transport already preserves original compressed inputs and bounded decompression. Preserve test-data provenance and do not infer writer admission from labels. |

## Important source limits

The new session assignment reader still finds XUIDs by scanning shifted bytes.
It then reads six bits before the first XUID. Our established type-8 grammar
reads a version-dependent absent bit and optional five-bit value. The present
form occupies six bits and agrees numerically; the absent form occupies one.
The new scanner is not evidence to replace the optional grammar with a fixed
six-bit field. Compare original results on complete captured entries, while
testing omitted options independently. Bot assignment comes from the second
32-bit prefix word (`Slot`), not the first word (`AbsoluteIndex`) or bot ID.

`holders::api` still falls back to the first later listing and finally to the
engine index itself. `owners::reader` keeps the first controlling player and
first end event for each generated slot. `attach::root` stops after four parent
steps and cannot distinguish a cycle from a valid root. These are convenience
policies requiring explicit uncertainty in a source-faithful implementation.

The component diff adds semantic outputs by replacing skips with reads of the
same widths. It supplies no new vehicle snapshot gate or packet-boundary proof.
`snapshot.rs` is unchanged (blob `cd8971c6b1e53e593361fe29026324a98a861728`):
it still tries both vehicle layouts and judges the next slot. The 207 local
vehicle-context stops therefore remain a separate investigation.

Relevant new source blobs:

| File | Git blob |
| --- | --- |
| `src/roster.rs` | `a32657daa1b5969321b96895b683c1f8c5b7df8f` |
| `src/holders.rs` | `739462eb4fd874507459111a30ee98ba12f87df2` |
| `src/owners.rs` | `3200d89841ebd7193b7c604e665193b7a829af1e` |
| `src/attach.rs` | `38c7f19233432de069c5f01aca840df8cd9f8a88` |
| `src/components.rs` | `6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4` |

The immediate implementation order is listing assignments, controller ownership,
then attachment/ride histories, with independent original-source fixtures for
the direct values and separate negative/lifecycle tests. The assignment view is now implemented with separate native fixtures and
lifecycle tests. Controller observations and event-time holder links are also
implemented; direct attachment histories/spans now share the snapshot boundary
handling. Full life/replay attribution, ride projections and consumer integration
remain open. The existing film / parser/v41 / runtime organization and
the full canonical-format objective remain unchanged.
