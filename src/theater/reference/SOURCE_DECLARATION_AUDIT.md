# Pinned production declaration inventory

Reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81.

inventory_source_declarations.go.txt parses exactly the production paths from
levelup-port-manifest.json with Go's AST parser. It verifies every source SHA-256
before classifying the file, rejects duplicate manifest paths and records package,
imports and all named function/method/type/constant/variable declarations.
source-declarations-v41.json contains 485 files and 5,253 named declarations.
These counts describe inventory, not implementation progress or test coverage.

Eighteen files have neither imports nor code declarations. Five still had stale
pending entries: the two facts revision chronology files and three research
package documentation files. Those entries now say reference-only. Their hashes
remain recorded; no executable parser work is represented by those files.
Files with imports are not classified as documentation-only, since imports can
have initialization effects even without other declarations.

internal/facts/rev.go has exactly one code declaration: Rev. It maps to
replay_decoder_coverage.rs::NATIVE_FACTS_REVISION and the published facts_rev
field. native_decoder_provenance and native_public_constant_contracts both
passed against the current code (0.03s; /private/tmp/halo-revision-audit-tests.log).
The manifest now records that constant as ported-v41. The revision identifies
the pinned native algorithms, not the Rust implementation's completion status.
Native production database backlog policy is not encoded by that constant port.

Regenerate after placing the retained helper in a file ending in .go:

```sh
GOCACHE=/private/tmp/halo-go-cache /private/tmp/halo-go-runtime/go/bin/go run \
  /private/tmp/halo-native-source-declarations.go \
  /private/tmp/halo-levelup-v75/apps/go-api/internal/games/halo_infinite/film \
  src/theater/reference/levelup-port-manifest.json \
  src/theater/reference/source-declarations-v41.json
```

This inventory does not classify every remaining file as required, implemented,
or irrelevant. In particular, code-bearing research tools, native cache codecs,
external asset readers and replay algorithms require their own scope and field
audits. It is not a reason to replace native recording parity with cache or
document agreement. The architecture in NEXT_PHASE.md remains deferred.
