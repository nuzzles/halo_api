# Changelog

<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

## [Unreleased]

This release has an [MSRV][] of 1.96.

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

[Unreleased]: https://github.com/nuzzles/halo_api/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/nuzzles/halo_api/releases/tag/v0.1.0
