# Crouch input in controls and Octagon gameplay

Match `ff4eb2e4-a2ba-4b95-a892-d8a413ac3a32` strongly matches the recorder's
sequence: crouch for about one second at 0:55 remaining, crouch for five seconds
at 0:50, then sprint forward and slide at 0:40. The v41 film ended normally and
lasts 90.784 seconds. It contains Nuzzles, one life, 315 position samples and
76 aim samples under the existing X15Y15Z17 layout.

The **crouch input is decoded**. Physical crouch posture and sliding are not yet
established independently. The decoder does not infer a slide from speed plus
crouch, or a stance from the input alone.

## Evidence

An extra tail appears after the existing two movement axes. The same form is
present in the older one-action crouch control, absent in its idle/walking
controls, and present during both new holds and the reported slide sequence.
The independent jump control has a distinct tail. The terminal-input follow-up
below recognizes it separately from crouch and also recognizes their combined form.

| Reported action | Observed film time | Interpretation |
| --- | --- | --- |
| Short crouch at 0:55 | 22.119–22.903 s | Repeated held input, approximately 0.8 s |
| Five-second crouch at 0:50 | 26.907–31.912 s | Repeated held input, approximately 5 s |
| Sprint/slide near 0:40 | Accepted held-input samples 37.535–38.269 s | Crouch input during forward movement; not an independently decoded slide |

All times are film timestamps, including pregame. First and last supported
samples bound observations, not exact animation durations. Short gaps include
record forms outside the existing input-chain grammar. The older crouch control
records held samples at 26.439–28.391 s; its reported one-second duration was
approximate and is not used to manufacture an end time.

## Initial input-chain decoding

The parser first validates the existing clock/entity/End/input-axis chain, bound
to wire 0, generation 1, roster player 0. At the end of its 25-bit input window:

- `00000`: explicit empty command tail, interpreted as released crouch input.
- `00100001000`: held crouch input in the checked control form.

In the original reader, only zero padding to the next byte may follow. This exact length check rejects
other buttons, combined inputs, additional players, and arbitrary extra bytes.
Those forms identify an observation; they do not establish the internal encoding
of every button bit or a general command schema. No missing record means false.

Example payloads from the new control:

```text
released: a07b420300806bef80
held:     a07b4200d0806bef9080
```

Both have clock bits [0,37), End [37,40), input prefix/axes [40,65).
The released tail is [65,70), followed by two zero bits. The held tail is
[65,76), followed by four zero bits. The opaque command prefix is not relabeled
as a physical-pose component.

## Terminal commands and multiplayer crouch

The full Octagon film `9a875c4e-03bc-4fff-a688-e21216d1618b` has a second
input header tag, even while Nuzzles is idle, and multiple command records at
the end of a frame. The original fixed first-player header rejected these.
The new `src/theater/input.rs` reader recognizes these complete terminal records:

```text
1 | roster slot (8 bits) | opaque tag (4 bits) | forward (6) | left (6) | command tail
```

Checked slots are 0 and 1; checked tags are 13 and 14. The tag's meaning remains
unknown. Both axes are bounded to 0–62, matching the independently decoded
analog input fields. Four exact tail forms are supported:

| Tail | Crouch input | Other corroborated input |
| --- | --- | --- |
| `00000` | released | empty checked form |
| `00100001000` | held | crouch form from both controls |
| `00100100000` | released | jump form from the separate jump control |
| `00100101000` | held | combined jump/crouch form in Octagon |

This is a list of validated encodings, not a decoder for arbitrary button masks.
The jump-only form occurs at Nuzzles's film time **197.410521 s** as his recorded
Z rises from 115227 to 115233, then 115239 and 115245. The combined form appears
for timesknightt at **154.133810 s**, also at the start of a recorded height rise.
These comparisons corroborate the command interpretation; the replay does not
derive crouch, jump, or slide state from the position trace.

The terminal reader checks ascending unique slots, the complete one/two-record
sequence, the preceding zero guard, and exact final-byte zero padding. Nested
suffixes keep the longer record sequence; incompatible overlapping candidates
are rejected. Unsupported headers, button combinations, axes, extra bytes, and
truncated tails are rejected. A preceding strict film clock is required.
Opaque replication components before the commands are left untouched: no
component widths are inferred to reach the suffix.

Each command's slot binds to that roster player's unique active life at the
recorded timestamp. It does not bind to the preceding pawn update or assume
wire zero after respawn. Samples before spawn or at/after death are excluded.
This extension is currently enabled for one/two-player recordings; larger
rosters remain unsupported.

### Octagon replay results

| Player | Crouch samples | Held samples | Observed held runs |
| --- | ---: | ---: | ---: |
| Nuzzles | 17,773 | 520 | 73 |
| timesknightt | 10,478 | 3,253 | 207 |

Runs group successive observations with the same held value and life. Gaps can
contain missing transitions, so these are not exact counts of physical crouches.

Select **Octagon gameplay → Octagon · First to 50** and seek to **2:34.84–2:35.36**
(film time). Nuzzles's **CROUCH INPUT** badge alternates held/released three times
while his recorded position follows the jump. The corresponding sample checks
are 154.84 held, 154.90 released, 154.98 held, 155.15 released, 155.22 held,
and 155.36 released, all bound to life 2. The other player's state remains
independent, including when eliminated.

Both idle Octagon players now have released input samples. Idle, walking,
aiming, shooting, grenade, reload, melee, and appearance controls gain no held
crouch samples. The independent jump control remains crouch=false. All previous
observations across the 29-film corpus are preserved; only input/crouch streams
are extended. Other player streams, summary events, clocks and projectiles are
unchanged. The corpus now has **154,898 crouch samples, 4,282 held**.

The original crouch/slide control now has **4,756 samples, 387 held**. Its three
holds retain their timings, with the third beginning at 37.534851 s because the
terminal command can be validated independently of the opaque ability prefix.
The old crouch control has 116 held samples over the same 26.439–28.391 s window.

Evidence: `films/analysis/posture/octagon-crouch-input.csv`, `octagon-evidence.json`,
and captured records in `src/theater/fixtures/terminal_input_records.json` at the
repository root. The existing `crouch-input.csv` and `evidence.json` are refreshed
for the control; the JSON retains the initial input-chain-only counts.

## Physical stance and sliding remain unverified

The extended scan checks sparse masks, the proposed dense 64-bit mask and
baseline-selector variants, binding candidate headers to living pawn identities.
It finds **no component-29 or component-62 candidate in the crouch/slide control**.
In Octagon, all 49 apparent hits are dense-mask interpretations at unanchored
interior offsets: none starts at the checked clock boundary and none is a sparse
record. Without a complete preceding parse, these are not accepted evidence.
They must not produce physical stance or slide samples.

Two tick-checked prefixes still contain components 57 and 59 at 36.734105 and
37.534851 seconds. Their seven bits change from `1010000` to `0000000`. The
external Infinite reference proposes a 2-bit component 57 plus a 5-bit component
59 for these forms, consistent with the following copied-position boundary.
This does not identify sprint or slide semantics. These bytes stay opaque in
the production decoder. Component 28 is active-camo state, not posture.

The reference proposes component 29 as a flag plus 10-bit crouch amount, and
component 62 as an active flag followed by a packed vector and three bytes.
No aligned active capture has validated these schemas in our films. Local
prediction may reconstruct posture from commands; that possibility is not proof
that physical state is absent elsewhere. A second player observing an isolated
crouch/slide control would be the most useful new capture.

## API and replay

`PlayerTrack.crouch_input` contains `Sample<bool>` values with life, timestamp,
and original bit range. Older Film JSON defaults the field to an empty vector.
Existing input, motion, aim, combat, appearance and other streams are preserved.

Select **Posture controls → Crouch & slide**. The replay shows a **CROUCH INPUT**
badge above players and in the roster. It uses recent recorded samples, clears
to unknown at death/new life or a gap over 100 ms, and recomputes on backward
seeks. It does not lower the camera, animate a guessed stance, or label sliding.
The same badge now works in **Octagon gameplay → Octagon · First to 50**.
Ranked input mapping remains unsupported, so Ranked crouch input remains unknown.

From `experiments/`:

```sh
HALO_PROBE_FILM=posture/01-crouch-slide cargo run --example download_film_chunks
cargo run --release --example decode_theater_film -- films/posture/01-crouch-slide --compact
node examples/build_decoded_replay.cjs --corpus films
```

Captured tests in `../src/theater/fixtures/crouch_input_records.json` include
holds, releases, movement at different alignments, the earlier crouch control,
and unsupported jump/multiplayer tails. Tests cover truncation, nonzero padding,
extra bytes, life binding, and JSON round trips. `films/analysis/posture/` retains
accepted input samples and the unpromoted ability-state candidates.
The terminal fixtures add both header tags, two-player commands, jump-only and
combined forms, respawn binding, missing clocks, and independent player states.
The browser check exercises the Octagon toggles, death, and reverse seeking.
