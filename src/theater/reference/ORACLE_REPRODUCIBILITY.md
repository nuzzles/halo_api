# Native oracle registration audit

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.

The generator audit found fourteen retained fixture harnesses whose native tests
were not invoked by generate_oracles.py. Counting literal harness filenames alone
overstates the problem: many installation and invocation paths use static loops
and computed names. ReplayKillInputs also has a separate subprocess invocation.
MovementTrace is an intentional captured-film diagnostic, with no retained golden;
it remains a manually selected probe rather than a fixture-regeneration gate.

SOURCE_PUBLICATION_ORACLES now declares the fourteen missing harnesses, packages,
test names, output paths and retained fixture destinations. The main generator
calls generate_source_publication_oracles after the grammar/source producers.
This covers context construction, profile warnings, roster strings, named-life
exports, navpoint sources, vehicle coverage/source/death observations, bridge
warnings, death context, round identity queries, kill positions, map entries and
the kill starting profile.

NavpointSource previously read an undeclared /private/tmp/halo-navpoint-primitive.json.
Its input is now explicitly mapped to the earlier navpoint-radial-scan output.
VehicleSourceLogs similarly uses the generated vehicle-creation/scan outputs,
the configured corpus and pinned map catalog through existing path remapping.
The runner does not depend on these old host temporary paths.

## Verification

All fourteen registered tests were executed through the new runner using the
pinned native checkout. Dependency inputs were staged from the retained native
fixtures. Complete parsed JSON values, including every array's order, matched
the existing outputs. No retained expectation was overwritten. The runner's
write_fixtures=False mode permits this comparison before accepting regeneration.
Native log and report: /private/tmp/halo-source-publication-regeneration.log and
/private/tmp/halo-source-publication-regeneration/report.json.

source-publication-regeneration.json records the fourteen outcomes and fixture/
harness hashes. Generator syntax, diff checks and all 485 reference source hashes
passed. Rust source and included fixtures did not change; the preceding full
Theater/Clippy/WASM results remain the applicable parser baseline.

This closes the identified registration omissions. It does not claim a complete
fresh end-to-end execution of the much larger generator, independently annotated
semantic goldens, full parser parity, or the deferred architecture requirements.
