# Recorded movement inputs

Contextual replay exposes every complete kind-0 control record through
`PlayerReplaySet::diagnostics.inputs`. Each observation retains its original
`ControlInput`, baseline, exact bit range and fields, packet address, relative
clock, and current human/bot population and assignment sources. Unique holders
also receive indices in `PlayerReplay::input_observations`; ambiguous, unknown
and unassigned records remain in diagnostics. Later roster listings never
relabel earlier observations.

`InputSchema::Reference8f0f8d0` labels the reference convention: subtract 31
from each unsigned six-bit analog word to obtain forward/left. Code 63 produces
+32. The optional five-bit word is exposed as `stance_code`, without assigning
crouch/sprint meanings. Auxiliary, action and secondary fields remain available
in the raw control. These values describe recorded controls, not velocity,
physical key presses or animation.

`input_samples` and `input_at(relative_timestamp_us)` expose the latest supported
analog observation. Repeated identical observations are retained. An absent
analog block is recorded but does not create neutral movement or replace the
previous analog sample. A present block without stance records `None`, rather
than inheriting stance. Neutral analog is explicitly `[31, 31]`. Before any
sample, `input_at` returns `None`; an unknown boundary returns a sample with
`values` and `observation` both `None`. Equal-time ordering follows source order.

Complete entries preceding an unread control suffix remain inspectable, then
input support ends. Opaque non-frame packet bodies also end input support,
including types 6 and 10. `diagnostics.input_gaps` records these input-specific
boundaries; general replay gaps remain in `diagnostics.gaps`. An untimed opaque
packet has no invented timestamp: its exclusive support bound is the preceding
packet clock. Population loss/conflict/reassignment, existing replay lifecycle
clears, structural source gaps and the requested replay endpoint also clear
sampled input. No historical observations are discarded. These conservative
boundaries do not prove that a held control changed in the engine.

Inputs resolve the recorded engine slot through independent human and bot
populations. They do not require or create a biped lifetime. Source loss that
could hide either population requires new independent population observations.
Entity ownership and input evidence remain separate; a known participant can
have controls even when no avatar is known.

The layer lives in `runtime/replay/inputs.rs` and consumes the existing
`parser/v41` contextual control reader and `film` models. It does not change
canonical wire admission, schema recovery, map selection or the runtime playhead.
It inherits contextual replay's explicit assumptions; it is not proof of a
complete canonical decoder.

## Reference and validation

The [57-case fixture](../fixtures/inputs-v41/README.md) executes unchanged pinned
libfilmshell control readers. It covers all stance words, analog extrema,
optional fields and baseline presence. Local tests compare exact field bytes,
endpoints and reference values, while preserving repeats that upstream deduplicates.
Separate authored integration tests exercise source ordering, slot reuse,
human/bot conflicts, neutral versus absent input, partial suffixes, untimed and
timed opaque packets, source loss and Film/playhead immutability. Retained
capture audits validate original source ranges and population references; see
[VALIDATION.md](VALIDATION.md). Continuous key-held intervals and full match
coverage are not established by these checks.
