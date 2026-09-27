//! Statborg scanning from native positional source metadata.
use super::{FilmSource, FilmStatborgStream, StatborgDiagnostic, scan_statborg_frame};
use serde_json::json;

/// Checked after a whole FRAME is scanned, so the final frame may exceed it.
pub const STATBORG_MAX_RECORDS_PER_FILM: usize = 33_076;

/// Missing source/metadata yields no datable records. Metadata positions, not
/// metadata file numbers, select chunks; all nonzero chunk types are admitted.
pub fn scan_source_statborg(source: Option<&FilmSource>) -> FilmStatborgStream {
    scan_source_impl(source, None).0
}

/// Native StatRecordsCtx outputs, including the missing-manifest INFO and the
/// record-limit WARN. match_id is diagnostic context only.
pub fn scan_source_statborg_with_diagnostics(
    source: Option<&FilmSource>,
    match_id: &str,
) -> (FilmStatborgStream, Vec<StatborgDiagnostic>) {
    scan_source_impl(source, Some(match_id))
}

fn scan_source_impl(
    source: Option<&FilmSource>,
    match_id: Option<&str>,
) -> (FilmStatborgStream, Vec<StatborgDiagnostic>) {
    let mut scan = StatborgScan::new(match_id, source.map_or(0, FilmSource::num_chunks));
    if let Some(source) = source {
        for (position, meta) in source.metadata().iter().enumerate() {
            if meta.chunk_type == 0 {
                continue;
            }
            let frames = source
                .packets(position)
                .iter()
                .filter(|p| p.packet_type == 0)
                .map(|p| {
                    (
                        p.timestamp_us,
                        source.payload(p).expect("source-owned packet range"),
                    )
                });
            if scan.chunk(meta.index, meta.start_ms, frames) {
                break;
            }
        }
    }
    scan.finish()
}

pub(super) struct StatborgScan<'a> {
    stream: FilmStatborgStream,
    diagnostics: Vec<StatborgDiagnostic>,
    match_id: Option<&'a str>,
    chunks: usize,
    described: bool,
}
impl<'a> StatborgScan<'a> {
    pub(super) fn new(match_id: Option<&'a str>, chunks: usize) -> Self {
        Self {
            stream: FilmStatborgStream::default(),
            diagnostics: Vec::new(),
            match_id,
            chunks,
            described: false,
        }
    }
    /// true stops the complete scan; the record limit is shared across chunks.
    pub(super) fn chunk<'b>(
        &mut self,
        index: i64,
        start_ms: i64,
        frames: impl Iterator<Item = (u64, &'b [u8])>,
    ) -> bool {
        self.described = true;
        let mut base = None;
        for (timestamp, payload) in frames {
            let base = *base.get_or_insert(timestamp);
            let time = start_ms.wrapping_add((timestamp.wrapping_sub(base) / 1000) as i64);
            self.stream
                .records
                .extend(scan_statborg_frame(payload, time));
            if self.stream.records.len() >= STATBORG_MAX_RECORDS_PER_FILM {
                self.stream.truncated = true;
                if let Some(match_id) = self.match_id {
                    self.diagnostics.push(StatborgDiagnostic {
                        level: "WARN".into(),
                        message: "statborg: plafond d'enregistrements atteint, lecture tronquee"
                            .into(),
                        attributes: vec![
                            ("match_id".into(), json!(match_id)),
                            ("records".into(), json!(self.stream.records.len())),
                            ("limite".into(), json!(STATBORG_MAX_RECORDS_PER_FILM)),
                            ("chunk".into(), json!(index)),
                        ],
                    });
                }
                return true;
            }
        }
        false
    }
    pub(super) fn finish(mut self) -> (FilmStatborgStream, Vec<StatborgDiagnostic>) {
        if !self.described
            && let Some(match_id) = self.match_id
        {
            self.diagnostics.push(StatborgDiagnostic {
                level: "INFO".into(),
                message: "objectives: film sans chunk décrit par le manifeste — rien à dater"
                    .into(),
                attributes: vec![
                    ("match_id".into(), json!(match_id)),
                    ("chunks_du_film".into(), json!(self.chunks)),
                ],
            });
        }
        self.stream
            .records
            .sort_by_key(|r| (r.time_ms, r.slot, r.round));
        (self.stream, self.diagnostics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::{FilmSourceMetadata, scan_film_statborg};
    use serde::Deserialize;
    use std::io::Read;

    fn hex(raw: &str) -> Vec<u8> {
        (0..raw.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&raw[i..i + 2], 16).unwrap())
            .collect()
    }
    fn fixture<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(bytes)
            .read_to_end(&mut raw)
            .unwrap();
        serde_json::from_slice(&raw).unwrap()
    }
    #[derive(Deserialize)]
    struct Case {
        chunks: Vec<String>,
        metadata: Vec<FilmSourceMetadata>,
        stream: FilmStatborgStream,
        diagnostics: Vec<StatborgDiagnostic>,
    }
    #[test]
    fn native_loaded_statborg_source() {
        let cases: Vec<Case> = fixture(include_bytes!("fixtures/statborg-source-v41.json.zlib"));
        assert_eq!(cases.len(), 128);
        assert_eq!(
            cases.iter().map(|c| c.stream.records.len()).sum::<usize>(),
            680
        );
        for (i, case) in cases.into_iter().enumerate() {
            let chunks: Vec<_> = case.chunks.iter().map(|s| hex(s)).collect();
            let source =
                (!chunks.is_empty()).then(|| FilmSource::load(&chunks, &case.metadata).unwrap());
            let (stream, diagnostics) =
                scan_source_statborg_with_diagnostics(source.as_ref(), "source-oracle");
            assert_eq!(stream, case.stream, "stream {i}");
            assert_eq!(diagnostics, case.diagnostics, "diagnostics {i}");
            assert_eq!(stream, scan_source_statborg(source.as_ref()));
            if let Some(source) = source
                && case.metadata.len() >= chunks.len()
            {
                if case.metadata.len() == chunks.len() {
                    assert_eq!(
                        super::super::scan_film_statborg_with_diagnostics(
                            &source.clone().into_chunks().unwrap(),
                            "source-oracle",
                        ),
                        (stream.clone(), diagnostics.clone()),
                        "legacy diagnostics {i}"
                    );
                }
                assert_eq!(
                    scan_film_statborg(&source.into_chunks().unwrap()),
                    stream,
                    "legacy {i}"
                );
            }
        }
    }
    #[derive(Deserialize)]
    struct LimitCase {
        payload: String,
        stream: FilmStatborgStream,
        diagnostics: Vec<StatborgDiagnostic>,
    }
    fn packet(timestamp: u64, payload: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 16];
        bytes[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes[8..16].copy_from_slice(&timestamp.to_le_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }
    #[test]
    fn native_loaded_statborg_whole_frame_limit() {
        let case: LimitCase = fixture(include_bytes!(
            "fixtures/statborg-source-limit-v41.json.zlib"
        ));
        let payload = hex(&case.payload);
        let one = packet(1000, &payload);
        let many: Vec<_> = (0..11026)
            .flat_map(|j| packet(j * 1000, &payload))
            .collect();
        let metadata = [
            FilmSourceMetadata {
                index: 18,
                chunk_type: 2,
                start_ms: 400,
            },
            FilmSourceMetadata {
                index: -7,
                chunk_type: 3,
                start_ms: -200,
            },
            FilmSourceMetadata {
                index: 99,
                chunk_type: 2,
                start_ms: 90000,
            },
        ];
        let source = FilmSource::load(&[one.clone(), many, one], &metadata).unwrap();
        let (stream, diagnostics) =
            scan_source_statborg_with_diagnostics(Some(&source), "limit-oracle");
        assert_eq!(stream, case.stream);
        assert_eq!(diagnostics, case.diagnostics);
        assert_eq!(stream.records.len(), 33078);
        assert!(stream.truncated);
        let chunks = source.into_chunks().unwrap();
        assert_eq!(scan_film_statborg(&chunks), stream);
        assert_eq!(
            super::super::scan_film_statborg_with_diagnostics(&chunks, "limit-oracle"),
            (stream.clone(), diagnostics.clone())
        );

        // A real LegacyFilm constructor and its serde export must retain the native
        // warning, not merely a manually populated field. The bootstrap supplies
        // a valid registry around the independently generated cap oracle.
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut bootstrap = vec![0; 8 + 37 * 16640];
        bootstrap[..4].copy_from_slice(&41_u32.to_le_bytes());
        bootstrap[4..8].copy_from_slice(&27_u32.to_le_bytes());
        bootstrap[8 + 36 * 16640] = 0xff;
        for i in 0..43 {
            let name = match i {
                0 => "object-position-dynamic-precision-component",
                1 => "object-translational-velocity-dynamic-precision-component",
                4 => "object-body-vitality-component",
                5 => "object-shield-vitality-component",
                21 => "unit-desired-aiming-vector-component",
                25 => "unit-command-tick-component",
                30 | 33 => "weapon-state-ammo",
                31 | 34 => "weapon-state-rounds-inventory",
                35 => "weapon-state-overheated",
                42 => "biped-desired-weapon-set",
                _ => "test-opaque-component",
            };
            let start = 8 + 35 * 16640 + i * 260;
            bootstrap[start..start + name.len()].copy_from_slice(name.as_bytes());
        }
        let mut chunks = chunks;
        chunks.push(FilmChunkData {
            metadata: FilmChunk {
                index: 0,
                chunk_type: 1,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: bootstrap.len() as i64,
                file_relative_path: String::new(),
            },
            data: bootstrap,
        });
        let film = super::super::LegacyFilm::try_from_chunks(
            &chunks,
            super::super::DecodeOptions {
                match_id: Some("limit-oracle".into()),
                ..super::super::DecodeOptions::v41()
            },
        )
        .unwrap();
        assert_eq!(film.statborg, stream);
        assert_eq!(film.statborg_diagnostics, diagnostics);
        let mut json = serde_json::to_value(&film).unwrap();
        let restored: super::super::LegacyFilm = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(restored.statborg, stream);
        assert_eq!(restored.statborg_diagnostics, diagnostics);
        // Older portable exports did not carry this field.
        json.as_object_mut().unwrap().remove("statborg_diagnostics");
        let old: super::super::LegacyFilm = serde_json::from_value(json).unwrap();
        assert!(old.statborg_diagnostics.is_empty());
        assert_eq!(old.statborg, stream);

        chunks.retain(|c| c.metadata.index != -7);
        let small =
            super::super::LegacyFilm::try_from_chunks(&chunks, super::super::DecodeOptions::v41())
                .unwrap();
        assert!(!small.statborg.truncated);
        assert!(small.statborg_diagnostics.is_empty());
        assert!(
            serde_json::to_value(&small)
                .unwrap()
                .get("statborg_diagnostics")
                .is_none()
        );
    }
    #[test]
    #[ignore = "requires the 32 local v41 films"]
    fn local_loaded_statborg_corpus() {
        let expected: std::collections::BTreeMap<String, FilmStatborgStream> =
            fixture(include_bytes!("fixtures/statborg-corpus-v41.json.zlib"));
        assert_eq!(expected.len(), 32);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films");
        let mut count = 0;
        for (key, expected) in expected {
            let path = root.join(&key);
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(path.join("film.json")).unwrap()).unwrap();
            let mut chunks = Vec::new();
            let mut metadata = Vec::new();
            for c in manifest["chunks"].as_array().unwrap() {
                chunks.push(std::fs::read(path.join(c["file"].as_str().unwrap())).unwrap());
                metadata.push(FilmSourceMetadata {
                    index: c["index"].as_i64().unwrap(),
                    chunk_type: c["chunk_type"].as_i64().unwrap(),
                    start_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                });
            }
            let source = FilmSource::load(&chunks, &metadata).unwrap();
            drop(chunks);
            let actual = scan_source_statborg(Some(&source));
            assert_eq!(actual, expected, "{key}");
            count += actual.records.len();
            eprintln!("{key}: {} records match", actual.records.len());
        }
        assert_eq!(count, 16113);
    }
}
