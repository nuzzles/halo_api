//! Native BOT_METADATA payload scanning and cross-packet deduplication.
use super::{IdentityBot, bits::Bits, native_sort};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmBotEntry {
    pub slot: u32,
    pub bot_id: u32,
    pub name: String,
    pub bit_position: usize,
}
impl From<&FilmBotEntry> for IdentityBot {
    fn from(b: &FilmBotEntry) -> Self {
        Self {
            film_index: b.slot.into(),
            bot_id: b.bot_id.into(),
            name: b.name.clone().into(),
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmBotMetadata {
    pub max_declared_bots: u32,
    pub packets: usize,
    pub bots: Vec<FilmBotEntry>,
}
fn read_name(payload: &[u8], bit: usize) -> Option<(String, usize)> {
    let bits = Bits(payload);
    let mut name = Vec::with_capacity(48);
    let mut p = bit;
    while name.len() < 48 {
        let Some(c) = bits.read(p, 16).filter(|c| (0x20..=0x7e).contains(c)) else {
            break;
        };
        name.push(c as u8);
        p += 16;
    }
    if name.len() < 4 || bits.read(p, 16) != Some(0) {
        return None;
    }
    Some((String::from_utf8(name).unwrap(), p))
}
/// A candidate located by printable-name scanning, not a sequential record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeBotCandidate {
    pub entry: FilmBotEntry,
    pub slot_bit: usize,
    pub bot_id_bit: usize,
    /// Exclusive end of the required UTF16BE terminator.
    pub name_end_bit: usize,
    pub name_units: Vec<u16>,
    /// The pinned scanner selects only the first occurrence of each name.
    /// Repeated-name candidates remain available here, without resolved identity.
    pub selected_by_reference: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeBotMetadataRead {
    pub source_bits: usize,
    /// Big-endian count at source bits 0..32; None means it was unavailable.
    pub declared_bots: Option<u32>,
    /// None means missing count or count >64 refused the reference scan.
    /// Some(empty) means scanned with no admitted name/reference candidates.
    pub candidates: Option<Vec<NativeBotCandidate>>,
}
/// Read a type-12 payload without cross-packet deduplication or player resolution.
/// Zero count still permits the reference scan. Candidate positions do not
/// partition the packet: the reference explicitly lacks a multi-entry stride.
pub fn read_native_bot_metadata(payload: &[u8]) -> NativeBotMetadataRead {
    let declared_bots = payload
        .get(..4)
        .map(|b| u32::from_be_bytes(b.try_into().unwrap()));
    let candidates = declared_bots.filter(|&n| n <= 64).map(|_| {
        let mut candidates = Vec::new();
        let mut names = BTreeSet::new();
        visit_bot_candidates(payload, |entry, terminator| {
            candidates.push(NativeBotCandidate {
                slot_bit: entry.bit_position - 0x74 * 8,
                bot_id_bit: entry.bit_position - 0x70 * 8,
                name_end_bit: terminator + 16,
                name_units: entry.name.bytes().map(u16::from).chain([0]).collect(),
                selected_by_reference: names.insert(entry.name.clone()),
                entry,
            });
        });
        candidates
    });
    NativeBotMetadataRead {
        source_bits: payload.len() * 8,
        declared_bots,
        candidates,
    }
}
/// Scan every bit, with no fixed entry stride. Only the first occurrence of a
/// name survives, preventing the shifted second copy from claiming slot zero.
pub fn scan_bot_metadata_entries(payload: &[u8]) -> Vec<FilmBotEntry> {
    let mut first = BTreeMap::<String, FilmBotEntry>::new();
    visit_bot_candidates(payload, |entry, _| {
        first.entry(entry.name.clone()).or_insert(entry);
    });
    let mut out: Vec<_> = first.into_values().collect();
    out.sort_unstable_by_key(|b| b.bit_position);
    out
}
fn visit_bot_candidates(payload: &[u8], mut visit: impl FnMut(FilmBotEntry, usize)) {
    let bits = Bits(payload);
    let mut at = 0;
    while at + 16 <= bits.len() {
        if let Some((name, next)) = read_name(payload, at) {
            let slot = at.checked_sub(0x74 * 8).and_then(|p| bits.read(p, 32));
            let id = at.checked_sub(0x70 * 8).and_then(|p| bits.read(p, 32));
            if let (Some(slot), Some(id)) = (slot.filter(|&v| v < 64), id.filter(|&v| v < 4096)) {
                visit(
                    FilmBotEntry {
                        slot: slot as u32,
                        bot_id: id as u32,
                        name,
                        bit_position: at,
                    },
                    next,
                );
            }
            at = next;
        }
        at += 1;
    }
}
/// Consume packets in native source order. Invalid declared counts still count
/// as packets but contribute neither declarations nor entries. Zero count does
/// not suppress the native payload scan.
pub fn collect_bot_metadata<'a>(
    packets: impl IntoIterator<Item = (u16, &'a [u8])>,
) -> FilmBotMetadata {
    let mut out = FilmBotMetadata::default();
    let mut seen = BTreeSet::new();
    for (kind, payload) in packets {
        if kind != 12 || payload.len() < 4 {
            continue;
        }
        out.packets += 1;
        let count = u32::from_be_bytes(payload[..4].try_into().unwrap());
        if count > 64 {
            continue;
        }
        out.max_declared_bots = out.max_declared_bots.max(count);
        for bot in scan_bot_metadata_entries(payload) {
            if seen.insert((bot.slot, bot.bot_id)) {
                out.bots.push(bot);
            }
        }
    }
    native_sort::sort_by(&mut out.bots, |a, b| a.slot.cmp(&b.slot));
    out
}

/// Native killsource consumes all supplied chunks in source order, including
/// noncontiguous metadata indices. The shared packet reader stops at CHUNK_END.
pub fn scan_film_bot_metadata(
    chunks: &[crate::clients::hi::models::FilmChunkData],
) -> FilmBotMetadata {
    collect_bot_metadata(chunks.iter().flat_map(|chunk| {
        super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .map(move |p| {
                (
                    p.packet_type,
                    &chunk.data[p.payload_offset..p.payload_offset + p.payload_size],
                )
            })
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        packets: Vec<Packet>,
        output: FilmBotMetadata,
    }
    #[derive(Deserialize)]
    struct Packet {
        kind: u16,
        hex: String,
        entries: Vec<FilmBotEntry>,
    }
    #[test]
    fn captured_bot_metadata_packets() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bot-corpus-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for row in rows {
            let packets: Vec<Vec<u8>> = row["packets"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| {
                    let hex = p.as_str().unwrap();
                    (0..hex.len())
                        .step_by(2)
                        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
                        .collect()
                })
                .collect();
            let expected: FilmBotMetadata = serde_json::from_value(row["output"].clone()).unwrap();
            assert_eq!(
                collect_bot_metadata(packets.iter().map(|p| (12, p.as_slice()))),
                expected,
                "{}",
                row["folder"]
            );
        }
    }
    #[test]
    fn native_bot_metadata_scanning() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bot-metadata-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        let mut repeated_names = 0;
        for (i, c) in rows.into_iter().enumerate() {
            let packets: Vec<_> = c
                .packets
                .into_iter()
                .enumerate()
                .map(|(j, p)| {
                    let data: Vec<_> = (0..p.hex.len())
                        .step_by(2)
                        .map(|at| u8::from_str_radix(&p.hex[at..at + 2], 16).unwrap())
                        .collect();
                    assert_eq!(
                        scan_bot_metadata_entries(&data),
                        p.entries,
                        "entries {i}/{j}"
                    );
                    let native = read_native_bot_metadata(&data);
                    let count = data
                        .get(..4)
                        .map(|b| u32::from_be_bytes(b.try_into().unwrap()));
                    assert_eq!(native.declared_bots, count);
                    assert_eq!(native.candidates.is_some(), count.is_some_and(|n| n <= 64));
                    if let Some(candidates) = &native.candidates {
                        assert_eq!(
                            candidates
                                .iter()
                                .filter(|c| c.selected_by_reference)
                                .map(|c| c.entry.clone())
                                .collect::<Vec<_>>(),
                            p.entries
                        );
                        let raw = |start: usize, width: usize| {
                            (start..start + width).fold(0u64, |v, bit| {
                                (v << 1) | u64::from((data[bit / 8] >> (7 - bit % 8)) & 1)
                            })
                        };
                        for candidate in candidates {
                            repeated_names += usize::from(!candidate.selected_by_reference);
                            assert_eq!(
                                raw(candidate.slot_bit, 32),
                                u64::from(candidate.entry.slot)
                            );
                            assert_eq!(
                                raw(candidate.bot_id_bit, 32),
                                u64::from(candidate.entry.bot_id)
                            );
                            assert_eq!(
                                candidate.name_end_bit,
                                candidate.entry.bit_position + candidate.name_units.len() * 16
                            );
                            for (unit, value) in candidate.name_units.iter().enumerate() {
                                assert_eq!(
                                    raw(candidate.entry.bit_position + unit * 16, 16),
                                    u64::from(*value)
                                );
                            }
                            assert_eq!(candidate.name_units.last(), Some(&0));
                        }
                        assert!(
                            candidates
                                .windows(2)
                                .all(|w| w[0].entry.bit_position < w[1].entry.bit_position)
                        );
                    }
                    assert_eq!(
                        native,
                        serde_json::from_slice(&serde_json::to_vec(&native).unwrap()).unwrap()
                    );
                    (p.kind, data)
                })
                .collect();
            let chunks: Vec<_> = packets
                .iter()
                .enumerate()
                .map(|(j, (kind, payload))| {
                    let mut data = vec![0_u8; 16];
                    data[..2].copy_from_slice(&kind.to_le_bytes());
                    data[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
                    data.extend_from_slice(payload);
                    let mut end = [0_u8; 16];
                    end[..2].copy_from_slice(&7_u16.to_le_bytes());
                    data.extend_from_slice(&end);
                    let mut ignored = [0_u8; 16];
                    ignored[..2].copy_from_slice(&12_u16.to_le_bytes());
                    ignored[4..8].copy_from_slice(&4_u32.to_le_bytes());
                    data.extend_from_slice(&ignored);
                    data.extend_from_slice(&64_u32.to_be_bytes());
                    crate::clients::hi::models::FilmChunkData {
                        metadata: crate::clients::hi::models::FilmChunk {
                            index: (j * 3) as i32,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                            chunk_type: 0,
                        },
                        data,
                    }
                })
                .collect();
            assert_eq!(
                scan_film_bot_metadata(&chunks),
                c.output,
                "chunk integration {i}"
            );
            assert_eq!(
                collect_bot_metadata(packets.iter().map(|(kind, data)| (*kind, data.as_slice()))),
                c.output,
                "metadata {i}"
            );
        }
        assert!(repeated_names > 0);
    }
}
