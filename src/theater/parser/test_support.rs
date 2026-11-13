//! Test-only captured source adapters.
use crate::clients::hi::models::{RawChunkKind, RawFilmChunk};
use crate::theater::film::{ChunkKind, Film, ParseError};
use crate::theater::parser::{ChunkReader, ParsedFilm};
use crate::theater::parser::{FilmParseContext, ParseLimits};
use std::io::Write;

/// Wrap clear fixture bytes as a downloaded zlib chunk.
pub(crate) fn zlib_fixture_data(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

pub(crate) fn zlib_fixture_chunk(mut chunk: RawFilmChunk) -> RawFilmChunk {
    chunk.data = zlib_fixture_data(&chunk.data);
    chunk
}

/// Adapt the historical clear-byte fixtures to the downloaded zlib input contract.
/// Some newer tests already construct zlib chunks; this distinction exists only
/// in test support, never in the production chunk reader.
fn fixture_chunks(chunks: impl IntoIterator<Item = RawFilmChunk>) -> Vec<RawFilmChunk> {
    chunks
        .into_iter()
        .map(|chunk| {
            let bytes = &chunk.data;
            let has_zlib_header = bytes.get(..2).is_some_and(|header| {
                header[0] & 15 == 8
                    && header[0] >> 4 <= 7
                    && u16::from_be_bytes([header[0], header[1]]) % 31 == 0
            });
            if has_zlib_header {
                chunk
            } else {
                zlib_fixture_chunk(chunk)
            }
        })
        .collect()
}

pub(crate) fn parse_fixture(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
) -> Result<Film, ParseError> {
    ChunkReader::read_structural(
        fixture_chunks(chunks),
        ParseLimits::default(),
        FilmParseContext::default(),
    )?
    .into_film()
}

pub(crate) fn parse_fixture_with_limits(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
    limits: ParseLimits,
) -> Result<Film, ParseError> {
    Film::parse_with_limits(fixture_chunks(chunks), limits)
}

pub(crate) fn parse_fixture_structural(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
) -> Result<ParsedFilm, ParseError> {
    parse_fixture_structural_with_context(chunks, FilmParseContext::default())
}

pub(crate) fn parse_fixture_structural_with_context(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
    context: FilmParseContext,
) -> Result<ParsedFilm, ParseError> {
    ChunkReader::read_structural(fixture_chunks(chunks), ParseLimits::default(), context)
}

pub(crate) fn parse_fixture_structural_with_limits(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
    limits: ParseLimits,
) -> Result<ParsedFilm, ParseError> {
    ChunkReader::read_structural(fixture_chunks(chunks), limits, FilmParseContext::default())
}

// Legacy fixture adapters are test-only; production parsing consumes RawFilmChunk directly.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FixtureChunkMetadata {
    pub index: usize,
    pub chunk_type: i64,
    pub start_ms: i64,
}

pub(crate) struct FixtureFilmSource {
    original_chunks: Vec<Vec<u8>>,
    metadata: Vec<FixtureChunkMetadata>,
}
impl FixtureFilmSource {
    pub(crate) fn load(
        chunks: &[impl AsRef<[u8]>],
        metadata: &[FixtureChunkMetadata],
    ) -> Result<Self, &'static str> {
        if chunks.is_empty() {
            return Err("empty film input");
        }
        Ok(Self {
            original_chunks: chunks.iter().map(|c| c.as_ref().to_vec()).collect(),
            metadata: metadata.to_vec(),
        })
    }
    pub(crate) fn original_chunks(&self) -> &[Vec<u8>] {
        &self.original_chunks
    }
    pub(crate) fn metadata(&self) -> &[FixtureChunkMetadata] {
        &self.metadata
    }
}

/// An explicitly empty summary for synthetic recordings that exercise replication only.
pub(crate) fn empty_summary() -> RawFilmChunk {
    RawFilmChunk {
        kind: ChunkKind::Summary.into(),
        index: i32::MAX,
        start_ms: 0,
        data: zlib_fixture_data(&[]),
    }
}

/// A present but empty replication chunk for tests of independent film sections.
pub(crate) fn empty_replication() -> RawFilmChunk {
    RawFilmChunk {
        kind: ChunkKind::Replication.into(),
        index: 1,
        start_ms: 0,
        data: zlib_fixture_data(&[]),
    }
}

/// Recognized registry and fixed bootstrap header for public admission tests.
/// No identification/map/options data are invented beyond that fixed header.
pub(crate) fn registry_with_corruption_policy(enabled: bool) -> RawFilmChunk {
    use std::io::Read;
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../fixtures/bootstrap-v41/registry-prefix.bin.zlib").as_slice(),
    )
    .read_to_end(&mut data)
    .unwrap();
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/bootstrap-v41/cases.json")).unwrap();
    data.extend(
        cases[usize::from(enabled)]["hex"]
            .as_str()
            .unwrap()
            .as_bytes()
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap()),
    );
    set_native_event_versions(&mut data);
    RawFilmChunk {
        kind: ChunkKind::Registry.into(),
        index: 0,
        start_ms: 0,
        data,
    }
}

/// A recognized registry with recorded map IDs and game options.
pub(crate) fn registry_with_complete_setup() -> RawFilmChunk {
    let mut registry = registry_with_corruption_policy(false);
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/bootstrap-v41/cases.json")).unwrap();
    let identity: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/bootstrap-v41/identification.json"
    ))
    .unwrap();
    let options: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/bond-v2/game-options.json")).unwrap();
    let bootstrap_start = 0xcb208 * 8;
    let identity_start = cases[0]["bit_len"].as_u64().unwrap() as usize;
    let identity_end = identity[0]["bit_len"].as_u64().unwrap() as usize;
    for (case, source_start, bit_len, destination) in [
        (
            &identity[0],
            identity_start,
            identity_end - identity_start,
            bootstrap_start + identity_start,
        ),
        (
            &options[0],
            0,
            options[0]["bit_len"].as_u64().unwrap() as usize,
            bootstrap_start + identity_end,
        ),
    ] {
        let source = case["hex"]
            .as_str()
            .unwrap()
            .as_bytes()
            .chunks(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        registry.data.resize((destination + bit_len).div_ceil(8), 0);
        for bit in 0..bit_len {
            let target = destination + bit;
            let mask = 1 << (7 - target % 8);
            registry.data[target / 8] = (registry.data[target / 8] & !mask)
                | (((source[(source_start + bit) / 8] >> (7 - (source_start + bit) % 8)) & 1)
                    << (7 - target % 8));
        }
    }
    // The independent options fixture exercises a different three-bit mode.
    // Public-film tests require the supported Multiplayer value (2).
    let game_mode_start = bootstrap_start + identity_end;
    for (offset, bit) in [0u8, 1, 0].into_iter().enumerate() {
        let target = game_mode_start + offset;
        let mask = 1 << (7 - target % 8);
        registry.data[target / 8] = (registry.data[target / 8] & !mask) | (bit << (7 - target % 8));
    }
    registry.data = zlib_fixture_data(&registry.data);
    registry
}

/// Populate synthetic public-admission inputs from the independent native
/// oracle, leaving the deliberately mixed raw bootstrap goldens unchanged.
pub(crate) fn set_native_event_versions(data: &mut [u8]) {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/event-admission-v41/reference.json"
    ))
    .unwrap();
    for (code, version) in oracle["results"]["Native"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let at = 0xcb208 + 4 * code;
        data[at..at + 4].copy_from_slice(&(version.as_u64().unwrap() as u32).to_le_bytes());
    }
}

/// Explicitly configured grammar for synthetic runtime fixtures with arbitrary
/// registries. This is not evidence of public Film admission for those layouts.
/// Source envelopes and source bytes still come from Film::parse unchanged.
pub(crate) fn parse_configured_fixture(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
    config: &crate::theater::parser::v41::context::V41DecodeConfig,
) -> Result<ParsedFilm, crate::theater::film::ParseError> {
    use crate::theater::parser::packet::PacketRead;
    use crate::theater::parser::v41::chunks::replication::{
        replication_stream::{DecodeContext, decode},
        state::ReplicationDecodeState,
    };
    let mut film = parse_fixture_structural(chunks)?;
    assert!(
        film.registry.body.bootstrap_header.is_none(),
        "use public parsing for recognized bootstrap layouts"
    );
    assert!(
        config.corruption_check().is_some(),
        "fixture must declare its guard policy"
    );
    let mut state = ReplicationDecodeState::default();
    let mut context = DecodeContext {
        config,
        registry: &film.registry.body.registry,
        state: &mut state,
        event_gate15: None,
    };
    for chunk in &mut film.replication_stream.0 {
        context.state.current_chunk = chunk.index;
        for packet in &mut chunk.body.packets {
            packet.body = match decode(
                packet.header.packet_type,
                packet.payload(&chunk.data).unwrap(),
                &mut context,
            ) {
                Ok(body) => PacketRead::Decoded(body),
                Err(reason) => PacketRead::Opaque { reason },
            };
        }
    }
    Ok(film)
}

/// Append physical vacant slots before packing a synthetic native keyframe table.
pub(crate) fn vacant_keyframe_slots(bits: &mut Vec<bool>, count: usize) {
    for _ in 0..count {
        bits.extend(std::iter::repeat_n(true, 96));
        bits.extend(std::iter::repeat_n(false, 12));
    }
}

pub(crate) fn test_chunks(source: &FixtureFilmSource) -> Vec<RawFilmChunk> {
    let mut chunks: Vec<_> = source
        .original_chunks()
        .iter()
        .enumerate()
        .map(|(i, bytes)| {
            let metadata = source.metadata().get(i);
            RawFilmChunk {
                kind: RawChunkKind(
                    i32::try_from(metadata.map(|m| m.chunk_type).unwrap_or(if i == 0 {
                        1
                    } else {
                        2
                    }))
                    .expect("fixture chunk type fits i32"),
                ),
                index: i32::try_from(metadata.map_or(i, |m| m.index))
                    .expect("fixture chunk index fits i32"),
                start_ms: metadata.map_or(0, |m| m.start_ms),
                data: bytes.clone(),
            }
        })
        .collect();
    // Legacy test fixtures often contain only the section under test.
    if !chunks
        .iter()
        .any(|chunk| chunk.kind == ChunkKind::Summary.into())
    {
        let mut summary = empty_summary();
        summary.index = chunks.iter().map(|chunk| chunk.index).max().unwrap_or(0) + 1;
        chunks.push(summary);
    }
    fixture_chunks(chunks)
}

pub(crate) fn recharge_device_baseline()
-> crate::theater::parser::v41::chunks::replication::source::KeyframeRecord {
    use crate::theater::parser::v41::{
        chunks::replication::replication_stream::keyframes::record::decode_reference_keyframe_record_contextual,
        context::V41DecodeConfig,
    };
    let declarations: Vec<(String, u32)> = serde_json::from_str(include_str!(
        "../fixtures/movement-schema-v41/recharge-device-registry.json"
    ))
    .unwrap();
    let registry = component_registry(&[(43, declarations)]);
    let mut config = V41DecodeConfig::default();
    config.profile.grammar.baseline_scope = true;
    config.profile.grammar.writer_absolute = true;
    let payload = include_bytes!("../fixtures/movement-schema-v41/recharge-device-keyframe.bin");
    decode_reference_keyframe_record_contextual(
        payload,
        0,
        &registry,
        &config.record_layout(),
        Some(&config.profile),
    )
    .unwrap()
}

pub(crate) fn component_registry(
    declarations: &[(usize, Vec<(String, u32)>)],
) -> crate::theater::parser::v41::chunks::registry::source::FilmRegistry {
    use crate::theater::parser::packet::ByteRange;
    use crate::theater::parser::v41::chunks::registry::source::{
        FilmArchetype, FilmRegistry, RegistryComponent,
    };
    let range = ByteRange { start: 0, end: 0 };
    FilmRegistry {
        archetypes: (0..=declarations.iter().map(|(index, _)| *index).max().unwrap())
            .map(|index| FilmArchetype {
                index,
                source: range,
                padding: range,
                components: declarations.iter().find(|(i, _)| *i == index).map_or_else(
                    Vec::new,
                    |(_, components)| {
                        components
                            .iter()
                            .map(|(name, level)| RegistryComponent {
                                name_bytes: name.as_bytes().to_vec(),
                                precision_level: *level,
                                source: range,
                            })
                            .collect()
                    },
                ),
            })
            .collect(),
    }
}
