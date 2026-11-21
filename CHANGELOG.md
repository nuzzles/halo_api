# Changelog

<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

## [Unreleased]

### Added

- Offline Theater player metadata: the initial 32-slot player table, timestamped roster and bot
  snapshots, and recorded Spartan appearance hashes for recognized HI_1_12_0/HI_1_13_0 blocks.
  Original appearance bytes are retained; item names, asset paths, and current API customization
  are not substituted for recorded data.
- Recorded match settings, including variant name/description, score/time/respawn limits,
  starting loadouts, and typed option values with original bytes.
- Typed reload, throw, melee, pickup, zoom, teleport, projectile impact, and detonation events.
  Positions retain quantized codes; gameplay state and projectile attachment interpretation
  remain outside the parser. `FilmRuntime` retains the latest complete roster/bot snapshots.

### Changed

- `Registry::players` is a required `Vec<InitialPlayer>`. Film parsing returns
  `ParseError::IncompleteRegistryPlayers` if the complete initial player table cannot be decoded,
  instead of succeeding with missing player metadata.
- `Film::parse_with_options` accepts `FilmParseOptions` to select single- or multi-threaded
  Theater decoding; `Film::parse` keeps the multi-threaded default. Film parsing benchmarks now
  measure both modes with 25 trials per fixture. Film chunk inflation now uses `zlib-rs`, which
  uses SIMD on supported processors.
- `player_customizations` now returns a `Vec<PlayerCustomization>` with typed player IDs,
  direct appearance and core fields, selected themes, and named weapon slots. Serialized
  customization output uses normalized field names and timestamps instead of Halo's response
  wrappers and result codes.
- Theater film decoding now processes independent replication chunks in parallel while preserving
  recorded observations and parser fallbacks. Reproducible benchmarks and a results site are in
  `benchmarks/`.

## [0.6.1] - 2026-11-20

This release has an [MSRV][] of 1.96.

### Added

- The `xbox` feature re-exports the Xbox crate as `halo_api::xbox`, so consumers can use
  `halo_api`'s Xbox authentication helpers without declaring a separate Xbox dependency.

## [0.6.0] - 2026-11-20

This release has an [MSRV][] of 1.96.

### Added

- Decode Halo Infinite Theater chunks into a typed `Film` with registry, replication stream,
  and summary highlights. `FilmRuntime` replays observations into seekable world snapshots;
  the `film_world_inspector` example explores entities and their components.
- `Medal::code()` returns the recorded Theater summary code for known and unknown medals.
- The optional `xbox` feature provides `HaloAuthClient::from_xbox_xsts_source` and
  `XboxXstsTicketSource` for `xbox 0.3.1`.

### Changed

- Xbox authentication is optional. The default build has no Xbox dependency; callers can supply
  an owned `XstsTicketSource` through `HaloAuthClient::from_xsts_source`, or Halo credentials
  through `from_tokens`. Replace `from_xbox_client` with `from_xbox_xsts_source` when using the
  `xbox` feature.
- `Player::Xuid` now contains `halo_api::clients::hi::Xuid`; callers using Xbox's XUID type
  should pass its string value to `Player::xuid`.
- Xbox-backed examples use `xbox 0.3.1` as a dev dependency and require the `xbox` feature.
- Medal-flagged summary highlights with unrecognized type codes now use
  `SummaryHighlightPayload::Medal(Medal::Unknown { type_code, medal_code })` instead of the
  generic `Unknown` payload.
- Reduced Theater film decoding overhead when parsing replication fields and compressed chunks.

## [0.5.0] - 2026-11-18

This release has an [MSRV][] of 1.96.

### Added

- `TheaterError`, exposed through `InfiniteClientError::Theater`, groups Theater-specific failures.
- `TheaterError::FilmUnavailable` identifies a match-film 404 without parsing error text.
  LAN/local games do not have downloadable Theater films; missing or expired recordings can also
  produce this error. Highlight-event methods propagate the same error.

### Changed

- Film decompression failures now return `InfiniteClientError::Theater(TheaterError::Decompression)`
  instead of `InfiniteClientError::FilmDecompression`.
- Halo Infinite requests are sent without a fixed per-second rate limit. A 429 response pauses
  requests to the same origin across client clones and retries up to 20 total attempts by default.
  Exhaustion returns `InfiniteClientError::RateLimitExhausted`; the attempt limit is configurable
  with `HaloInfiniteClientBuilder::max_rate_limit_attempts`.
  Credible `Retry-After` delay-seconds and HTTP dates are respected; otherwise the fallback delay
  doubles from two seconds, up to one minute.
- Removed the `requests_per_second` and `rate_limit_retries` builder options.

## [0.4.0] - 2026-10-23

This release has an [MSRV][] of 1.96.

### Added

- `WaypointClient::hcs_leaderboard` reads Waypoint's public HCS leaderboard and returns gamertags with points.
- `HaloInfiniteClientPool` and `HaloInfiniteClientBuilder::build_pool` support round-robin
  selection across multiple Halo accounts.

## [0.3.1] - 2026-09-27

This release has an [MSRV][] of 1.96.

### Fixed

- Models are now `Serialize`.

## [0.3.0] - 2026-08-03

This release has an [MSRV][] of 1.96.

### Added

- `HaloInfiniteClientBuilder::rate_limit_retries` configures how many times a rate-limited (429)
  response is retried before giving up. Defaults to 3.

### Changed

- Requests that hit a 429 (rate limited) response are now retried automatically, up to
  [`rate_limit_retries`][] times (3 by default), instead of surfacing immediately as an error.
  Each retry backs off using Waypoint's `Retry-After` header when present, falling back to
  exponential delays starting at 1s. The cooldown is applied to the shared per-origin rate
  limiter, so concurrent requests to the same origin also pause rather than immediately
  re-triggering the limit.

[`rate_limit_retries`]: https://docs.rs/halo_api/latest/halo_api/clients/hi/struct.HaloInfiniteClientBuilder.html#method.rate_limit_retries

## [0.2.0] - 2026-07-30

This release has an [MSRV][] of 1.96.

### Changed

- `current_ranked_arena` now fetches all rotation entries (map-mode pairs, maps, and game variants)
  concurrently rather than sequentially. For a typical 26-entry rotation this reduces wall-clock
  time from ~16 s at the default 5 req/s rate to the time of the single slowest request.
- Raised the default per-origin request rate from 5/s to 9/s.

## [0.1.1] - 2026-07-30

This release has an [MSRV][] of 1.96.

### Fixed

- `hero_url`/`thumbnail_url` on `MapAsset`, `GameVariantAsset`, and `PlaylistAsset` now find the
  conventionally-named image regardless of its file extension. Halo serves some assets' hero and
  thumbnail images as `.jpg`/`.jpeg` rather than `.png` (observed on several Ranked Arena rotation
  maps), and the previous exact-filename match on `hero.png`/`thumbnail.png` silently returned
  `None` for those, even though an image existed.

## [0.1.0] - 2026-07-30

This release has an [MSRV][] of 1.96.

### Added

- Initial release: `HaloAuthClient` for Xbox Live sign-in and Halo Waypoint credential acquisition
  (Spartan token and flight clearance), or `HaloAuthClient::from_tokens` for applications that
  already manage token acquisition themselves.
- `HaloInfiniteClient` covering stats, skill, profile, UGC, progression, career rank, reward
  tracks, challenge decks, ban, and privacy endpoints, plus a `Player` type accepting a gamertag
  or XUID interchangeably and a `MatchHistoryPager` for walking full match history.
- Per-origin request pacing, automatic 401 invalidation and retry, and a `ServiceRecordFilter` for
  scoping service records by season, playlist, mode, and ranked/social.
- An experimental Theater film decoder for match highlight events.

[MSRV]: README.md#msrv

[Unreleased]: https://github.com/nuzzles/halo_api/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/nuzzles/halo_api/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/nuzzles/halo_api/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/nuzzles/halo_api/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/nuzzles/halo_api/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/nuzzles/halo_api/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/nuzzles/halo_api/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/nuzzles/halo_api/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/nuzzles/halo_api/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nuzzles/halo_api/releases/tag/v0.1.0
