# Changelog

<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

## [Unreleased]

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

[Unreleased]: https://github.com/nuzzles/halo_api/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/nuzzles/halo_api/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/nuzzles/halo_api/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/nuzzles/halo_api/releases/tag/v0.1.0
