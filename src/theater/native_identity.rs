//! Source-preserving identity-section reads from the pinned film_identity.go map.
use super::bits::Bits;
use super::{FilmIdentity, FilmRegistry, bootstrap::find_identity_build, decode_film_identity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeIdentityValue {
    /// Bits in wire order. No endian conversion or signed interpretation.
    Scalar(u64),
    /// Fixed-width byte field, including bytes after a string terminator.
    Bytes(Vec<u8>),
    /// Known extent, unknown interpretation; retained bootstrap holds the bytes.
    Opaque,
    /// Entire requested extent was not present. No zero padding is supplied.
    Unavailable,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeIdentityField {
    pub name: String,
    /// Bootstrap-relative source coordinates, including the one-bit shift.
    pub bit: usize,
    pub bits: usize,
    pub value: NativeIdentityValue,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeIdentityRead {
    /// Complete legacy projection only; partial reads live in fields.
    pub identity: Option<FilmIdentity>,
    pub error: Option<String>,
    pub source_bits: usize,
    /// Selected by the reference's bounded HI_ search, not a proven boundary.
    pub build_anchor_byte: Option<usize>,
    pub fields: Vec<NativeIdentityField>,
}
/// Retain the writer-map fields at the reference-selected identity anchor.
/// Unknown blocks are opaque, not decoded structures. A truncated identification
/// preserves earlier fields and the first unavailable range, while retaining the
/// original projection error. No anchor means no fields are guessed.
pub fn read_native_identity(data: &[u8], registry: &FilmRegistry) -> NativeIdentityRead {
    let (identity, error) = match decode_film_identity(data, registry) {
        Ok(identity) => (identity, None),
        Err(error) => (None, Some(error.to_string())),
    };
    let mut out = NativeIdentityRead {
        identity,
        error,
        source_bits: data.len() * 8,
        build_anchor_byte: None,
        fields: Vec::new(),
    };
    if registry.major_version != 41 || registry.truncated {
        return out;
    }
    let Ok(Some(anchor)) = find_identity_build(data, registry.end_byte) else {
        return out;
    };
    out.build_anchor_byte = Some(anchor);
    let mut at = registry.end_byte * 8;
    let mut stopped = false;
    // The anchor is at a four-byte step from registry end plus 32 bytes.
    let mut add = |name: &str, width: usize, kind: u8| {
        if stopped {
            return;
        }
        let end = at.checked_add(width);
        let value = if end.is_none_or(|end| end > data.len() * 8) {
            stopped = true;
            NativeIdentityValue::Unavailable
        } else {
            match kind {
                0 => NativeIdentityValue::Scalar(Bits(data).read(at, width).unwrap()),
                1 => NativeIdentityValue::Bytes(
                    (0..width / 8)
                        .map(|i| Bits(data).read(at + i * 8, 8).unwrap() as u8)
                        .collect(),
                ),
                _ => NativeIdentityValue::Opaque,
            }
        };
        out.fields.push(NativeIdentityField {
            name: name.into(),
            bit: at,
            bits: width,
            value,
        });
        if !stopped {
            at = end.unwrap();
        }
    };
    for _ in (registry.end_byte..anchor - 32).step_by(4) {
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

#[cfg(test)]
#[path = "native_identity_tests.rs"]
mod tests;
