//! Recorded context for the shared player record. No nearest-build fallback.
use crate::theater::parser::v41::registry::source::{BootstrapHeaderFieldKind, FilmRegistryRead};

#[derive(Debug, Clone, Default)]
pub(crate) struct PlayerContext {
    pub format_version: Option<u32>,
    pub build: Option<String>,
    pub personalization_bytes: Option<usize>,
}

impl PlayerContext {
    pub(crate) fn recorded(registry: &FilmRegistryRead) -> Self {
        let build = registry.bootstrap_header.as_ref().and_then(|h| {
            let f = h
                .fields
                .iter()
                .find(|f| f.kind == BootstrapHeaderFieldKind::BuildBranch)?;
            let raw = f.raw.as_ref()?;
            if f.width != 256 || raw.bit_len != 256 || raw.bytes.len() != 32 {
                return None;
            }
            let end = raw.bytes.iter().position(|b| *b == 0).unwrap_or(32);
            std::str::from_utf8(&raw.bytes[..end])
                .ok()
                .map(str::to_owned)
        });
        Self {
            format_version: registry.header.map(|h| h[1]),
            personalization_bytes: build.as_deref().and_then(personalization_bytes),
            build,
        }
    }
}

/// Pinned reference analysis 413971b profile/build_profile.go. HI_1_13_0 is from the
/// disassembled writer; other widths are upstream corpus measurements.
pub(crate) fn personalization_bytes(build: &str) -> Option<usize> {
    match build {
        "HI_1_12_0" | "HI_1_13_0" => Some(1852),
        "HI_1_10_0" | "HI_1_11_0" => Some(1492),
        "HI_1_8_0" | "HI_1_9_0" => Some(1312),
        "HI_1_4_1" => Some(2052),
        _ => None,
    }
}
