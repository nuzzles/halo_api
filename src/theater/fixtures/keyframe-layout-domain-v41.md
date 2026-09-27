# Native keyframe layout domain

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
90 cases cross chain/table APIs, header skips -108, -1, 0, 32, 64, 108, 160,
2^32 and i64::MAX, and size-word widths -1, 0, 1, 32 and 64. Each entry has the
no-archetype sentinel, so the size-word width is never consumed. Native preserves
the header skip and resulting cursor/stop/counts; ten table cases panic during
negative sentinel fallback. This fixture does not establish consumed wide-word
behavior. Large skips use tiny inputs, not multi-gigabit recordings.

Native source: reference/halo_rust_keyframe_layout_domain_test.go.txt, registered
in reference/generate_oracles.py. Native execution passes. Rust uses signed NativeKeyframeLayout values and compares
cursors, stops and counts. Each row also checks a generic End frame: unused
keyframe settings remain accepted, cursor 5 and no non-End records. Full
validation is tracked in reference/keyframe-layout-validation.json.
