# Incomplete v41 profile warning

Pinned reference: LevelUp feat/v75, 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_profile_warning_test.go.txt.
Fixture: profile-warning-v41.json.

128 cases call actual profile.Resoudre and journaliserProfilIncomplet with
source.Load registry-present or data-only inputs. Format keys include known
formats with unavailable MPP widths and unknown formats. Build keys cover both
supported v41 builds, an unknown printable ASCII build, and missing identity.
The profile supplied to the logger is independent of the registry-presence
gate, as in the native function signature. Source.Load rejects an empty source;
the missing-registry controls therefore use a loaded data-only source.

There are 44 WARN records and 84 silent controls. Native slog wall-clock time
is removed; message, severity, exact joined error text, format and build are
compared. Rust profiles additionally carry MissingMap in these cases, proving
that map-only issues do not produce this warning. Error order is format then
build, joined by a newline. Identification accepts printable ASCII build fields.

Rust map-context construction invokes the same warning immediately after profile
resolution. Generic profile resolution remains pure; base map-free decoding
does not acquire the map-constructor warning. This is warning-boundary evidence,
not proof of all constructor-wide logging order or lazy context behavior.
