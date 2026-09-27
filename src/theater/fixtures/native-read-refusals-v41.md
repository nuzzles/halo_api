# Bounded component and view read refusals

The shared component Reader now retains a `NativeReadRefusal` when a scalar
cursor returns None. The diagnostic preserves field name, signed bit position,
requested width, source length and operation. It is not a scalar with value zero.
Control views additionally retain failures of their native grouped guards, which
happen before any group member is consumed. `DecodedFrameView` now carries the
reader diagnostics into production frames and the native film entry.

This does not change bounds, widths, padding policy, stop categories, or cursor
movement. A padded read that succeeds beyond source does not become a refused
read. Scalar refusal is not necessarily source truncation: a bounded cursor also
refuses unsupported widths or positions. Existing width-limit diagnostics remain
separate. This covers operations through the shared scalar reader and the two
control group guards, not every early constructor/header failure in the module.

Tests:
- Manually encoded control records assert a 5-bit scalar failure at bit 4 and a
  13-bit grouped failure at bit 11 without consuming the available group prefix.
- A bounded loadout read retains its first byte and reports the next missing byte.
- Unsupported message bodies and empty messages must not produce false refusals.
- Diagnostic merge and JSON round trips retain failures; older view JSON defaults
  to absent diagnostics. Views allocate diagnostic storage only when nonempty.
- A type-0 payload through NativeFilmData retains its failed 7-bit selector read
  at payload bit 2, alongside its successful presence field.
- Existing pinned view/control oracles verify unchanged endpoints, kinds and
  completion, including prefixes and native signed-width behavior.

Reproduce:

```sh
CARGO_INCREMENTAL=0 cargo test --release --lib bounded_read_refusals -- --nocapture
CARGO_INCREMENTAL=0 cargo test --release --lib components::views -- --nocapture
CARGO_INCREMENTAL=0 cargo test --release --lib native_signed_width
CARGO_INCREMENTAL=0 cargo test --release --lib native_data_ -- --include-ignored --nocapture
```
