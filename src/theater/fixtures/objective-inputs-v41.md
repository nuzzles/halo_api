# Objective consumer input controls on captured films

Pinned reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
`halo_rust_full_document_test.go.txt::TestHaloRustObjectiveInputs` uses the same
six recorded films/map bounds as the complete-document matrix. It explicitly
opens/closes the resolved consumer guards for zones, flag gauges/marks, and bomb
radial reads. This does not assert that these recordings are CTF or Assault.
Flag-mode recognition is outside this control: enabled marks call the same native
ScanCarrierMarks routine that recognized flag consumers call. Zone/gauge reads
use the native shared ti13 reader. Bomb reads use decodeFilmBombReads with the
actual manifest clocks. Disabled variants expect native empty inputs.

The enabled paths retain 16,569 managed-property reads per shared consumer and
13,805 radial-progress reads. Carrier marks are empty, but the enabled keyframe
and record denominators remain compared. Positive captured flag marks and actual
Assault action validation remain gaps. Native generation passed in 15.692s.
All fields and ordered reads match the Rust retained-input projection; no Rust
output generated the expectations. No semantic flag/bomb actions are inferred.

The separate compressed fixture is linked from each full-document row. All prior
document expectations were verified unchanged. The six-film run comparing all
44 FilmInputs fields, these controls, and full documents passed in 101.76s.
The full Theater suite passed 561 tests with 46 ignored (107.28s); Clippy and
all-features WASM passed. This is integrated input evidence, not complete parity.

The audit also fixed managed-property replay consumption: zone and flag-gauge
consumers now suppress reads when managed_properties_error is present, matching
native failure handling. Film retains the partial scan and error unchanged.
Positive constructor-oracle cases additionally exercise injected partial/error
pairs and serialization. Those injections test adapter behavior, not evidence
that current source failures occur after emitting those reads.
