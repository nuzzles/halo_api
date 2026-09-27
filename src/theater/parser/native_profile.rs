//! Native resolved profile metadata and loaded-source resolution for v41.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub(crate) enum NativeProfileIssue {
    #[error("filmdec: version de format de chunk_00 absente de la table de profil : {0}")]
    UnknownFormat(u32),
    #[error("filmdec: build absent de la table de profil : {0:?}")]
    UnknownBuild(String),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativePrecisionDescriptor {
    pub index_bits: u64,
    pub axis_bits: [u64; 3],
    pub region: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeMovementProfile {
    pub traversal: NativePrecisionDescriptor,
    pub world_object: NativePrecisionDescriptor,
    pub delta_quantum: f32,
    pub delta_axis_width: u64,
    pub range: FilmQuantizationRange,
    pub full_precision: bool,
    pub delta_has_handle_tail: bool,
    pub calibrated_skip: bool,
    pub mobility_action_extra_bits: i64,
}
impl Default for NativeMovementProfile {
    fn default() -> Self {
        Self {
            traversal: NativePrecisionDescriptor {
                index_bits: 1,
                axis_bits: [6; 3],
                region: 0,
            },
            world_object: NativePrecisionDescriptor {
                index_bits: 1,
                axis_bits: [13, 13, 14],
                region: 0,
            },
            delta_quantum: NATIVE_DELTA_QUANTUM,
            delta_axis_width: 14,
            range: NATIVE_QUANT_RANGE_CE_BIPED,
            full_precision: false,
            delta_has_handle_tail: false,
            calibrated_skip: false,
            mobility_action_extra_bits: 0,
        }
    }
}
/// Profile values are metadata, not substituted wire fields. Optional keys retain
/// the distinction between an unread header and a recorded zero. Absent identity
/// is distinct from a recorded false corruption-control flag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct NativeResolvedFilmProfile {
    pub registry_present: bool,
    pub major_version: Option<u32>,
    pub format_version: Option<u32>,
    pub identity: Option<FilmIdentity>,
    pub map: Option<FilmMapBounds>,
    pub highlight: FilmGamertagLayout,
    pub keyframe: KeyframeLayout,
    pub movement: NativeMovementProfile,
    /// None is the native zero/unknown slot profile, not a zero-byte declaration.
    pub personalization_bytes: Option<usize>,
    pub personalization_delta_bits: Option<i32>,
    /// None is the native zero-width MPP profile, distinct from scan defaults.
    pub mpp: Option<FilmMppWidths>,
    /// Native ordered format/build errors. Missing maps are not profile errors.
    pub issues: Vec<NativeProfileIssue>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NativeProfileResolveError {
    #[error("unsupported Theater profile major version {0}; supported: 41")]
    UnsupportedVersion(u32),
}

/// Native ResolveProfile source behavior, restricted to v41 identification.
/// Unavailable identities preserve readable format keys. Profile resolution reads
/// original source bytes independently of mutations to a context's cached registry.
pub(crate) fn resolve_native_film_profile(
    source: Option<&FilmSource>,
    map: Option<&FilmMapBounds>,
) -> Result<NativeResolvedFilmProfile, NativeProfileResolveError> {
    let raw = source.and_then(FilmSource::registry_chunk);
    let word = |offset| {
        raw.and_then(|b| b.get(offset..offset + 4))
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
    };
    let major = word(0);
    if let Some(major) = major.filter(|&v| v != 41) {
        return Err(NativeProfileResolveError::UnsupportedVersion(major));
    }
    let format = word(4);
    let identity = raw.and_then(|data| {
        let parsed = parse_registry_chunk(data).ok()?;
        decode_film_identity(data, &parsed.registry).ok().flatten()
    });
    let mut issues = Vec::new();
    let resolved = resolve_film_mpp(format.unwrap_or(0));
    if raw.is_none() || resolved.unknown_format {
        issues.push(NativeProfileIssue::UnknownFormat(format.unwrap_or(0)));
    }
    let build = identity.as_ref().map_or("", |id| id.build.as_str());
    let personalization_bytes = matches!(build, "HI_1_13_0" | "HI_1_12_0").then_some(1852);
    if personalization_bytes.is_none() {
        issues.push(NativeProfileIssue::UnknownBuild(build.into()));
    }
    Ok(NativeResolvedFilmProfile {
        registry_present: raw.is_some(),
        major_version: major,
        format_version: format,
        identity,
        map: map.cloned(),
        highlight: native_gamertag_layout(i64::from(major.unwrap_or(0))),
        keyframe: KeyframeLayout::default(),
        movement: NativeMovementProfile::default(),
        personalization_bytes,
        personalization_delta_bits: personalization_bytes.map(|n| (n as i32 - 1852) * 8),
        mpp: resolved.widths.map(|w| FilmMppWidths {
            lead: w[0] as i64,
            index: w[1] as i64,
        }),
        issues,
    })
}
