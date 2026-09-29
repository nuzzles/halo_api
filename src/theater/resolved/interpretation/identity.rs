//! Source-preserving identity-section reads from the pinned film_identity.go map.
use super::bits::Bits;
use super::{
    FilmRegistryRead, RegistryStop,
    bootstrap::{decode_film_identity, find_identity_build},
};

/// Retain the writer-map fields at the reference-selected identity anchor.
/// Unknown blocks are opaque, not decoded structures. A truncated identification
/// preserves earlier fields and the first unavailable range, while retaining the
/// original projection error. No anchor means no fields are guessed.
pub(crate) fn read_identity(data: &[u8], registry: &FilmRegistryRead) -> IdentityRead {
    let (identity, error) = match decode_film_identity(data, registry) {
        Ok(identity) => (identity, None),
        Err(error) => (None, Some(error.to_string())),
    };
    let mut out = IdentityRead {
        identity,
        error,
        source_bits: data.len() * 8,
        build_anchor_byte: None,
        fields: Vec::new(),
    };
    if registry.header.map(|header| header[0]) != Some(41)
        || registry.stop != RegistryStop::BoundaryBlock
    {
        return out;
    }
    let Ok(Some(anchor)) = find_identity_build(data, registry.registry_end_byte) else {
        return out;
    };
    out.build_anchor_byte = Some(anchor);
    let mut at = registry.registry_end_byte * 8;
    let mut stopped = false;
    // The anchor is at a four-byte step from registry end plus 32 bytes.
    let mut add = |name: &str, width: usize, kind: u8| {
        if stopped {
            return;
        }
        let end = at.checked_add(width);
        let value = if end.is_none_or(|end| end > data.len() * 8) {
            stopped = true;
            IdentityValue::Unavailable
        } else {
            match kind {
                0 => IdentityValue::Scalar(Bits(data).read(at, width).unwrap()),
                1 => IdentityValue::Bytes(
                    (0..width / 8)
                        .map(|i| Bits(data).read(at + i * 8, 8).unwrap() as u8)
                        .collect(),
                ),
                _ => IdentityValue::Opaque,
            }
        };
        out.fields.push(IdentityField {
            name: name.into(),
            bit: at,
            bits: width,
            value,
        });
        if !stopped {
            at = end.unwrap();
        }
    };
    for _ in (registry.registry_end_byte..anchor - 32).step_by(4) {
        add("type_version", 32, 0);
    }
    for name in ["version", "build", "flavor"] {
        add(name, 256, 1);
    }
    add("build_id", 32, 0);
    add("changelist", 32, 0);
    add("corruption_checks", 1, 0);
    for _ in 0..2 {
        add("name", 2048, 1);
    }
    add("match_start_unix", 32, 0);
    for _ in 0..3 {
        add("unknown_word", 32, 0);
    }
    for _ in 0..3 {
        add("unknown_4096_bytes", 32768, 2);
    }
    for _ in 0..2 {
        add("unknown_16_bytes", 128, 2);
    }
    out
}

use crate::theater::resolved::identity::{IdentityField, IdentityRead, IdentityValue};
