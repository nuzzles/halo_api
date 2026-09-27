# Vehicle source failure and context audit

Reference: LevelUp feat/v75 at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`. Scope is the vehicle source path;
this does not complete the general FilmContext/observer port.

## Verified MPP selection and restoration

The 88-case source oracle now supplies varied preexisting MPP widths and varied
calibrated widths to actual `decodeFilmVehicleScan`. For these major-41,
format-27 fixtures, recorded widths 9/5 take precedence. Every old output and log
is unchanged. The native harness compares its complete `ProfilDeBalayage` before
and after the call, covering 50 successful scans, 30 no-band exits and eight
creation failures. It separately reinstalls the selected widths for the test's
shared-march input and compares those statistics with the scan's statistics.

Rust resolves the same inputs through `resolve_film_mpp(...).creation_widths`,
then passes widths by value and map/registry/component precision by immutable
reference. The source scanner does not install a mutable shared profile. That
satisfies this local no-leak requirement; it does not assert equivalence of the
native general profile setters, future-reader context or shared observation API.

## Failure paths inspected

| Native path | Current Rust boundary and evidence | Remaining work |
| --- | --- | --- |
| No ti=40 census band | Early source INFO preserves keyframe denominator; 30 actual source cases | Captured absence is complementary evidence |
| Missing vehicle archetype | Reachable with truncated registry plus vehicle keyframes; eight actual source cases compare exact WARN; Rust also retains issue and partial slot count | Other creation failure categories |
| Missing bounds / undetected layout | Map-aware Rust source API requires a map and creates its layout from it; lower-level creation APIs expose optional bounds/layout results | Do not equate static input requirements with completion of all native optional APIs |
| No readable chunks | Loaded Rust byte slices cannot fail a later file read; nonempty census proves a readable data prefix here | Lazy FilmSource/IO wrapper behavior remains independently audited |
| Position failure | Supplied map/layout and nonempty band cover native production setup; Rust additionally rejects invalid map/scan configuration | Exact source error observations for other supported failures |
| Missing biped band for aim | Additive failure now retained and warned; eight failing and ten original healthy source cases plus delta controls | Nonzero source aim summary evidence |
| Event failure | Native normal error is no readable chunks; malformed reference parsing can panic. Rust returns a bounded truncation error and preserves it in issues | Recovery diagnostics are intentionally safer than reproducing a panic; do not falsely call them identical native error paths |
| Death-read failure | Native ScanMarchFacts returns registry-resolution errors; Rust map constructor already resolves registry and separately rejects unsupported majors/invalid encoding | Full source-wrapper failure equivalence remains open |

The native source stage restores MPP with a defer even on early return. The new
fixture covers registry truncation by retaining 33 through 40 complete blocks of
the existing captured v41 registry. Vehicle creation warnings preserve native
wording for the known missing-archetype error. Other Rust creation errors use
Rust's diagnostic display; exact wording parity for those is not established.

Source pointer/cache identity is not recording data. Nevertheless, lazy error
order, profile values and emitted observations can affect behavior and remain
part of the broader context audit. No next-phase Film/ResolvedFilm/playback
architecture was introduced by this work.

## Positive aim source coverage

The 120-case source fixture now adds 32 nonempty occupant-aim results, one per
case, while rejecting an out-of-band control packet in every case. Native
packet ordinals, timestamps and raw aim quanta match Rust alongside ordered
source logs and the nonzero summary count. All 88 prior rows are unchanged.
This supersedes the table's missing nonzero source-aim evidence, not its other
source/context gates. Positive death and occupancy summaries remain open.

## Positive death and occupancy source coverage

The 152-case source fixture preserves all 120 prior cases and adds 32 sources
with bound vehicle/biped records: true then false dead state and attached then
detached parent state. Native exact-actor/time assertions verify the controls.
The new cases run Rust's full chronological march rather than injecting native
facts, comparing all facts, coverage maps, event/located counts and calibration
retention. They yield 32 vehicle deaths and 64 occupancy readings, with one
additional biped death retained from the composed inputs. Logs and positive
summary counts match. This supersedes the remaining positive death/occupancy
summary gap above; general source failures and shared-context API gates remain.
