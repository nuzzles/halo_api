//! v41 native highlight scanning, independent of the source-attributed summary decoder.
use super::{DecodeError, IdentityDeath, bits::Bits, native_sort};
use crate::clients::hi::models::FilmChunkData;
use flate2::{Decompress, FlushDecompress, Status};
use serde::{Deserialize, Serialize};

/// Native highlight layout metadata. `major_version: None` means the header
/// was unreadable and the layout is only the native historical fallback.
/// It is not evidence that the recording is v41 or safe to decode as v41.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmHighlightProfile {
    pub major_version: Option<u32>,
    pub implantation: String,
    pub gamertag_offset_bytes: usize,
}

/// Inspect a decompressed bootstrap header, retaining missing-version metadata.
/// Readable versions other than 41 are rejected; this does not enable decoding
/// older films. Explicit caller configuration must not be passed as a header.
pub fn v41_highlight_profile_from_header(data: &[u8]) -> Result<FilmHighlightProfile, DecodeError> {
    let major_version = super::film_major_version_from_header(data);
    if let Some(major) = major_version.filter(|&major| major != 41) {
        return Err(DecodeError::UnsupportedVersion(major as i32));
    }
    Ok(FilmHighlightProfile {
        major_version,
        implantation: "gamertag_en_tete".into(),
        gamertag_offset_bytes: 0,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NativeHighlightEvent {
    #[serde(rename = "XUID")]
    pub xuid: u64,
    pub gamertag: String,
    pub event_type: String,
    pub type_hint: u8,
    pub is_medal: bool,
    #[serde(rename = "TimeMS")]
    pub time_ms: u32,
    pub medal_type: u8,
}

impl NativeHighlightEvent {
    /// Native identity lookup; a non-medal never acquires a medal name.
    pub fn medal_name(&self) -> Option<&'static str> {
        (self.event_type == "medal" && self.is_medal)
            .then(|| super::native_medal_name(self.type_hint.into(), self.medal_type.into()))
            .flatten()
    }
}

/// One tail-marker candidate considered by the reference scanner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeHighlightTailRead {
    pub marker_bit: usize,
    /// None when fewer than 480 bits separate this marker from the identity.
    pub data_start_bit: Option<usize>,
    /// All 60 bytes, including uninterpreted bytes and raw UTF16 string tails.
    pub data: Option<Vec<u8>>,
    pub event: Option<NativeHighlightEvent>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeHighlightIdentityRead {
    /// The native objective reader only accepts the FIRST tail marker. Raw
    /// slot/team bytes are exposed without player resolution or action inference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective_fields: Option<super::ObjectiveFooterEvent>,
    pub identity_bit: usize,
    pub xuid: u64,
    pub prefix: u8,
    pub window_end_bit: usize,
    /// Stops at the first accepted layout. Rejected tails remain in scan order.
    pub tails: Vec<NativeHighlightTailRead>,
}
/// Reference highlight scanning on an already decompressed v41 source chunk.
/// Identity/tail matches are heuristic and do not partition the source chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeHighlightScan {
    pub source_bits: usize,
    pub identities: Vec<NativeHighlightIdentityRead>,
    pub events: Vec<NativeHighlightEvent>,
}
pub fn read_native_v41_highlights(data: &[u8]) -> NativeHighlightScan {
    let mut identities = Vec::new();
    let events = scan_highlights(data, Some(&mut identities));
    NativeHighlightScan {
        source_bits: data.len() * 8,
        identities,
        events,
    }
}

/// Accept plaintext or zlib, but reject damage after a valid zlib header.
/// Unknown event layouts are skipped. Duplicate values at distinct bit positions
/// are retained in scan order; timestamps do not determine this order.
pub fn parse_highlight_events(
    data: &[u8],
    major_version: i32,
) -> Result<Vec<NativeHighlightEvent>, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let inflated = inflate_highlights(data)?;
    Ok(scan_highlights(inflated.as_deref().unwrap_or(data), None))
}
fn scan_highlights(
    data: &[u8],
    mut trace: Option<&mut Vec<NativeHighlightIdentityRead>>,
) -> Vec<NativeHighlightEvent> {
    let bits = Bits(data);
    let mut out = Vec::new();
    if bits.len() < 80 {
        return out;
    }
    for marker in 72..=bits.len() - 8 {
        if bits.read(marker, 8) != Some(0xc0)
            || !matches!(bits.read(marker - 8, 8), Some(0x2d | 0x25))
        {
            continue;
        }
        let start = marker - 72;
        let xuid = bits.read(start, 64).unwrap().swap_bytes();
        if xuid <= 2_000_000_000_000_000 || xuid >= 3_000_000_000_000_000 {
            continue;
        }
        let end = start.saturating_add(20_000).min(bits.len());
        let mut identity = trace.as_ref().map(|_| NativeHighlightIdentityRead {
            objective_fields: None,
            identity_bit: start,
            xuid,
            prefix: bits.read(start + 64, 8).unwrap() as u8,
            window_end_bit: end,
            tails: Vec::new(),
        });
        let first_tail = if identity.is_some() {
            start
        } else {
            start + 480
        };
        for tail in first_tail..=end.saturating_sub(32) {
            if bits.read(tail, 32) != Some(0x00002ee0) {
                continue;
            }
            if tail < start + 480 {
                if let Some(identity) = &mut identity {
                    identity.tails.push(NativeHighlightTailRead {
                        marker_bit: tail,
                        data_start_bit: None,
                        data: None,
                        event: None,
                    });
                }
                continue;
            }
            let block: [u8; 60] =
                std::array::from_fn(|i| bits.read(tail - 480 + i * 8, 8).unwrap() as u8);
            let event = decode_event(&block, xuid);
            if let Some(identity) = &mut identity {
                if identity.tails.is_empty() {
                    identity.objective_fields = super::decode_objective_footer_block(&block, xuid);
                }
                identity.tails.push(NativeHighlightTailRead {
                    marker_bit: tail,
                    data_start_bit: Some(tail - 480),
                    data: Some(block.to_vec()),
                    event: event.clone(),
                });
            }
            if let Some(event) = event {
                out.push(event);
                break;
            }
        }
        if let (Some(trace), Some(identity)) = (&mut trace, identity) {
            trace.push(identity);
        }
    }
    out
}

fn decode_event(b: &[u8; 60], xuid: u64) -> Option<NativeHighlightEvent> {
    let hint = b[47];
    let is_medal = b[55] == 1;
    let event_type = if is_medal
        && matches!(
            hint,
            50 | 51
                | 52
                | 100
                | 101
                | 150
                | 200
                | 205
                | 210
                | 220
                | 225
                | 230
                | 235
                | 240
                | 245
                | 250
        ) {
        "medal"
    } else {
        match hint {
            10 => "mode",
            20 => "death",
            50 => "kill",
            _ => return None,
        }
    };
    let name: Vec<u16> = b[..32]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&c| c != 0)
        .collect();
    Some(NativeHighlightEvent {
        xuid,
        gamertag: String::from_utf16_lossy(&name),
        event_type: event_type.into(),
        type_hint: hint,
        is_medal,
        time_ms: u32::from_be_bytes(b[48..52].try_into().unwrap()),
        medal_type: b[59],
    })
}

fn inflate_highlights(data: &[u8]) -> Result<Option<Vec<u8>>, DecodeError> {
    // Go's NewReader fails during header setup for malformed headers, incomplete
    // dictionary IDs and a required nonempty dictionary. Those inputs are plaintext.
    if data.len() < 2
        || data[0] & 15 != 8
        || data[0] >> 4 > 7
        || !u16::from_be_bytes([data[0], data[1]]).is_multiple_of(31)
    {
        return Ok(None);
    }
    let normalized;
    let input = if data[1] & 32 != 0 {
        if data.get(2..6) != Some(&[0, 0, 0, 1]) {
            return Ok(None);
        }
        // Go accepts the Adler32 of an empty dictionary with its nil dictionary.
        normalized = [&[0x78, 0x9c][..], &data[6..]].concat();
        normalized.as_slice()
    } else {
        data
    };
    let mut decoder = Decompress::new(true);
    let mut out = Vec::new();
    loop {
        let mut buffer = [0u8; 8192];
        let before_in = decoder.total_in();
        let before_out = decoder.total_out();
        let status = decoder
            .decompress(
                &input[before_in as usize..],
                &mut buffer,
                FlushDecompress::None,
            )
            .map_err(|e| DecodeError::Inconsistent(format!("highlight zlib stream: {e}")))?;
        let written = (decoder.total_out() - before_out) as usize;
        out.extend_from_slice(&buffer[..written]);
        if status == Status::StreamEnd {
            return Ok(Some(out));
        }
        if decoder.total_in() == before_in && written == 0 {
            return Err(DecodeError::Inconsistent(
                "truncated highlight zlib stream".into(),
            ));
        }
    }
}

/// Native highlights from the final contiguous readable manifest chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmHighlightStream {
    /// Header-derived metadata, unavailable for explicit-version scans and old exports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<FilmHighlightProfile>,
    pub chunk_index: i32,
    pub events: Vec<NativeHighlightEvent>,
    /// Native medal identities, indexed exactly like events. None means non-medal
    /// or an unrecognized pair; duplicate events retain separate entries.
    #[serde(default)]
    pub medal_names: Vec<Option<String>>,
}
impl FilmHighlightStream {
    pub fn deaths(&self) -> Result<Vec<IdentityDeath>, DecodeError> {
        highlight_deaths(&self.events)
    }
}
/// Select the last contiguous readable chunk starting at index one, in manifest
/// order, as LevelUp does. Chunk type is immaterial.
pub fn scan_film_highlights(
    chunks: &[FilmChunkData],
    major_version: i32,
) -> Result<FilmHighlightStream, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    let mut want = 1;
    let mut last = None;
    for chunk in chunks {
        if chunk.metadata.index < 1 {
            continue;
        }
        if chunk.metadata.index != want {
            break;
        }
        last = Some(chunk);
        want += 1;
    }
    let last = last.ok_or(DecodeError::Missing("readable film chunk"))?;
    let events = parse_highlight_events(&last.data, major_version)?;
    let medal_names = events
        .iter()
        .map(|event| event.medal_name().map(str::to_owned))
        .collect();
    Ok(FilmHighlightStream {
        profile: None,
        chunk_index: last.metadata.index,
        events,
        medal_names,
    })
}
/// Read the native death feed. No recorded deaths is an explicit refusal.
pub fn scan_film_deaths(
    chunks: &[FilmChunkData],
    major_version: i32,
) -> Result<Vec<IdentityDeath>, DecodeError> {
    scan_film_highlights(chunks, major_version)?.deaths()
}

/// Extract and natively sort the death feed by time, including equal-time ties.
pub fn highlight_deaths(
    events: &[NativeHighlightEvent],
) -> Result<Vec<IdentityDeath>, DecodeError> {
    let mut deaths: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == "death")
        .map(|e| IdentityDeath {
            xuid: e.xuid,
            gamertag: e.gamertag.clone().into(),
            time_ms: i64::from(e.time_ms),
        })
        .collect();
    if deaths.is_empty() {
        return Err(DecodeError::Missing("highlight deaths"));
    }
    native_sort::sort_by(&mut deaths, |a, b| a.time_ms.cmp(&b.time_ms));
    Ok(deaths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_highlight_oracle() {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            events: Option<Vec<NativeHighlightEvent>>,
            error: bool,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/highlights-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.iter().enumerate() {
            let data: Vec<u8> = c
                .hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
                .collect();
            let result = parse_highlight_events(&data, 41);
            assert_eq!(result.is_err(), c.error, "case {i}");
            if let Ok(events) = result {
                assert_eq!(events, c.events.clone().unwrap_or_default(), "case {i}");
                let inflated = inflate_highlights(&data).unwrap();
                let payload = inflated.as_deref().unwrap_or(&data);
                let scan = read_native_v41_highlights(payload);
                assert_eq!(scan.events, events);
                let raw = |start: usize, width: usize| {
                    (start..start + width).fold(0u64, |v, bit| {
                        (v << 1) | u64::from((payload[bit / 8] >> (7 - bit % 8)) & 1)
                    })
                };
                for identity in &scan.identities {
                    assert_eq!(raw(identity.identity_bit, 64).swap_bytes(), identity.xuid);
                    assert_eq!(
                        raw(identity.identity_bit + 64, 8),
                        u64::from(identity.prefix)
                    );
                    for tail in &identity.tails {
                        assert_eq!(raw(tail.marker_bit, 32), 0x2ee0);
                        assert!(tail.marker_bit + 32 <= identity.window_end_bit);
                        if let (Some(start), Some(bytes)) = (tail.data_start_bit, &tail.data) {
                            assert_eq!(bytes.len(), 60);
                            assert_eq!(start + 480, tail.marker_bit);
                            for (i, byte) in bytes.iter().enumerate() {
                                assert_eq!(raw(start + i * 8, 8), u64::from(*byte));
                            }
                        } else {
                            assert!(tail.marker_bit < identity.identity_bit + 480);
                            assert!(tail.event.is_none());
                        }
                    }
                }
                assert_eq!(
                    scan,
                    serde_json::from_slice(&serde_json::to_vec(&scan).unwrap()).unwrap()
                );
            }
        }
    }
    #[test]
    fn native_highlight_objective_fields_match_reference() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/objective-extract-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for (index, case) in cases.iter().enumerate() {
            let hex = case["data"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let scan = read_native_v41_highlights(&bytes);
            let mut fields: Vec<_> = scan
                .identities
                .iter()
                .filter_map(|i| i.objective_fields.clone())
                .collect();
            // Sort only this test projection to compare the reference's published
            // footer ordering. The native trace itself retains source order.
            native_sort::sort_by(&mut fields, |a, b| a.time_ms.cmp(&b.time_ms));
            assert_eq!(
                serde_json::to_value(fields).unwrap(),
                case["footer"],
                "case {index}"
            );
            for identity in &scan.identities {
                if let Some(fields) = &identity.objective_fields {
                    let data = identity.tails[0].data.as_ref().unwrap();
                    assert_eq!(fields.slot, data[36]);
                    assert_eq!(fields.team, data[37]);
                    assert_eq!(fields.xuid, identity.xuid);
                }
            }
            assert_eq!(
                scan,
                serde_json::from_slice(&serde_json::to_vec(&scan).unwrap()).unwrap()
            );
        }
    }

    #[test]
    fn native_highlight_scan_retains_rejected_and_missing_tails() {
        let mut data = vec![0; 200];
        data[..8].copy_from_slice(&2_500_000_000_000_001u64.to_le_bytes());
        data[8..10].copy_from_slice(&[0x2d, 0xc0]);
        let no_tail = read_native_v41_highlights(&data);
        assert_eq!(no_tail.identities.len(), 1);
        assert!(no_tail.identities[0].tails.is_empty());
        for at in [10, 80, 160] {
            data[at..at + 4].copy_from_slice(&[0, 0, 0x2e, 0xe0]);
        }
        data[147] = 50;
        data[148..152].copy_from_slice(&1000u32.to_be_bytes());
        data[132] = 0xf7; // Uninterpreted byte in accepted block.
        let scan = read_native_v41_highlights(&data);
        assert_eq!(scan.events.len(), 1);
        assert_eq!(scan.events[0].time_ms, 1000);
        let tails = &scan.identities[0].tails;
        assert!(tails.iter().any(|t| t.data.is_none()));
        assert!(tails.iter().any(|t| t.data.is_some() && t.event.is_none()));
        let accepted = tails.last().unwrap();
        assert!(accepted.event.is_some());
        assert_eq!(accepted.data.as_ref().unwrap()[32], 0xf7);
        assert_eq!(accepted.marker_bit, 160 * 8);
        data[147] = 10;
        data[136] = 254;
        data[137] = 255;
        let later_mode = read_native_v41_highlights(&data);
        assert_eq!(later_mode.events[0].event_type, "mode");
        assert!(later_mode.identities[0].objective_fields.is_none());
        data[10..14].fill(0);
        data[80..84].fill(0);
        let first_mode = read_native_v41_highlights(&data);
        let fields = first_mode.identities[0].objective_fields.as_ref().unwrap();
        assert_eq!((fields.slot, fields.team, fields.time_ms), (254, 255, 1000));
    }

    #[test]
    fn death_chunk_selection_stops_at_manifest_gap() {
        use crate::clients::hi::models::FilmChunk;
        fn chunk(index: i32, time: u32) -> FilmChunkData {
            let mut data = vec![0; 80];
            data[..8].copy_from_slice(&2_500_000_000_000_001u64.to_le_bytes());
            data[8..10].copy_from_slice(&[0x2d, 0xc0]);
            data[10 + 47] = 20;
            data[10 + 48..10 + 52].copy_from_slice(&time.to_be_bytes());
            data[70..74].copy_from_slice(&[0, 0, 0x2e, 0xe0]);
            FilmChunkData {
                data,
                metadata: FilmChunk {
                    index,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: 80,
                    file_relative_path: String::new(),
                    chunk_type: 1,
                },
            }
        }
        assert_eq!(
            scan_film_deaths(&[chunk(0, 0), chunk(1, 10), chunk(3, 30)], 41).unwrap()[0].time_ms,
            10
        );
        assert_eq!(
            scan_film_deaths(&[chunk(0, 0), chunk(1, 10), chunk(2, 20)], 41).unwrap()[0].time_ms,
            20
        );
        assert!(scan_film_deaths(&[chunk(2, 20)], 41).is_err());
        assert!(parse_highlight_events(&[], 40).is_err());
    }
}
