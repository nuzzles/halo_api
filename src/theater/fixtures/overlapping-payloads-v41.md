# Ordered payloads with overlapping source ranges

Pinned reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: reference/halo_rust_overlapping_payloads_test.go.txt.

32 parent cases cross first/second precision parameters 0..3 through generic
frames and full keyframes. A calibrated negative skip returns the second parent
to the first parent's start; the two typed values remain distinct and ordered.
Native observer values retain parameter, endpoints and optional-field gates.
Four full-keyframe round-timer cases use patterned source data and calibrated
skips -35, -20, -10 and 0, checking quanta, seconds and tail values. They include
partial overlap where source-range filtering mixes fields from separate reads.

These are synthetic native grammar controls, not captured gameplay or independent
action annotations. Status: reference/overlapping-payload-validation.json.
