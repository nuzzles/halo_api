# Cached grapple and active equipment episodes

build_facts_replay_grapple consumes exact cache time, slot, heavy/light and u32
coordinate quanta. It shares the recording path's pure pairing/arrival reducer;
no synthetic source records are constructed. The native 500ms pairing window,
attachment/fire/nearest-life precedence, 2.5s arrival search, tie order and published
life clamps remain shared.

The extended 1,024-case Go oracle includes all-width quanta and u64 timestamp
extremes. It exposed an existing distance-rounding mismatch at case 749: Rust
previously picked arrival frame 39 instead of native frame 65. Native Go/arm64
compiles dx*dx + dy*dy + dz*dz as x*x followed by fused y*y and fused z*z. The
shared Rust reducer now uses dz.mul_add(dz, dy.mul_add(dy, dx*dx)), retaining that
rounding order on WASM too. The evidence came from the pinned native result and
Go tool compile -S of the same float32 expression (FMULS, FMADDS, FMADDS).

build_facts_replay_equipment_episodes accepts cache positions and camo readings.
It selects only positions with has_shield, sorts them chronologically as native
assembly does, and uses shield_quantum rather than the scalar shield value. It
shares the life/death-aware accumulator with recording inputs. Camo readings other
than 0/4095 count as nonbinary without changing active state; readings for absent
published slots do not inflate that counter. Gaps, death closure and life windows
retain the existing native rules.

A separate 1,024-case native episode oracle preserves unsorted original position
inputs, inactive shield quanta, all-byte shield values and out-of-range u16 camo
values. It compares all episode and coverage fields. Native source tests remain
separate regression checks. Platform results are in facts-state-inputs-validation.json.

The WASM harness accepts repeated --test-prefix options so these two adapters can
be checked together without rerunning unrelated oracles. A selected pass is not a
full-suite pass. Cache/player/document composition, remaining impulse/charge and
movement adapters, capture/fallback order and the final parity audit remain open.
