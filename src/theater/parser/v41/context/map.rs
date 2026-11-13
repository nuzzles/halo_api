//! Reference data models.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FilmMapBounds {
    pub module: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    #[serde(rename = "axisWidths")]
    pub axis_widths: [usize; 3],
    #[serde(default)]
    pub region: u32,
    #[serde(default, rename = "regionIndexBits")]
    pub region_index_bits: usize,
}

/// Fixed v41 format-27 component widths.
pub const REFERENCE_MPP_DEFAULT_WIDTHS: [usize; 2] = [9, 5];

/// Bind external map data only to the exact recorded build and level tag.
pub(crate) fn recorded_level(
    registry: &crate::theater::parser::v41::chunks::registry::source::FilmRegistryRead,
) -> Option<&'static crate::theater::film::catalog::maps::levels::LevelCalibration> {
    use crate::theater::parser::v41::chunks::registry::source::*;
    let header = registry.bootstrap_header.as_ref()?;
    let identity = header.identification.as_ref()?;
    let options = identity.game_options.as_ref()?;
    if header.stop != BootstrapHeaderStop::Complete
        || identity.stop != BootstrapIdentificationStop::Complete
        || options.stop != GameOptionsStop::Complete
    {
        return None;
    }
    let text = |kind| {
        let bytes = &header
            .fields
            .iter()
            .find(|f| f.kind == kind)?
            .raw
            .as_ref()?
            .bytes;
        let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
        std::str::from_utf8(&bytes[..end]).ok()
    };
    let id = header
        .fields
        .iter()
        .find(|f| f.kind == BootstrapHeaderFieldKind::BuildId)?;
    if id.width != 32 {
        return None;
    }
    let id = id.raw.as_ref()?;
    if id.bit_len != 32 {
        return None;
    }
    let build_id = u32::from_le_bytes(id.bytes.as_slice().try_into().ok()?);
    let branch = text(BootstrapHeaderFieldKind::BuildBranch)?;
    let tag = options.entries.iter().find_map(|e| match e {
        GameOptionsEntry::Field {
            kind: GameOptionsFieldKind::LevelTag,
            bits,
        } => bits.raw.as_ref().map(|r| r.low_u64() as u32),
        _ => None,
    })?;
    crate::theater::film::catalog::maps::levels::resolve_recorded(build_id, branch, tag)
}

pub(crate) fn level_precision(
    level: &crate::theater::film::catalog::maps::levels::LevelCalibration,
    precision: u8,
) -> Option<super::movement::PrecisionDescriptor> {
    Some(super::movement::PrecisionDescriptor {
        index_bits: level.index_bits as u64,
        region: level.play.bounds.region,
        axis_bits: level
            .play
            .bounds
            .axis_bits_at_precision(precision)?
            .map(|w| w as u64),
        regions: level
            .regions()
            .map(|r| {
                Some((
                    r.bounds.region,
                    r.bounds
                        .axis_bits_at_precision(precision)?
                        .map(|w| w as u64),
                ))
            })
            .collect::<Option<_>>()?,
    })
}

#[cfg(test)]
mod tests;
