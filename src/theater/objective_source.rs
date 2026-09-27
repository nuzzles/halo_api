//! Loaded-source objective primitives. Metadata positions select buffers;
//! metadata file numbers select the footer and never substitute for positions.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveSourceChunk {
    pub position: usize,
    pub metadata: FilmSourceMetadata,
}

/// Native manifestChunks: preserve metadata order and every nonzero type,
/// including rows whose position has no loaded buffer.
pub fn objective_manifest_chunks(source: Option<&FilmSource>) -> Vec<ObjectiveSourceChunk> {
    source
        .into_iter()
        .flat_map(|s| s.metadata().iter().enumerate())
        .filter(|(_, m)| m.chunk_type != 0)
        .map(|(position, &metadata)| ObjectiveSourceChunk { position, metadata })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveSourceFooter<'a> {
    pub chunk: ObjectiveSourceChunk,
    /// None means the selected metadata row has no loaded buffer. An empty
    /// loaded buffer is Some(&[]). Neither case falls back to a lower index.
    pub data: Option<&'a [u8]>,
}

/// Native footerData selects the first strictly highest type-3 index above -1.
/// A selected row and absent bytes remain distinct from no selected footer.
pub fn objective_source_footer(source: Option<&FilmSource>) -> Option<ObjectiveSourceFooter<'_>> {
    let source = source?;
    let mut selected = None;
    let mut index = -1;
    for chunk in objective_manifest_chunks(Some(source)) {
        if chunk.metadata.chunk_type == 3 && chunk.metadata.index > index {
            index = chunk.metadata.index;
            selected = Some(ObjectiveSourceFooter {
                data: source.chunk(chunk.position),
                chunk,
            });
        }
    }
    selected
}

pub fn scan_source_objective_footer(source: Option<&FilmSource>) -> Vec<ObjectiveFooterEvent> {
    objective_source_footer(source).map_or_else(Vec::new, |f| {
        scan_objective_footer(f.data.unwrap_or_default())
    })
}

/// Native collectCaptureBursts: only type-2 chunks, frame order within each
/// chunk, first FRAME skipped, and final match-time ordering across chunks.
pub fn scan_source_capture_bursts(source: Option<&FilmSource>) -> Vec<i64> {
    let Some(source) = source else {
        return Vec::new();
    };
    let mut bursts = Vec::new();
    for chunk in objective_manifest_chunks(Some(source)) {
        if chunk.metadata.chunk_type != 2 {
            continue;
        }
        let frames = source
            .packets(chunk.position)
            .iter()
            .filter(|p| p.packet_type == 0)
            .map(|p| {
                (
                    p.timestamp_us,
                    source.payload(p).expect("source-owned packet range"),
                )
            });
        bursts.extend(capture_bursts_in_frames(frames, chunk.metadata.start_ms));
    }
    bursts.sort_unstable();
    bursts
}

pub fn extract_source_objective_events(
    match_id: &str,
    variant: &str,
    source: Option<&FilmSource>,
    major_version: i32,
    roster: &BTreeMap<String, i64>,
) -> Result<ExtractedObjectives, DecodeError> {
    extract_source_objective_events_with_roster(match_id, variant, source, major_version, |xuid| {
        roster.get(xuid).copied()
    })
}

/// Loaded-source native `Extract` with a stateful roster callback.
/// Source scanning precedes lookups; the callback cannot override recorded teams.
/// Unsupported versions fail before scanning or invoking the callback.
pub fn extract_source_objective_events_with_roster(
    match_id: &str,
    variant: &str,
    source: Option<&FilmSource>,
    major_version: i32,
    roster: impl FnMut(&str) -> Option<i64>,
) -> Result<ExtractedObjectives, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let kind = replay_objective_type(variant);
    if !matches!(kind, "flag" | "zone" | "hill" | "skull") {
        return Ok(ExtractedObjectives::default());
    }
    let bursts = if kind == "flag" {
        scan_source_capture_bursts(source)
    } else {
        Vec::new()
    };
    Ok(extract_objective_events_with_roster(
        match_id,
        variant,
        &bursts,
        &scan_source_objective_footer(source),
        roster,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Extraction {
        variant: String,
        roster: BTreeMap<String, i64>,
        expected: ExtractedObjectives,
    }
    #[derive(Deserialize)]
    struct Callback {
        variant: String,
        calls: Vec<String>,
        expected: ExtractedObjectives,
    }
    #[derive(Deserialize)]
    struct Case {
        callback_hex: String,
        callback_packets: String,
        callback_start: i64,
        callbacks: Vec<Callback>,
        extractions: Vec<Extraction>,
        chunks: Vec<String>,
        metadata: Vec<FilmSourceMetadata>,
        manifest: Vec<ObjectiveSourceChunk>,
        footer_found: bool,
        footer_nil: bool,
        footer_hex: String,
        footer: Vec<ObjectiveFooterEvent>,
        bursts: Vec<i64>,
    }
    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
    #[test]
    fn native_objective_source_selection() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/objective-source-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 128);
        assert!(cases.iter().any(|c| c.footer_found && c.footer_nil));
        assert!(
            cases
                .iter()
                .any(|c| c.footer_found && !c.footer_nil && c.footer_hex.is_empty())
        );
        assert!(cases.iter().any(|c| !c.footer.is_empty()));
        assert!(cases.iter().any(|c| !c.bursts.is_empty()));
        for (i, case) in cases.into_iter().enumerate() {
            let callback_source = FilmSource::load(
                &[hex(&case.callback_hex), hex(&case.callback_packets)],
                &[
                    FilmSourceMetadata {
                        index: 0,
                        chunk_type: 3,
                        start_ms: 0,
                    },
                    FilmSourceMetadata {
                        index: 1,
                        chunk_type: 2,
                        start_ms: case.callback_start,
                    },
                ],
            )
            .unwrap();
            for callback in &case.callbacks {
                let mut calls = Vec::new();
                let actual = extract_source_objective_events_with_roster(
                    "callback-fixture",
                    &callback.variant,
                    Some(&callback_source),
                    41,
                    |xuid| {
                        calls.push(xuid.to_owned());
                        (calls.len() % 3 != 0).then_some((calls.len() % 4) as i64)
                    },
                )
                .unwrap();
                assert_eq!(
                    calls, callback.calls,
                    "callback order {i} {}",
                    callback.variant
                );
                assert_eq!(
                    actual, callback.expected,
                    "callback result {i} {}",
                    callback.variant
                );
            }
            let chunks: Vec<_> = case.chunks.iter().map(|s| hex(s)).collect();
            let source =
                (!chunks.is_empty()).then(|| FilmSource::load(&chunks, &case.metadata).unwrap());
            assert_eq!(
                objective_manifest_chunks(source.as_ref()),
                case.manifest,
                "manifest {i}"
            );
            assert_eq!(case.extractions.len(), 24);
            for extraction in &case.extractions {
                assert_eq!(
                    extract_source_objective_events(
                        "source-fixture",
                        &extraction.variant,
                        source.as_ref(),
                        41,
                        &extraction.roster,
                    )
                    .unwrap(),
                    extraction.expected,
                    "extraction {i} {} {:?}",
                    extraction.variant,
                    extraction.roster,
                );
            }
            let selected = objective_source_footer(source.as_ref());
            assert_eq!(selected.is_some(), case.footer_found, "found {i}");
            let bytes = selected.and_then(|s| s.data);
            assert_eq!(bytes.is_none(), case.footer_nil, "nil {i}");
            assert_eq!(
                bytes.unwrap_or_default(),
                hex(&case.footer_hex),
                "bytes {i}"
            );
            if let Some(selected) = selected
                && let Some(bytes) = selected.data
            {
                let original = source
                    .as_ref()
                    .unwrap()
                    .chunk(selected.chunk.position)
                    .unwrap();
                assert_eq!(bytes.as_ptr(), original.as_ptr(), "borrowed {i}");
                assert_eq!(bytes.len(), original.len());
            }
            assert_eq!(
                scan_source_objective_footer(source.as_ref()),
                case.footer,
                "footer {i}"
            );
            assert_eq!(
                scan_source_capture_bursts(source.as_ref()),
                case.bursts,
                "bursts {i}"
            );
        }
        assert!(matches!(
            extract_source_objective_events("", "CTF", None, 42, &BTreeMap::new()),
            Err(DecodeError::UnsupportedVersion(42))
        ));
    }
}
