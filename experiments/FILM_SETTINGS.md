# Match settings lookup

For `9a875c4e-03bc-4fff-a688-e21216d1618b`, match history links to the exact
**FFA Ockygon** UGC revision:

- Asset: `02cfa398-6462-4a90-b8be-a6ff8cddd1fb`
- Version: `107f4bc6-56da-4597-9ce6-6b79938de538`
- Base engine mode: **Slayer-CompetitiveFFA**, asset
  `499b0c55-7022-4e8f-b9a1-83389b6ed35d`, version
  `23f79db8-ffbb-4eb6-9d92-3443248673de`.

The saved variant's `CustomData.KeyValues` has nine overrides. Two directly
relevant values are `globalMalleableProperties.InfiniteAmmo.uint_value = 1`
and `globalmalleableproperties.bottomlessclip.uint_value = 1`. The English
custom-game menu confirms **Off / On** for these options, with **On** corresponding
to true/1. This independently supports the recorder's bottomless-magazine report.

There is **no established descope setting** yet. No descope/unzoom/flinch option
was found in the saved overrides or extracted menu parameters. The engine binary
contains additional numeric fields whose semantic names are unresolved. Absence
from the override list does not establish an inherited default. This is the
referenced saved variant, not proof of every effective setting after lobby edits
or Forge traits. The replay does not consume these API settings or synthesize
scope changes, starting weapons, or ammo values from them.

## Reproduce

From `experiments/`, using the existing Xbox authentication environment:

```sh
HALO_TIMEOUT_SECS=120 cargo run --example download_match_settings -- \
  9a875c4e-03bc-4fff-a688-e21216d1618b films/octagon/03-first-to-50/settings
```

An optional third argument supplies a participant's XUID; otherwise it uses the
logged-in account. The example searches up to 1,000 recent matches through
`player_matches`, avoiding the slow scoreboard endpoint. It caches the matched
history entry, follows immutable version links, saves raw JSON, exports
`variant-overrides.csv`, and downloads the engine binary and English menu files.
Raw union members are preserved; arbitrary `uint_value` fields must not be
treated as literal seconds, weapon IDs, or scores without decoding their schema.

Existing typed API entry points are `player_matches` (or `match_stats`) and
`mode(GameModeId::new(asset_id, version_id))`. `mode().custom_data.key_values`
already exposes the override object. The experiment preserves the full response
because the current typed asset models omit `EngineGameVariantLink`, and the
unversioned `engine_game_variant()` method would retrieve the latest revision.

## Captured evidence

Files are under [the clip's settings folder](films/octagon/03-first-to-50/settings):

- `match-history-entry.json`, `game-variant.json`, `engine-variant.json`, and
  provenance files preserve the API linkage and retrieval time.
- `engine-files/CompetitiveFFA.bin`: 411,201 bytes. An exploratory Bond Compact
  Binary v2 structural reader consumes the entire file, with checked container
  lengths. Numeric field IDs still need semantic schemas.
- `engine-files/CustomGamesUIMarkup/Slayer_CustomGamesUIMarkup_en.bin`: 98,304
  bytes; a complete 83,041-byte Bond structure followed by zero padding.
- `menu-parameters.json`: 114 extracted enumerated menu parameters, names,
  descriptions, choices, and source byte offsets. This is a schema snapshot,
  not a list of settings effective in the match. The structural `*-bond.json`
  dumps retain field IDs/types/offsets for further research.

The Infinite Ammo menu parameter occupies bytes `[36008,36230)`, and Bottomless
Clip `[36230,36452)`. The latter's Off and On entries begin at 36421 and 36437.
The exploratory Bond dumps are research artifacts; the download example retains
the original bytes and does not yet implement a supported settings decoder.
