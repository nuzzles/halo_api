//! Version-41 bootstrap identification;
//! see `docs/CREDIT.md` and the pinned port manifest.

use super::{DecodeError, FilmRegistryRead, RegistryStop, bits::Bits};

fn field(data: &[u8], offset: usize) -> Option<String> {
    let raw = data.get(offset..offset.checked_add(32)?)?;
    let end = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
    raw[..end]
        .iter()
        .all(|b| (32..=126).contains(b))
        .then(|| String::from_utf8_lossy(&raw[..end]).into_owned())
}

pub(super) fn find_identity_build(
    data: &[u8],
    registry_end_byte: usize,
) -> Result<Option<usize>, DecodeError> {
    let start = registry_end_byte
        .checked_add(32)
        .ok_or_else(|| DecodeError::Inconsistent("bootstrap offset overflow".into()))?;
    let limit = start
        .saturating_add(0x1000)
        .min(data.len().saturating_sub(32));
    Ok((start..limit)
        .step_by(4)
        .find(|&off| data.get(off..off + 3) == Some(b"HI_") && field(data, off).is_some()))
}

/// Read the section immediately after a structurally parsed registry.
/// Returns `None` when no build anchor is present; never supplies guessed metadata.
pub(crate) fn decode_film_identity(
    data: &[u8],
    registry: &FilmRegistryRead,
) -> Result<Option<FilmIdentity>, DecodeError> {
    let [major_version, format_version] = registry
        .header
        .ok_or_else(|| DecodeError::Inconsistent("truncated bootstrap registry header".into()))?;
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version as i32));
    }
    if registry.stop != RegistryStop::BoundaryBlock {
        return Err(DecodeError::Inconsistent(
            "truncated bootstrap registry".into(),
        ));
    }
    let Some(build_offset) = find_identity_build(data, registry.registry_end_byte)? else {
        return Ok(None);
    };
    // Two 256-byte names, four u32s, three 4096-byte blocks and two 16-byte blocks,
    // all following one corruption-check flag bit. Their unknown meanings stay raw.
    const AFTER_FLAG_BYTES: usize = 2 * 256 + 4 * 4 + 3 * 4096 + 2 * 16;
    let flag_byte = build_offset + 0x48;
    let body_bit = (flag_byte + AFTER_FLAG_BYTES) * 8 + 1;
    if body_bit > data.len().saturating_mul(8) {
        return Err(DecodeError::Inconsistent(
            "truncated bootstrap identification".into(),
        ));
    }
    let word = |offset| -> u32 {
        u32::from_le_bytes(
            data[offset..offset + 4]
                .try_into()
                .expect("checked identification bounds"),
        )
    };
    let bits = Bits(data);
    let timestamp_bit = flag_byte * 8 + 1 + 2 * 256 * 8;
    Ok(Some(FilmIdentity {
        format_version,
        registry_blocks: registry.registry.archetypes.len(),
        registry_fingerprint: registry.registry.fingerprint().unwrap_or(0),
        registry_named_slots: registry
            .registry
            .archetypes
            .iter()
            .map(|a| a.components.len())
            .sum(),
        version: field(data, build_offset - 32).unwrap_or_default(),
        build: field(data, build_offset).expect("validated build anchor"),
        flavor: field(data, build_offset + 32).unwrap_or_default(),
        build_id: word(build_offset + 0x40),
        changelist: word(build_offset + 0x44),
        type_versions: (registry.registry_end_byte..build_offset - 32)
            .step_by(4)
            .map(word)
            .collect(),
        corruption_checks: bits.read(flag_byte * 8, 1) == Some(1),
        match_start_unix: (bits
            .read(timestamp_bit, 32)
            .expect("checked timestamp bounds") as u32)
            .swap_bytes(),
        build_offset,
        body_bit,
    }))
}

use crate::theater::runtime::identity::FilmIdentity;
