# Native action and control grammar

Additional reference: `d61443ef59268ad734355db8e9974f68db5ca6d0` (`bloc_action.go`, `frame_vue_controle.go`). The Rust shared action reader now reads R2 aim-vector mode (0: native absolute position; 1: R19 direction; 2/3: no payload) and target handles in categories 1/2. The raw grouped trigger/barrel fields stay in wire order; typed masks number the first wire bit as entry zero. Weapon indices distinguish absent (-2), recorded sentinel (-1), and recorded 0..3 values.

Control kind 0 now reads the optional five-bit third analog field, six-bit field, and five-bit flags. Complete native control entries retain their recorded index, optional fields, action values, and source boundaries. Truncated/unsupported payloads still retain their partial raw fields; they do not create a complete control entry. The index plus input gate uses the native six-bit grouped source guard. Native context and world precision flow through production, inference-view, and bounded replication-frame callers; standalone readers use the documented native default profile.

`halo_rust_action_control_d61443e_test.go.txt` independently generates 768 action blocks, actor-control component calls, and complete control views across all alignments and three precision/scope modes. It also generates 8,885 truncated control prefixes. Rust compares native action values, every raw action bit, component endpoints, control values, ordered kinds, native stop categories and all prefix endpoints. Complete views round-trip through JSON. No inferred firing or player timeline is generated.

`halo_rust_views_d61443e_test.go.txt` refreshes 2,048 view cases including their production/film prefix checks; 83 rows change. The three signed-view generators refresh 792/120/16 cases with no changed expected values. Original fixtures remain preserved.

Run generators in the additional reference grammar package via `go test ./internal/games/halo_infinite/film/internal/grammar -run 'TestHaloRust(ActionControlD61443e|ViewsD61443eParity|SignedView.*D61443e)' -count=1`. The action fixture is emitted compressed to `/private/tmp/halo-action-control-d61443e.json.zlib`; zlib-compress the views and signed-view JSON outputs separately. Do not overwrite the original reference pin.

Additional component/reference/keyframe fixtures were generated for migration; full corpus acceptance remains outstanding.

The broader migration retains the 225-name selection in `reference/component-selection-v41.json`: copy it to `/private/tmp/halo-components-selected.json` before running the components/unit-reference generators. The additional Go outputs contain 18,144 component cases, 18,144 reference cases, 256 unit-reference hook sequences, and two sets of 256 keyframes. Their Rust keyframe callers now explicitly supply the reference default position profile; corrected action vectors can no longer be read without position context. `reference/component-cases-d61443e.json` records the new component output hash and the limits of that selection.

The unit-reference fixture expands from 15,864 to 18,144 cases with the current selection, retaining all previous (name, archetype, level) contexts. Its random sequence changes after the first new selection entry; positional row diffs are not evidence of grammar changes. All expectations still come from Go.

Expanded hook validation exposed a missing region observation: `absAxisWFor` records the region on its first axis. Both action and projectile readers now use the original observed absolute reader, and both direct Go oracles explicitly export their histograms. The previous unobserved helper and its assumption were removed. Native control views also forward the caller's shared observer.
