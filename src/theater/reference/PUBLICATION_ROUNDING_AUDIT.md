# Publication rounding contract reconciliation

Reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.

replay/build_arrondis.go declares round2 and fractionForJSON. They map to
replay_tracks.rs::rounded with scales 100 and 1000, respectively: widen the
input f32 to f64, multiply, round ties away from zero, divide, then narrow to
f32. ReplayPoint x/y/z use the hundredth scale; optional hp/sh use the thousandth
scale after the native display-fraction clamp. The unrounded source values
remain in ReplayPositionSample and the underlying parser outputs.

Native fractionForJSON returns a pointer so a measured zero is not omitted.
Rust hp/sh are Option<f32> serialized only when present: Some(0) and None remain
distinct. Coordinate publication and display fractions do not replace native
position/vitality source data or establish byte-for-byte re-encoding.

The existing 1,024-case independent native_track_publication fixture compares
complete tracks and coverage, including samples, lifetimes, gaps and minimum
point filtering. It passed in the current 629-test host suite. The manifest's
pending entry was stale and is now partial-v41 with this mapping and evidence.
This is not an exhaustive standalone float-bit test or proof of every caller's
publication behavior. In particular, native/Rust JSON handling of nonfinite
caller-supplied floats is outside this finite captured-publication evidence.
