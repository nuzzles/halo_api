# Reloads, weapon selection, and magazine ammo

## Stalker Rifle firing control

`9ea11b0a-dddc-4061-a7f7-ac22fa62a05b` (`weapons/05-ar-stalker-rifle`,
**AR & Stalker Rifle**) directly calibrates Stalker Rifle **`daf193c7`** and
independently confirms Assault Rifle **`48c19d2d`**. The 90.740-second recording
contains one Stalker firing event followed by three AR firing events, in the
order reported by the recorder. All four event sequence values are consecutive.

| Film time | Recorded result |
| ---: | --- |
| 56.957497 s | Select secondary, corresponding to the reported 0:20 switch |
| 59.310053 s | Stalker shot: `3daf193c74`, slot 1; no magazine quantity |
| 67.050945 s | Select primary, corresponding to the reported 0:10 switch |
| 68.569291 s | AR shot: `148c19d2d4`, slot 0; magazine 35 |
| 68.652670 s | AR shot; magazine 34 |
| 68.736064 s | AR shot; magazine 33 |

Spawn references `b48c19d2d4` and `bdaf193c74` corroborate the reported loadout;
their fingerprint ranges are 1309:1341 and 1471:1503 in chunk 1's packet at
payload byte 501925. Firing supplies the actual replay identity binding. The
new fingerprint also names **279 existing raid shots**, 10 in primary and 269
in secondary. No Stalker shots were found in either retained Ranked game.

The return switch exposed a previously blocking field: archetype 35 component
**35**, `weapon-state-overheated`. There are **496 checked records** in this
control where it has a nine-bit form: upper seven bits decrease **21 → 0**,
followed by `00`. The first observation coincides with the Stalker shot; zero
appears at 67.568061 s. That is consistent with cooling, but the physical scale,
overheat threshold, other flags and energy/battery ammo remain uncalibrated.

At the return switch, the sparse component list is `[35,42]`. The nine-bit
component occupies bits 70:79 (`000001000`), followed by the established
primary-selection form `0010001` at 79:86 and the exact End/input marker.
Source: chunk 4, payload byte 503906. The decoder consumes only the observed
component-35 forms (upper value 0–21, trailing `00`) when walking supported
weapon fields. It rejects other values or guards and still checks the following
record. The registry name is validated too. No heat or ammo value is exported
from this field, and standalone cooling records do not become weapon events.

In replay, both switches clear the held identity; the next recorded shot supplies
the gun name. Stalker ammo remains `?`. AR shows 35 → 34 → 33 from its magazine
records. Initial inventory is still unknown. This behavior is independent of
match IDs, map names and reported loadouts.

Evidence is under `films/analysis/stalker-rifle/`: the original spawn bytes,
all 496 component records, the controlled event summary and corpus audit.
Captured-byte fixtures are `../src/theater/fixtures/weapon_identity_records.json`
and `stalker_weapon_records.json`; they cover the mixed cooling/selection record,
invalid flags, unsupported values, truncation and absent energy-ammo output.

The 32-film refresh preserves all previous magazine/selection observations and
every unrelated player stream. Consuming the checked component-35 form also
unlocks **17 older magazine observations** (one in each Ranked game, 15 in the
raid) and **18 raid selections**. With this new control, the corpus totals are
**19,723 magazine observations** and **1,338 selections**. These additions are
listed with player/life/source evidence in `stalker-rifle/corpus-validation.json`.

## Weapon fingerprints and both selection directions

The BR75/Shock Rifle control, `731ef1f7-a119-49ca-a056-4b8305aaf7c6`
(`weapons/04-br-shock-rifle`), separates the carried slot from the weapon
fingerprint. The old 40-bit firing window includes both, plus checked guards:

| Firing-record bit | Width | Meaning |
| ---: | ---: | --- |
| 40 | 3 | Zero-based carried slot, currently checked for 0 or 1 |
| 43 | 1 | Guard `1` |
| 44 | 32 | Weapon fingerprint |
| 76 | 4 | Guard `0100` |

For example, `12b1824d54` is BR75 in slot 0 and `32b1824d54` is BR75 in
slot 1. `Firing::weapon_slot()` and `Firing::weapon_fingerprint()` expose these
checked fields; the complete window remains in JSON for compatibility.
Unsupported wrappers or slot values return `None`. The decoder checks the slot
against an independent unique same-packet magazine association when available;
a conflict withholds the slot and adds a diagnostic.

The leading slot matches **17,311 independent firing/magazine associations with
zero mismatches**: 15 in the reload control, 983 in Ranked Bandit, 2,109 in Oddball,
29 in the AR kill control, and 14,175 in the raid. This makes recorded slots
available even when firing has no supported ammo update.

| 32-bit fingerprint | Calibrated name |
| --- | --- |
| `6acdc44d` | Bandit EVO |
| `f408190f` | Pistol |
| `0a1992bc` | S7 Sniper |
| `2b1824d5` | BR75 |
| `9387a8b9` | Shock Rifle |
| `48c19d2d` | Assault Rifle |
| `daf193c7` | Stalker Rifle |

These are recorded fingerprints calibrated against controls, not established
universal asset IDs. The new clip contains no accepted firing events. Its spawn
packet carries `b2b1824d54` and `b9387a8b94`, each followed by the existing
28-bit reference guard. Their fingerprints occupy bits 1330:1362 and 1491:1523
in chunk 1's packet at payload byte 501968. Known Bandit/Pistol, AR/Pistol and
Bandit/S7 controls corroborate the reference ordering. The `b` wrapper is a
different context and must not be interpreted as a firing slot.

**Shock Rifle naming is a spawn-loadout calibration**, based on the recorder's
reported primary BR75 and secondary Shock Rifle; there is no Shock firing
confirmation in this clip or the two Ranked games/raid. The BR fingerprint does
occur in older firing records: 163 Oddball shots, 87 Ranked Bandit shots, and
4,326 raid shots. The older control titled “BR / S7 scope steps” contains the
Bandit fingerprint in its first spawn reference, so its reported BR label is not
used to calibrate BR75. Its description is retained as reported.

| Reported countdown | Film time | Component 42 bits | Decoded selection |
| --- | ---: | --- | --- |
| 0:20, switch to Shock | 57.011430 s | `0010011` | Slot 1 |
| 0:10, switch back to BR | 66.971899 s | `0010001` | Slot 0 |

Both payloads occupy bits 64:71, at chunk 3/byte 529423 and chunk 4/byte 501534
respectively. Only these complete seven-bit forms are supported. They differ at
relative bit 5; the old interpretation that their first three bits encode the
slot was incorrect. An earlier primary-shaped update at 17.034559 s also includes
unsupported component 45 and remains withheld.

The full spawn inventory-to-pawn-slot grammar is still unparsed. Consequently,
the new replay entry shows the recorded slot switches but leaves weapon names
and ammo unknown. Reported loadouts and unbound spawn references do not populate
replay state. The name table applies to checked firing fingerprints elsewhere.

Evidence: `films/analysis/weapon-identities/evidence.json`; captured regression
fixtures: `../src/theater/fixtures/weapon_identity_records.json` and
`weapon_switch_back_records.json`. The refreshed corpus audit is
`films/analysis/weapon-identities/corpus-validation.json`.

The 31-film refresh preserves all previous observations and every unrelated
player stream. It now has **1,318 selected-slot observations**, including 661
new primary observations in the raid, and two additional raid magazine samples
(Orbital: 29 at 452.443784 s, 36 at 478.701547 s). Those two deltas contain
component 42's primary form after the ammo fields; both complete records are now
checked in the switch fixture. No firing/magazine slot conflicts were found.

## Current upstream multiplayer support

The native decoder now exports **19,723 magazine observations** across the 32
retained films, including **2,306 in Ranked Oddball**, **1,071 in Ranked Bandit**,
and **16,288 in the raid**. Every player in both Ranked games has observations.
The AR kill control now has 33 observations, Bazaar has four, and the original
17 reload-control observations are preserved. Bottomless Octagon gameplay still
has no magazine observations; replay does not invent them from match settings.

The missing Ranked ammo was principally a **record-boundary restriction**:
multiplayer weapon deltas usually continue into another pawn's delta, while the
original control parser required the exact End/input marker immediately after
the weapon fields. The native parser now also accepts a following checked pawn
prefix with a matching command tick, or the already-supported isolated vitality
record with its End/input guard. Unsupported neighbors still reject the candidate.
Player/life, sorted component IDs, preceding field widths and scalar guards
remain checked. No match IDs or playlist names select this behavior.

Literal positive magazine amounts keep the ten-bit `0 + amount8 + 1` form;
captured AR and multiplayer sequences extend the accepted range to **1–36**.
Zero remains the two-bit `11`. Energy/heat-based ammo uses other forms and is
withheld. Paired component 31/34 rounds-inventory updates have a checked 11-bit
width in reload refills; the parser consumes that field to reach the next record,
but its value is not exported as reserve ammo. The component registry now checks
those names as well as magazine components 30/33.

For a replay check, select **Ranked Arena · Oddball**, Nuzzles:

| Film time | Recorded magazine |
| ---: | ---: |
| 30.040560 s | 14 |
| 30.424402 s | 13 |
| 30.975107 s | 12 |
| 31.408623 s | 11 |
| 36.113054 s | 15, following the reload start at 34.795188 s |

At 32.377023 s, the accepted firing record has no accepted magazine update.
The replay holds the last **11**, marked stale, without subtracting a round.
It retains the recorded quantity across subsequent firing with the same weapon
fingerprint and slot, and clears it at an observed weapon-set/identity change,
selection or life boundary. This also lets a later refill display when the last
shot's packet omitted a supported ammo observation.

Coverage is partial: missing neighboring grammars, unsupported components,
energy ammo, reserves and starting inventory remain open. Samples are recorded
amounts, not a complete current HUD. The earlier 30-film ammo audit had 646 accepted
slot-1 selections, before support for return-to-primary selections. That re-decoding preserved every previous magazine/selection sample
and all other player streams. The corpus audit is
`films/analysis/ranked-ammo/corpus-validation.json`; independent captured-byte
fixtures are `../src/theater/fixtures/ranked_ammo_records.json`. Native Rust tests
cover refills, zero, larger magazines, two maps/layouts, invalid boundaries,
wrong peer ticks, truncation and unsupported energy/neighbor forms.

## Original controlled investigation

**2026-09-10 · film major version 41.** The controlled match
`4050c25f-3da5-4b16-9a50-2fab9660fed4` establishes a reload-start event and
magazine observations for the Bandit and pistol. Manual and automatic reloads
use the same guarded event form. The event does not establish which input caused
it. Weapon/ammo overlays are now available in Theater Lab; visual confirmation
of these new overlays is pending.

Historically, the later [BR/S7 scope control](FILM_ZOOM.md) raised the catalog to 36 films
and adds a fifth accepted slot-1 selection at 29.394270 s. Reload (344) and
magazine (23) counts are unchanged. The 35-film measurements below describe
the original reload investigation. The legacy parsing limits below describe
that investigation; the current native decoder extends them as documented above.

## Controlled sequence and independent checks

Catalog key: `weapons/02-reload-comparison`, category **Weapon controls**.
The download contains seven chunks, 4,647,201 decompressed bytes, and 90.761 s of
film. The recorder described a one-minute game with these actions:

| Remaining game time | Reported action | Observed film time | Checked result |
| --- | --- | --- | --- |
| 0:55 | Three Bandit shots | 21.936667, 22.353776, 22.754345 s | Three firing events; component 30 values 14, 13, 12 |
| 0:50 | Manual reload | 26.875065 s | Reload-start event; component 30 becomes 15 at 28.192607 s |
| 0:45 | Switch to pistol | 32.246683 s | Component 42 selection form `0010011`; companion event at 32.430241 s |
| 0:40 onward | Shoot pistol until empty; automatic reload | 36.901965–39.337740 s | Twelve firing events; component 33 decreases 11 through 0 |
| After empty | Automatic reload | 39.371112 s | Same reload-start form; component 33 becomes 12 at 40.171841 s |

Film timestamps include pregame time and are not the HUD countdown. No exact
countdown-to-film offset is inferred from the recorder's approximate times.
The initial 15/12 magazine values are not exported before their first observation.

Every one of the 15 firing events shares a packet and player life with exactly
one accepted magazine update. This is independent agreement between the existing
firing decoder and the new component parser. The refill delays are **1.317542 s**
for the Bandit and **0.800729 s** for the pistol, measured from reload start to
recorded magazine refill. They are not full reload animation durations.

The corrected previous control, `83fc43ff-e8d3-4d9d-b747-14c4ffee5a69`
(`weapons/01-switch-bandit-pistol`), is **Bandit EVO → Pistol switching**, not a
grenade throw. It and both original switch-weapon controls independently contain
the same component 42 selection form. There are no accepted reload starts in
any other controlled film, including firing, melee, grenade, and switch controls.

## Reload-start window

Offsets are zero-based bits relative to the candidate start, MSB-first. This is
a checked event window, not a complete record grammar.

| Bit | Width | Interpretation |
| ---: | ---: | --- |
| 0 | 1 | Uninterpreted |
| 1 | 9 | Signature `101001101`, also shared by other event families |
| 10 | 8 | Pawn wire ID |
| 18 | 2 | Generation `01` or `10` |
| 20 | 7 | Reload-start guard **`0001000`** |
| 27 | 5 | Roster index; last bit overlaps the following leading bit |

Life serial is `wire + 256 × (generation − 1)`, checked against the firing
probe's independent spawn/roster/life export. Events outside life bounds, after
recorded death, with different roster identity, or with unchecked continuations
are withheld.

At bit 31 the automatic reload is followed by a 37-bit clock-shaped window. In
the manual control, a 64-bit auxiliary record intervenes; its bits 1:32 are
`0100000000000100100001001100000`, with the clock-shaped window at bit 95.
The auxiliary payload remains opaque. The following clock has 26 checked
signature bits at 3:29 and an eight-bit counter at 29:37. Leading `001`, `101`,
and the manual control's `111` are accepted here but left semantically unknown.
This local guard does not change the general clock parser.

Evidence in chunk 2, with byte offsets relative to the decompressed chunk:

| Event | Payload byte | Event bit | Following clock bit | Payload hex |
| --- | ---: | ---: | ---: | --- |
| Manual reload | 501848 | 0 | 95 | `d3401100400484c000000329c0f684037100d7df00` |
| Automatic reload | 538021 | 0 | 31 | `d340110040f684024880112acc00201248100d7df000` |

The 35-film scan accepts **344 reload starts**: 2 here, 100 in Ranked Bandit,
and 242 in Ranked Oddball. It rejects **2,262 raw signature candidates** on the
following boundary/guard before life binding; those counts include arbitrary
payload matches and must not be described as 2,262 missed reloads. The earlier
exploratory life/roster-only search found 125 Bandit and 304 Oddball candidates;
the guarded export is deliberately smaller. There is no independent API reload
total or recorder validation of every Ranked event.

## Sparse weapon components

This section records the original probe's narrower grammar; the current native
decoder's larger quantities, multiplayer boundaries and paired inventory-field
widths are described above.

The registry identifies archetype 35 components **30** and **33** as
`weapon-state-ammo`, and **42** as `biped-desired-weapon-set`. This control binds
30 to slot 0 (Bandit) and 33 to slot 1 (pistol). Other registry slots, reserve
inventory components 31/34, pickups, and full weapon-set state remain undecoded.

Positive magazine values have a ten-bit form: **`0 + amount8 + 1`**, established
for amounts 1–15. **Zero is `11`, a two-bit form.** The probe rejects zero in the
ten-bit form and values above 15; general scalar encoding is still open. At the
empty-pistol shot, the variable-width zero is followed by the same checked
End/input boundary, independently supporting its two-bit length.

The original probe accepted only component 42's **`0010011`** (raw slot 1).
The current decoder additionally accepts **`0010001`** (slot 0), as established
above. The first three bits are constant across both forms, not the slot index.
Other slot or dual-wield forms remain unknown.

The parser validates the pawn header, generation, sorted sparse component list,
supported preceding field widths/guards, and a repeated tick against its local
clock when component 25 is present. It requires the exact 16-bit End/input
continuation. Unsupported components or continuations terminate this candidate;
there are no guessed skips. No ammo/selection deltas are accepted in either
Ranked film yet.

Selected component evidence (bits relative to packet payload):

| Observation | Chunk | Payload byte | Delta bit | Value bits | Value |
| --- | ---: | ---: | ---: | --- | --- |
| First Bandit shot, component 30 | 2 | 487680 | 198 | 225:235 | `0000011101` → 14 |
| Bandit refill, component 30 | 2 | 505510 | 37 | 64:74 | `0000011111` → 15 |
| Switch, component 42 | 2 | 516757 | 37 | 64:71 | `0010011` → slot 1 |
| Pistol empty, components [21,25,33] | 2 | 537889 | 204 | 278:280 | `11` → 0 |
| Pistol refill, component 33 | 3 | 482657 | 37 | 64:74 | `0000011001` → 12 |

Across all 35 films the strict export contains **23 magazine observations**:
17 in this control, one in each original single-shot control, and four late
magazine values in the Octagon AR burst. It also contains **four slot-1 selection
observations**. This is partial coverage, including for weapons whose larger
magazine values are not accepted.

## Weapon naming and replay semantics

Current native behavior: every accepted firing record supplies its raw weapon
window even when magazine fields are absent. Checked windows independently
identify their carried slot and 32-bit fingerprint. The replay names the seven
calibrated fingerprints above in either supported slot, without inventing
starting inventory or ammo. A unique same-packet firing/magazine association
remains a cross-check and fallback for a window with unsupported subfields.

An observed component-42 weapon-set update clears the held name even when its
payload cannot identify a slot. The full Octagon film has 570 firing observations
and 153 such invalidations, with no accepted magazine fields. Ammo stays `?`.
The saved variant independently enables bottomless clips, but API settings are
kept outside replay state; see [FILM_SETTINGS.md](FILM_SETTINGS.md).

The paragraphs below describe the earlier probe's narrower association rules;
its opaque-window and same-packet-only limitations are superseded above.

In this control the opaque firing windows `16acdc44d4` and `3f408190f4` identify
the Bandit and pistol, respectively. The earlier Bandit EVO starts/switch control
is consistent with the Bandit family. The internal 40-bit fields and universal
asset IDs remain unknown; the UI uses **Bandit** and **Pistol** without claiming
to distinguish every variant.

The probe binds an active weapon/slot only when one accepted magazine field and
one firing event share packet, player, and life. A selection observation clears
the held weapon name and magazine until a new observation identifies them.
Slots are not assumed to retain the same weapon through pickups. Unobserved
selection/pickup forms can still leave a held name outdated; all values are
observations, not a complete inventory simulation.

The replay shows a **LOAD** badge and **RELOAD START** control with previous/next
navigation and timeline marks. Its 500 ms pulse is purely a display duration.
The compact magazine readout uses `?` until observed, preserves recorded zero,
marks held amounts with a dashed underline, and clears at death or a new life.
Tooltips show sample time/age. UI slot numbers start at 1; evidence indices start
at 0. Ranked reload starts display even while magazines remain unknown. Existing
shooting, melee, grenade, health, and shield indicators remain independent.

The inspector annotates exact reload, magazine, selection, and boundary bits.
Unknown prefixes, auxiliary content, and unrelated packet data remain unparsed
or opaque. It rechecks source bytes before displaying exported evidence.

## Reproduction and regression checks

Run from the repository root; only downloading needs authentication:

```sh
HALO_TIMEOUT_SECS=120 HALO_PROBE_FILM=weapons/04-br-shock-rifle \
  cargo run --manifest-path experiments/Cargo.toml --example download_film_chunks
cargo run --offline --release --manifest-path experiments/Cargo.toml \
  --example decode_theater_film -- experiments/films/weapons/04-br-shock-rifle --compact
node experiments/examples/build_decoded_replay.cjs --corpus experiments/films
cargo test --offline --lib theater
cargo check --offline --lib --target wasm32-unknown-unknown
cargo clippy --offline --lib -- -D warnings
node experiments/examples/check_decoded_replay.cjs
python3 experiments/examples/theater_lab.py
```

Open <http://127.0.0.1:8766/replay?clip=weapons/04-br-shock-rifle>.
Outputs are each film's `decoded-film.json` and the standalone
`films/analysis/theater_viewer.html`. Re-decode the entire corpus before building
to apply new slot observations to older games; see [the lab workflow](README.md).

The 37 native Theater tests include captured packets for both switch directions,
the same BR fingerprint in either slot, invalid weapon wrappers, slots without
ammo, Stalker cooling/selection, reload/refill/zero forms, and unsupported neighbor rejection. Browser
checks cover both control switches (with unknown names/ammo), BR names in either
slot, held Oddball ammo/refills, bottomless Octagon, death/life clearing, backward
seeking and mobile layout. Historical probe exports remain under
`films/analysis/weapons/`; their standalone Python tools are archived.
