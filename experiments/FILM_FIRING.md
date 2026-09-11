# Firing activity: checked event prefixes and replay indicators

Run reproduction commands from `experiments/`.

The replay now shows firing activity from film event records, independently of
aim changes and kill summaries. This is a partial firing-event extraction, not
a complete bullet, damage, or ammunition decoder. The visual pulse lasts 150 ms
of film time so individual events remain visible during playback and scrubbing.

## Evidence from the controlled captures

The guarded prefix appears exactly once in each original single-shot recording:

| Capture | Event film time | Following decoded recoil |
| --- | ---: | ---: |
| `force-end/03-shoot` | 21.184038 s | First aim sample at 21.200559 s |
| `natural-end/03-shoot` | 26.305376 s | First aim sample at 26.338777 s |

It appears 30 times during Nuzzles' stationary AR burst in
`octagon/02-ar-kill`, starting at 22.834719 s and recurring about every 83 ms.
All 30 records bind to Nuzzles, and none bind to Yet. The new reload control
`4050c25f-3da5-4b16-9a50-2fab9660fed4` adds three Bandit and twelve pistol events,
each matched to a magazine decrease (see [weapons](FILM_WEAPONS.md)). The current
replay total is 4,986 firing pulses. The other **30 controlled
films** have no accepted firing events, including idle, walking, jumping,
controller circles, aiming, melee, grenade, ping, and weapon/grenade-switch
experiments. This separates the signal from motion, view changes, and generic
button activity.

The Bandit Ranked capture adds **1,584 guarded records** across all eight players.
Each identifies both a spawn serial and its roster player, independently matching
the previously decoded spawn bindings. Five records arrive 12–49 ms after their
player's summary death timestamp. Their exact role is unknown; they remain in
the evidence but are excluded from the replay's **1,579 firing pulses**.

Oddball adds **3,374 guarded records**: seven identical same-frame copies and
seven events after death are retained as evidence but excluded from the replay.
The remaining **3,360 events** cover all eight players and all three rounds.
Full wire IDs and generation tags prevent aliasing at IDs 128 and 256. The
sequence wraps 255→0 for every Oddball player; only modulo-forward steps 1–127
are accepted after identical copies are removed. These are activity counts,
not an exact API bullet-total claim.

## Observed extraction windows

All offsets are zero-based, MSB-first, relative to the start of the matched
prefix. The leading bit varies between standalone and embedded occurrences.
These are guarded event windows, not a claim that the older generic 14-bit entity
header model parses this record.

| Relative bit | Width | Observation |
| ---: | ---: | --- |
| 0 | 1 | Observed 0 or 1; semantic role unknown |
| 1 | 11 | `10100100110` |
| 12 | 8 | Wire identity, observed 0–255 |
| 20 | 2 | Generation: `01` / `10` |
| 22 | 4 | `0000` |
| 26 | 7 | Low seven bits of an observed increasing sequence |
| 33 | 1 | Observed sequence high bit, weight 128 |
| 34 | 1 | Observed 0; other forms rejected |
| 35 | 5 | Persistent roster index |
| 40 | 3 | Carried slot, currently checked for 0/1 |
| 43 | 1 | Checked weapon wrapper bit `1` |
| 44 | 32 | Weapon fingerprint |
| 76 | 4 | Checked weapon wrapper `0100` |
| 80 | 28 | `0010110010010110011110011111` |

The sequence reading is supported through 127→128 and 255→0 transitions.
The scene life serial is `wire + 256 × (generation − 1)` for the two observed
generations. Further generation tags remain unchecked. Sequence gaps are
reported, not synthesized into shots. The AR capture has five gaps; Bandit has
two, for Miiindful (18→20) and Nuzzles (123→125). Other action types or unsupported
record forms may consume sequence values. The field is not assumed to be an
ammunition count or a complete bullet counter. Oddball has nine forward gaps;
its ordinary 255→0 wraps are recorded separately from gaps.

The BR75/Shock control separates the slot and 32-bit fingerprint inside the
40-bit weapon window; the complete window is still exported. Across 17,311
independent same-packet magazine associations, the slot agrees every time.
`12b1824d54` and `32b1824d54` both identify BR75, in slots 0 and 1 respectively.
Seven fingerprints now have calibrated names; Shock Rifle is calibrated only from
reported spawn loadout, with no firing confirmation in that control. See
[weapon identities and limits](FILM_WEAPONS.md#weapon-fingerprints-and-both-selection-directions).
These labels do not establish universal weapon asset IDs. Unsupported wrapper
forms retain their raw window but do not supply a slot or fingerprint. The parser uses the
constant following guard, spawn/roster agreement, life boundaries, and sequence
consistency instead of selecting events by recoil or proximity to a kill.

## Independent comparison with match statistics

The match-statistics API was queried separately for
`ec02ed9d-346e-4eb0-a491-77e6b557847c`. These are the actual results, not an exact
validation claim:

| Player | Guarded events | Replay events before death | API shots fired |
| --- | ---: | ---: | ---: |
| diarrhea698343 | 185 | 184 | 184 |
| NON STOP BRS | 188 | 188 | 188 |
| Yet | 218 | 217 | 215 |
| Live4rmda504 | 176 | 175 | 175 |
| Miiindful | 212 | 211 | 213 |
| Nuzzles | 224 | 224 | 224 |
| Relic Future | 183 | 182 | 180 |
| ELITExBOSH | 198 | 198 | 195 |
| Total | 1,584 | 1,579 | 1,574 |

Four players match exactly after excluding late events; others differ by two or
three. Timing around death, unsupported forms, and weapon-specific shot accounting
remain unresolved. This comparison supports showing **firing activity** alongside
the controlled evidence. It does not establish one event per API-counted bullet,
shot accuracy, hits, damage, or a continuous trigger-held state. The earlier
kill/death/medal aggregate validation still matches exactly and is separate.

[The vitality follow-up](FILM_VITALITY.md) now identifies separate body/shield
components and a regeneration-delay countdown. Those updates describe the target's
state; they do not resolve the firing/API count differences or establish a damage
value for every firing event.

## Reproduce

First build the Octagon and Ranked scenes using the
[replay guide](examples/theater_viewer/README.md), so their death boundaries are
available. To refresh the optional API shot comparison, the roster probe now
exports `validation.firing_stats` as well as its existing event validation:

```sh
HALO_TIMEOUT_SECS=120 HALO_VALIDATE_STATS=1 HALO_PROBE_FILM=bandit/01-evo \
  HALO_ROSTER_OUTPUT=films/analysis/bandit cargo run --example film_roster_probe

python3 examples/film_firing_probe.py
python3 examples/build_theater_viewer.py
python3 -m unittest discover -s examples -p 'test_*firing.py'
open films/analysis/theater_viewer.html
```

The firing probe reads the checked profiles in [films.csv](films.csv) and
writes `films/analysis/firing/events.json` and `evidence.json`. The latter keeps
chunk, payload byte, bit offset, identity, sequence, opaque window, late-event
flags, rejected candidates, and API comparison counts. Missing cached films are
skipped; the current audit includes all 36 checked films, including Oddball.
The evidence also exports independently checked lives for the melee probe.
Re-running is offline unless the
roster API comparison is explicitly requested.

The HTML builder automatically merges the firing export into matching clips;
`--firing-json PATH` selects another export. Match IDs must agree. Player rows are
`[film_seconds, spawn_serial, sequence]`. Firing extraction does not change the
position or aim CSVs/scenes. The complete current replay includes **4,986 pulses**:
3,360 Oddball, 1,579 Bandit, 30 Octagon, the two single-shot controls, and
15 in the manual/automatic reload control.

## Display and validation

An orange **SHOOT** badge, player label, and ground ring pulse for 150 ms after
an event. A schematic muzzle flash is added when the avatar has a recent position
and aim, or the controlled scene supplies reported stationary facing. Unknown or
stale pose does not create a guessed firing direction; the badge still identifies
the event. Every player has persistent roster badges and compact health/shield meters,
including when off camera. [Melee](FILM_MELEE.md) has a separate purple badge, so
overlaps display both actions. The pulse is a display duration, not a measured animation or trigger
hold. It is hidden at death and cannot cross into a new life.

The arrow buttons in the firing readout jump to the selected player's previous or
next event. Orange timeline ticks show that player's events. Seeking in either
direction recomputes the same state, and sample stepping includes firing times.
The old “Recorded position” readout is now **“Recorded sample.”**

Captured fixtures cover both prefix-leading bits, the sequence high bit, distinct
spawn/player identities after respawn, truncated/corrupted guards, and incorrect
life bindings. Browser validation checks every displayed Ranked event at its exact
timestamp, firing navigation and reverse seeks, clearing at deaths/spawns, and
the controlled shooting/non-shooting clips. The
committed Chrome driver is `examples/check_theater_replay.cjs`.
