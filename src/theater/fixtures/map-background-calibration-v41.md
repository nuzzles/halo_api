# Native world-to-pixel calibration oracle

Reference commit: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: ../reference/halo_rust_map_background_calibration_test.go.txt,
registered in generate_oracles.py as TestHaloRustMapBackgroundCalibration.

3,465 cases call native MapBackgroundCalibration.MondeVersPixel and json.Marshal.
Input coefficient/point floats are retained as IEEE f64 bits. Returned coordinates
remain signed 64-bit integers, with a separate in-frame boolean. Native JSON or
its refusal is compared as well. Boundary cases explicitly preserve truncation
toward zero rather than substituting floor or half-pixel adjustments.

This is calibration evidence, not a complete background-file loader. See
../reference/MAP_BACKGROUND_SCOPE.md for mappings and remaining work.
