# Statborg file JSON payload

`facts_statborg_json.rs` ports native `replay.FilmStatborg`, `types.StatRecord`
and `types.StatValue` with declaration-order JSON. Nil slices/maps remain distinct
from empty values. Native int fields and signed map keys use i64 on both host and
WASM; StatValue's explicitly declared int64 fields retain that diagnostic name.
Signed JSON map keys follow Go ParseInt base-10 semantics (including signs and
leading zeros) and overflow checks. Writers sort decimal key strings lexically.
Repeated map values start from zero; repeated successful sections retain native
struct and slice backing state. Errors poison the section reader.

The independent native generator covers 256 writer cases and 767 read sequences,
including signed limits, duplicate normalized keys, malformed keys and values,
nested error precedence, nil collections and repeated section state. Host shared
regressions passed before the subsequent killsource/file additions. Final combined
platform validation is recorded in `facts-file-validation.json`.
