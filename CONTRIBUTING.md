# Contributing to halo_api

Thanks for considering a contribution. This is a solo-maintained, community library — PRs and issues are welcome.

## Getting started

```sh
git clone https://github.com/nuzzles/halo_api
cd halo_api
cargo build
cargo test --all-features
```

## Before opening a PR

Run the same checks CI runs:

```sh
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
cargo doc --no-deps
```

All of these must pass. If `cargo fmt` reports diffs, run `cargo fmt` (without `--check`) to fix them.

## Coding conventions

- No `unwrap()`/`expect()` in library code outside of tests — return the owning client's typed error.
- New public types/functions should have doc comments; non-obvious behavior (auth quirks, API response shapes,
  retry semantics) should explain *why*, not just restate the signature.
- New Infinite endpoints, response models, and constants belong under `src/clients/hi`.
  Don't inline response structs at call sites — this crate exists specifically to stop that pattern from spreading.
- New authenticated HTTP call sites should go through `HaloInfiniteClient`'s existing credential handling and
  401-retry wrapper rather than hand-rolling their own `reqwest` call.

## Filing issues

Bug reports should include the crate version, a minimal reproduction if possible, and what you expected vs.
what happened. Feature requests should describe the use case, not just the desired API shape.

## Branches and commits

- Branch from `main`, name branches descriptively (e.g. `feat/match-stats-endpoint`).
- Keep commits focused; a clear commit message beats a long diff with a vague one.

## Review and merge

Open a PR against `main`. CI must be green. A maintainer will review and may ask for changes before merging.
