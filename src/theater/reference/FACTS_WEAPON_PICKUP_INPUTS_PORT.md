# Cached weapon changes and pickups

build_facts_replay_weapon_changes and build_facts_replay_pickups consume exact
cache inputs through the same pure reductions as recording inputs. Neither
constructs synthetic source records or narrows cache-only fields into wire types.

Weapon publication excludes restated before the origin check, maps dropped and
swapped explicitly, and maps every other kind to taken. Unknown/invalid-UTF-8 kind
bytes and signed slot indices stay in FactsWeaponChange; the native replay does
not publish them. All-ones family IDs remain absent, while zero remains a real
family. Publication keeps scan order and imposes no track or end-frame gate.

Pickup publication uses the caller's catalog, occupant and origin judge. It keeps
all byte-sized classes, abstains from unrelated catalogs, omits before-origin
events, and preserves the no-end-frame-clamp rule. ReplayPickupCoverage.multi_event
and refused now use i64. The sum of the three refusal counters wraps at 64 bits,
matching native integer arithmetic even on WASM. Other counts are derived from
actual collections and remain host-sized.

Two independent Go generators each produce 1,024 cases. Weapon cases include
signed slot-index extrema, arbitrary kind bytes, timestamp wrapping, zero-step
and family sentinels. Pickup cases cover every class byte, signed counter extrema
and overflow, optional occupants and origin judges, catalog selection and coverage.
See facts-weapon-pickup-inputs-validation.json for executed platform checks.

These are cache input adapters. Remaining active-state, impulse/charge, placement,
vehicle/objective and full document integration, capture/fallback handling and
final all-data v41 audit remain pending.
