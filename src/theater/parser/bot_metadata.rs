//! Native BOT_METADATA payload scanning and cross-packet deduplication.
use super::bits::Bits;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmBotEntry {
    pub slot: u32,
    pub bot_id: u32,
    pub name: String,
    pub bit_position: usize,
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
pub(crate) fn read_native_bot_metadata(payload: &[u8]) -> NativeBotMetadataRead {
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
