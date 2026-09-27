//! Native resolved profile metadata and loaded-source resolution for v41.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum NativeProfileIssue {
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
pub struct NativeResolvedFilmProfile {
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
impl NativeResolvedFilmProfile {
    pub fn build(&self) -> &str {
        self.identity.as_ref().map_or("", |id| id.build.as_str())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NativeProfileResolveError {
    #[error("unsupported Theater profile major version {0}; supported: 41")]
    UnsupportedVersion(u32),
}

/// Native ResolveProfile source behavior, restricted to v41 identification.
/// Unavailable identities preserve readable format keys. Profile resolution reads
/// original source bytes independently of mutations to a context's cached registry.
pub fn resolve_native_film_profile(
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    fn inflate(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        flate2::read::ZlibDecoder::new(data)
            .read_to_end(&mut out)
            .unwrap();
        out
    }
    #[test]
    fn native_profile_from_source_and_lazy_context() {
        let oracle: Value = serde_json::from_slice(&inflate(include_bytes!(
            "fixtures/profile-source-v41.json.zlib"
        )))
        .unwrap();
        let base = inflate(include_bytes!("fixtures/bootstrap-v41.zlib"));
        let offset = oracle["build_offset"].as_u64().unwrap() as usize;
        let rows = oracle["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 64);
        for row in rows {
            let mut data = base.clone();
            data[4..8]
                .copy_from_slice(&(row["format_input"].as_u64().unwrap() as u32).to_le_bytes());
            data[offset..offset + 32].fill(0);
            let build = row["build_input"].as_str().unwrap();
            data[offset..offset + build.len()].copy_from_slice(build.as_bytes());
            data.truncate(row["cut"].as_u64().unwrap() as usize);
            let missing = row["missing"].as_bool().unwrap();
            let source = (!missing).then(|| {
                FilmSource::load(
                    &[data],
                    &[FilmSourceMetadata {
                        index: 0,
                        chunk_type: 1,
                        start_ms: 0,
                    }],
                )
                .unwrap()
            });
            let map: Option<FilmMapBounds> = serde_json::from_value(row["entry"].clone()).unwrap();
            let p = resolve_native_film_profile(source.as_ref(), map.as_ref()).unwrap();
            assert_eq!(p.registry_present, !missing);
            assert_eq!(
                u64::from(p.format_version.unwrap_or(0)),
                row["format"].as_u64().unwrap()
            );
            assert_eq!(p.build(), row["build"].as_str().unwrap());
            assert_eq!(
                p.identity.is_some(),
                row["identity_read"].as_bool().unwrap()
            );
            if let Some(id) = &p.identity {
                assert_eq!(
                    json!({"Version":id.version,"Build":id.build,"Flavor":id.flavor,"BuildID":id.build_id,"Changelist":id.changelist,"MatchStartUnix":id.match_start_unix,"FormatVersion":id.format_version,"TypeVersions":id.type_versions,"RegistryBlocks":id.registry_blocks,"RegistryFingerprint":id.registry_fingerprint,"RegistryNamedSlots":id.registry_named_slots,"BuildOffset":id.build_offset,"BodyBit":id.body_bit,"ControleDeCorruption":id.corruption_checks}),
                    row["identity"]
                );
            }
            assert_eq!(
                json!({"MajorVersion":p.major_version.unwrap_or(0),"Lue":p.major_version.is_some(),"Implantation":p.highlight,"GamertagOffsetBytes":p.highlight.offset_bytes()}),
                row["highlight"]
            );
            assert_eq!(
                json!({"EnTeteBits":p.keyframe.header_bits,"MotDeTailleBits":p.keyframe.size_word_bits}),
                row["keyframe"]
            );
            assert_eq!(
                json!(p.mpp.unwrap_or(FilmMppWidths { lead: 0, index: 0 })),
                row["mpp"]
            );
            assert_eq!(
                json!({"PersoBytes":p.personalization_bytes.unwrap_or(0),"DeltaBits":p.personalization_delta_bits.unwrap_or(0),"Connu":p.personalization_bytes.is_some()}),
                row["slots"]
            );
            assert_eq!(
                p.issues
                    .iter()
                    .any(|i| matches!(i, NativeProfileIssue::UnknownFormat(_))),
                row["unknown_format"].as_bool().unwrap()
            );
            assert_eq!(
                p.issues
                    .iter()
                    .any(|i| matches!(i, NativeProfileIssue::UnknownBuild(_))),
                row["unknown_build"].as_bool().unwrap()
            );
            assert_eq!(
                p.issues
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
                row["error"].as_str().unwrap()
            );
            assert_eq!(p.map, map);
            if let Some(map) = &p.map {
                assert_eq!(
                    map,
                    &serde_json::from_value::<FilmMapBounds>(row["map"].clone()).unwrap()
                );
            }
            let m = &p.movement;
            let e = &row["movement"];
            for (d, key) in [
                (m.traversal.clone(), "Traversal"),
                (m.world_object.clone(), "WorldObject"),
            ] {
                assert_eq!(
                    json!({"IndexW":d.index_bits,"AxisW":d.axis_bits,"Region":d.region}),
                    e[key]
                );
            }
            assert_eq!(
                m.delta_quantum.to_bits(),
                (e["DeltaQuantum"].as_f64().unwrap() as f32).to_bits()
            );
            for (axis, range) in m.range.iter().enumerate() {
                for (k, name) in ["Min", "Max"].iter().enumerate() {
                    assert_eq!(
                        range[k].to_bits(),
                        (e["Range"][axis][name].as_f64().unwrap() as f32).to_bits()
                    );
                }
            }
            assert_eq!(
                m.delta_axis_width as u64,
                e["DeltaAxisWidth"].as_u64().unwrap()
            );
            assert_eq!(m.full_precision, e["FullPrecision"].as_bool().unwrap());
            assert_eq!(
                m.delta_has_handle_tail,
                e["DeltaHasHandleTail"].as_bool().unwrap()
            );
            assert_eq!(m.calibrated_skip, e["CalibratedSkip"].as_bool().unwrap());
            assert_eq!(
                m.mobility_action_extra_bits,
                e["MobilityActionExtraBits"].as_i64().unwrap()
            );
            let mut context = NativeFilmContext::new(source.as_ref());
            if let Ok(cached) = context.registry_mut() {
                cached.registry.format_version = 12345;
            }
            let original = context.profile().unwrap();
            let mut copied = original.clone();
            if let Some(id) = &mut copied.identity {
                id.build = "changed".into();
                if let Some(v) = id.type_versions.first_mut() {
                    *v += 1;
                }
            }
            copied.movement.delta_quantum = 1.;
            assert_eq!(context.profile().unwrap(), original);
            let mut expected = p.clone();
            expected.map = None;
            assert_eq!(original, expected);
            let restored: NativeResolvedFilmProfile =
                serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
            assert_eq!(restored, p);
        }
    }
}
