# Reachable native type declarations

Reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`inventory_transitive_types.go.txt` walks the AST from the 50 public facade type
aliases and 71 function signatures. It recursively resolves same-module named
types in fields (including private and embedded fields), callback signatures,
underlying aliases and exported receiver method signatures. Cycles retain edges
rather than duplicating declarations. Generic function constraints are inspected;
unsupported expressions and unresolved names fail generation.

The deterministic snapshot `public-transitive-type-declarations.json` contains
121 roots and 268 nodes, including the 71 synthetic function-signature nodes.
Seventy-one non-facade declarations were not included in the earlier alias and
signature inventories. These are additional audit targets, not missing Rust
implementations or completion percentages. Every edge resolves to a recorded
node or one of three explicit external leaves: context.Context, sync.Mutex and
time.Duration. All 84 defining/method source-file hashes were checked against
pinned git objects, including same-module domain files outside film.

This is a closed graph only for the stated syntactic traversal. In particular,
CompResult.Payload is `any`; its dynamic concrete values require inspection of
producers and consumers, not a claim that an AST walker has enumerated them.
Exported variable initializers, array-length constants, private method behavior,
generic instantiation behavior, callbacks, synchronization and external type
semantics need their existing separate audits. Names and declarations alone do
not establish field retention or behavioral parity.

The first concrete follow-up is native killsource.Provenance.String, which was
already marked unaudited in DATA_CONTRACT_INVENTORY.md. The Rust struct retains
Path, Origin and Multiplicity; Display now has a separate 48-case native
comparison (see ../fixtures/kill-provenance-v41.md). Other nodes include already represented reader context, observation
hooks, vitality payloads, component enums, source bits and profile fields. Reconcile
those against existing evidence before labeling any as missing or complete.

Regenerate with a copy of the retained helper ending in `.go`:

```sh
GOCACHE=/private/tmp/halo-go-cache /private/tmp/halo-go-runtime/go/bin/go run \
  /private/tmp/halo-inventory-transitive-types.go \
  /private/tmp/halo-levelup-v75/apps/go-api/internal/games/halo_infinite/film \
  src/theater/reference/public-transitive-type-declarations.json
```

The architecture in NEXT_PHASE.md remains deferred.

## Reconciled raw grammar configuration declarations

Three reachable declarations now have explicit storage mappings checked against
the retained AST and current Rust definitions:

- internal/profile.KeyframeProfile: EnTeteBits and MotDeTailleBits map to
  NativeKeyframeLayout.header_bits and size_word_bits, both i64. The raw layout
  and consumed-word fixtures cover signed values, lazy consumption and retained
  fields. See keyframe-layout-validation.json; the legacy bounded layout is a
  separate API. `CadreBits()` maps to `NativeKeyframeLayout::frame_bits()`: signed
  wrapping header + twice the size-word width. This is fixed framing overhead,
  not a record length. The 81-case keyframe-frame-bits-v41 oracle covers raw
  layouts and JSON round-trips; see keyframe-frame-bits-validation.json.
- internal/grammar.FrameConfig: HasExtraFields, IDLowBits, IDBase,
  NewDefaultStateBits and PacketPreambleBits map to the five NativeFrameConfig
  scalar fields. Native int widths remain i64, and IDBase remains u32. Profil
  and Obs are retained together in NativeReaderContext. Contextual consumers
  read raw profile fields lazily; no pointer-sized conversion is part of storage.
- internal/grammar.GrammaireBalayage: all fourteen fields are present in
  NativeScanGrammar. ControleDeCorruption, DeserEtatParArchetype,
  SimStateComplet, PorteeBaseline, GrammaireEcrivainI0, CorpsActionMobilite,
  CorpsAncrageCapacite, InferenceChaine, GenerationStricte, TablesParVue and
  ClassesDeVue map to their named Rust boolean policies. BitsDeQueueRecordNew
  maps to signed new_record_tail_bits. LargeursCalibrees and LargeursBouchon map
  to optional NativeSharedWidths handles, preserving nil versus allocated-empty
  maps and shared mutations across profile clones.

These are declaration/storage reconciliations. Existing reader/context and
signed-width oracles supply behavioral evidence for their represented domains;
this does not assert exhaustive callback behavior or complete parser parity.

## Reconciled vitality payload declarations

The pinned internal/grammar/vitality.go producers and current vitality.rs and
components/payloads.rs retain both reachable vitality structures:

- BodyVitality: Q -> quantum, Health -> health, and F5c/F5d/F5e -> flags in
  that order. Raw quantum and f32 health remain separate; negative health is
  not clamped by the stored representation.
- ShieldVitality: Q -> quantum, Shield -> shield, RegenPresent -> regen_present,
  Block64 -> block_64, and F66/F67/F69/F68 -> flags in the native wire order.
  HasRegen0/HasRegen1 and Regen0/Regen1 map to two independent Option<u16>
  entries. None reconstructs native false/zero because the producer initializes
  the structure to zero and only assigns a word behind its presence gate.
  Some(0) remains distinguishable from absence. The outer regen_present bit is
  retained even when both inner words are absent. Shield values above one and
  the raw overshield quantum remain available.

CapturedComponentPayload::Body/Shield project the same structures from retained
component fields. Ordered field ranges associate keyframe payloads with their
actual reads, including signed rewinds; the projection does not reread source
bytes. The bounded standalone vitality readers have atomic truncation semantics
and must not be mistaken for the native reader's padded-read contract.

Evidence: vitality::tests::every_vitality_quantum_matches_native_go compares all
published fields and endpoints against the independent native fixture, including
presence flags and native float values rounded to f32. It passed in the current
626-test host baseline (/private/tmp/halo-mask-wrap-suite.log). The overlapping
payload and captured-anchor validations separately exercise payload association
and serialization; see component-mask-wrap-validation.json and
overlapping-payload-validation.json. This reconciles these two declarations,
not every dynamic CompResult.Payload producer or every caller-constructed Go
struct with inconsistent presence flags.

Three additional dynamic payload declarations are reconciled against their
native producers and components/payloads.rs:

- RespawnTimer: Active -> active, T0/T1 -> timers[0]/timers[1]. Both ten-bit
  values remain raw integers without an invented time unit.
- RoundTimer: QA/QB -> quanta[0]/quanta[1], A/B -> seconds[0]/seconds[1],
  Tail -> tail. Raw and dequantized values remain separate; neither field is
  labeled elapsed or remaining without additional evidence.
- ObjectDissolver: Etat -> state, Corps -> body.is_some(), DureeQ ->
  duration_quantum, Drapeau -> flag. The state-13 branch produces no body,
  zero duration and false flag, matching the native initialized result. Rust
  additionally retains the three raw body words that native consumes but does
  not publish. Those extra words are not an interpretation of the body.

The existing native_captured_component_payloads test passed in the same host
baseline: 1,024 component cases plus exact f32 bit comparisons for all 65,536
round-timer quanta. The signed-rewind fixture adds four overlapping round-timer
cases. These storage/projection mappings do not claim that arbitrary absent
payloads represent zero state or that every dynamic payload type is covered.

## Object-parent payload and producer inventory

ObjectParentState maps to NativeObjectParentState in read_diagnostics.rs:
TypeIndex/Param -> archetype/parameter; StartBit/EndBit -> start_bit/end_bit;
Attached -> attached; Quant16/Word16 -> quantized_word/word; FlagA/FlagB ->
flags; Mtx -> matrix; Byte8/FlagC -> byte/flag_c; FreeRead/FreeBits ->
free_read/free_bits; TailSign/TailBit -> tail_sign/tail_bit. HasOpt16/Opt16,
HasVel/Vel, HasFreeID/FreeID, HasAlt11/Alt11, HasTail6/Tail6 and HasTail3/Tail3
map to optional_word, velocity, free_id, alternate, tail6 and tail3. These
Options preserve absent versus transmitted zero. Unread native branch fields
are initialized to zero, so the mapping concerns actual producer outputs.

Native EndBit is assigned only when the parent hook exists. Rust retains an
internal observation and its actual endpoint; this additional measurement must
not be described as native observer-absent EndBit parity. The signed-overlap
oracle verifies component-specific association rather than selecting whichever
observation happens to share a source start. Parent words remain positional raw
values, not inferred parent identities.

Inspection of pinned capture.go and production CompResult construction sites in
traverse.go establishes six dynamic Payload producers: body, shield, parent,
dissolver, respawn and round timer. Other dispatch names, calibrated replacements
and stub recovery publish nil. This closes the dynamic-type inventory for this
pinned producer, not all runtime contracts. The component-result contract audit
then found missing generic/keyframe variant metadata and the unretained failed
keyframe component; its correction and validation are tracked separately in
../fixtures/component-result-contract-v41.md.
