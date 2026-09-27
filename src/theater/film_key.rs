//! Recorded film-key diagnostics, independent of precision/profile installation.
use super::{
    DecodeError, FilmIdentity, FilmProfileIssue, decode_film_identity, parse_registry,
    unknown_build_metric_pairs, unknown_format_metric_pairs,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmKey {
    pub format_version: u32,
    pub major_version: Option<u32>,
    pub build: String,
    pub written: String,
    pub readable: bool,
    pub issues: Vec<FilmProfileIssue>,
}
impl FilmKey {
    pub fn refused(&self) -> bool {
        !self.issues.is_empty()
    }
    /// Counter deltas only; no process-global mutation. Both reasons are retained.
    pub fn metric_pairs(&self) -> Vec<(String, i64)> {
        self.issues
            .iter()
            .flat_map(|issue| match issue {
                FilmProfileIssue::UnknownFormat(format) => {
                    unknown_format_metric_pairs(i64::from(*format)).to_vec()
                }
                FilmProfileIssue::UnknownBuild(build) => unknown_build_metric_pairs(build).to_vec(),
                _ => Vec::new(),
            })
            .collect()
    }
}
// This is native table membership, not a decoder for older formats/builds. Known
// format and available precision are distinct facts in the reference.
fn known_format(format: u32) -> bool {
    matches!(format, 20 | 21 | 24 | 25 | 27)
}
fn known_build(build: &str) -> bool {
    matches!(
        build,
        "HI_1_13_0"
            | "HI_1_12_0"
            | "HI_1_11_0"
            | "HI_1_10_0"
            | "HI_1_9_0"
            | "HI_1_8_0"
            | "HI_1_4_1"
    )
}

pub(crate) fn film_key_from_identity(
    format: u32,
    major: Option<u32>,
    identity: Option<&FilmIdentity>,
) -> FilmKey {
    let build = identity.map(|id| id.build.clone()).unwrap_or_default();
    let readable = !build.is_empty() || major.is_some();
    let written = if !build.is_empty() {
        format!("build={build}")
    } else {
        major.map(|v| format!("majeure={v}")).unwrap_or_default()
    };
    let mut issues = Vec::new();
    if readable {
        if !known_format(format) {
            issues.push(FilmProfileIssue::UnknownFormat(format));
        }
        // All v41 films without a recorded build are unknown to the native table.
        if !known_build(&build) {
            issues.push(FilmProfileIssue::UnknownBuild(build.clone()));
        }
    }
    FilmKey {
        format_version: format,
        major_version: major,
        build,
        written,
        readable,
        issues,
    }
}

/// Read the native first little-endian u32 without interpreting support.
/// None means fewer than four bytes; Some(0) is a recorded zero, not absence.
/// This helper does not decompress data or enable older-version decoding.
pub fn film_major_version_from_header(data: &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(..4)?.try_into().ok()?))
}

/// Inspect decompressed v41 bootstrap bytes. Partial headers retain absence;
/// other major versions are explicitly rejected. Malformed identity is not a key.
pub fn decode_v41_film_key(data: &[u8]) -> Result<FilmKey, DecodeError> {
    let word = |at: usize| {
        data.get(at..at + 4)
            .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
    };
    let major = film_major_version_from_header(data);
    if let Some(version) = major.filter(|v| *v != 41) {
        return Err(DecodeError::UnsupportedVersion(version as i32));
    }
    let identity = parse_registry(data).and_then(|r| decode_film_identity(data, &r).ok().flatten());
    Ok(film_key_from_identity(
        word(4).unwrap_or(0),
        major,
        identity.as_ref(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Pair {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Value")]
        value: i64,
    }
    #[derive(Deserialize)]
    struct Output {
        format: u32,
        major: u32,
        major_read: bool,
        build: String,
        written: String,
        readable: bool,
        refused: bool,
        unknown_format: bool,
        unknown_build: bool,
        metrics: Vec<Pair>,
    }
    #[derive(Deserialize)]
    struct Row {
        build: String,
        format: u32,
        length: usize,
        output: Output,
    }
    #[derive(Deserialize)]
    struct Oracle {
        headers: Vec<serde_json::Value>,
        offset: usize,
        rows: Vec<Row>,
    }
    #[test]
    fn native_v41_film_key() {
        let inflate = |data: &[u8]| {
            let mut out = Vec::new();
            flate2::read::ZlibDecoder::new(data)
                .read_to_end(&mut out)
                .unwrap();
            out
        };
        let base = inflate(include_bytes!("fixtures/bootstrap-v41.zlib"));
        let oracle: Oracle =
            serde_json::from_slice(&inflate(include_bytes!("fixtures/film-key-v41.json.zlib")))
                .unwrap();
        assert_eq!(oracle.headers.len(), 27);
        for case in &oracle.headers {
            let word = case["word"].as_u64().unwrap() as u32;
            let mut data = word.to_le_bytes().to_vec();
            data.resize(8, 0);
            data.truncate(case["length"].as_u64().unwrap() as usize);
            let major = film_major_version_from_header(&data);
            assert_eq!(major.is_some(), case["read"]);
            assert_eq!(major.unwrap_or(0), case["major"]);
            let highlight = super::super::v41_highlight_profile_from_header(&data);
            if major.is_none() || major == Some(41) {
                let highlight = highlight.unwrap();
                let native = &case["highlight_profile"];
                assert_eq!(highlight.major_version.is_some(), native["Lue"]);
                assert_eq!(highlight.major_version.unwrap_or(0), native["MajorVersion"]);
                assert_eq!(highlight.implantation, native["Implantation"]);
                assert_eq!(
                    highlight.gamertag_offset_bytes,
                    native["GamertagOffsetBytes"]
                );
                assert_eq!(
                    serde_json::from_value::<super::super::FilmHighlightProfile>(
                        serde_json::to_value(&highlight).unwrap()
                    )
                    .unwrap(),
                    highlight
                );
            } else {
                assert!(matches!(highlight, Err(DecodeError::UnsupportedVersion(_))));
            }
            if major.is_some_and(|m| m != 41) {
                assert!(matches!(
                    decode_v41_film_key(&data),
                    Err(DecodeError::UnsupportedVersion(_))
                ));
            }
        }
        assert_eq!(oracle.rows.len(), 512);
        let mut accepted = 0;
        let mut both = 0;
        for (i, row) in oracle.rows.into_iter().enumerate() {
            let mut data = base.clone();
            data[4..8].copy_from_slice(&row.format.to_le_bytes());
            data[oracle.offset..oracle.offset + 32].fill(0);
            data[oracle.offset..oracle.offset + row.build.len()]
                .copy_from_slice(row.build.as_bytes());
            data.truncate(row.length);
            let key = decode_v41_film_key(&data).unwrap();
            let o = row.output;
            assert_eq!(key.format_version, o.format, "format {i}");
            assert_eq!(
                key.major_version,
                o.major_read.then_some(o.major),
                "major {i}"
            );
            assert_eq!(key.build, o.build, "build {i}");
            assert_eq!(key.written, o.written, "written {i}");
            assert_eq!(key.readable, o.readable, "readable {i}");
            assert_eq!(key.refused(), o.refused, "refused {i}");
            let mut issues = Vec::new();
            if o.unknown_format {
                issues.push(FilmProfileIssue::UnknownFormat(o.format));
            }
            if o.unknown_build {
                issues.push(FilmProfileIssue::UnknownBuild(o.build));
            }
            assert_eq!(key.issues, issues, "issues {i}");
            assert_eq!(
                key.metric_pairs(),
                o.metrics
                    .into_iter()
                    .map(|p| (p.name, p.value))
                    .collect::<Vec<_>>(),
                "metrics {i}"
            );
            assert_eq!(
                serde_json::from_value::<FilmKey>(serde_json::to_value(&key).unwrap()).unwrap(),
                key
            );
            accepted += usize::from(key.readable && !key.refused());
            both += usize::from(key.issues.len() == 2);
        }
        assert!(accepted > 0 && both > 0);
        assert!(matches!(
            decode_v41_film_key(&39u32.to_le_bytes()),
            Err(DecodeError::UnsupportedVersion(39))
        ));
    }
}
