# Named-life export oracle

Pinned reference: JGtm/LevelUp feat/v75,
43a01721e8a02c87c955e175936f0ccf8dd97a81, replay/lives_export.go.
Generator: ../reference/halo_rust_named_life_export_test.go.txt.

Calls the actual native IdentityRegistry.ViesNommees method in 512 cases with
10,674 input lives and 6,396 output lives. 128 cases disable the native
DeathsNamed gate. Inputs include anonymous identities, repeated slots, shuffled
life times, signed submillisecond bounds, signed clock offsets including i64
extremes, and exact sort-key ties with distinct cause/naming provenance.

Rust compares every ordered field and portable serialization. Expectations come
from the pinned Go method; the Rust implementation does not generate them.
The export omits anonymous lives, applies truncating millisecond conversion
before wrapping offset subtraction, and stably sorts start/XUID/end. A nonzero
death-naming count is required even when another route supplied named lives.

This verifies the existing derived identity export, not independently annotated
life boundaries, identity truth, a new canonical Film hierarchy, or a general
claim of lossless recording export. The original registry lives remain intact.
