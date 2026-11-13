//! Recorded context for the shared player record. No nearest-build fallback.
use crate::theater::parser::v41::chunks::registry::source::{
    BootstrapHeaderFieldKind, FilmRegistryRead,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::parser::v41::{
        chunks::registry::parse_registry_chunk,
        context::events::tests::{copy_bits, registry},
    };

    #[test]
    fn recorded_player_profile_uses_complete_build_field_without_needing_options() {
        let mut bytes = registry(3, 1);
        let read = parse_registry_chunk(&bytes).unwrap();
        let f = read
            .bootstrap_header
            .as_ref()
            .unwrap()
            .fields
            .iter()
            .find(|f| f.kind == BootstrapHeaderFieldKind::BuildBranch)
            .unwrap();
        let (bit, width) = (f.bit, f.width);
        let mut name = [0u8; 32];
        name[..9].copy_from_slice(b"HI_1_13_0");
        copy_bits(&mut bytes, bit, &name, width);
        let mut read = parse_registry_chunk(&bytes).unwrap();
        read.bootstrap_header.as_mut().unwrap().identification = None;
        let profile = PlayerContext::recorded(&read);
        assert_eq!(profile.format_version, Some(27));
        assert_eq!(profile.personalization_bytes, Some(1852));
        let f = read
            .bootstrap_header
            .as_mut()
            .unwrap()
            .fields
            .iter_mut()
            .find(|f| f.kind == BootstrapHeaderFieldKind::BuildBranch)
            .unwrap();
        f.raw = None;
        assert!(
            PlayerContext::recorded(&read)
                .personalization_bytes
                .is_none()
        );
        let partial = parse_registry_chunk(&bytes[..(bit + width) / 8 - 1]).unwrap();
        assert!(
            PlayerContext::recorded(&partial)
                .personalization_bytes
                .is_none()
        );
    }
}
