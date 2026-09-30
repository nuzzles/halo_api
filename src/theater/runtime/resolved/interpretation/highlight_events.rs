//! v41 reference highlight scanning, independent of the source-attributed summary decoder.
use super::bits::Bits;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HighlightEvent {
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

/// One tail-marker candidate considered by the reference scanner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightTailRead {
    pub marker_bit: usize,
    /// None when fewer than 480 bits separate this marker from the identity.
    pub data_start_bit: Option<usize>,
    /// All 60 bytes, including uninterpreted bytes and raw UTF16 string tails.
    pub data: Option<Vec<u8>>,
    pub event: Option<HighlightEvent>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightIdentityRead {
    /// The reference objective reader only accepts the FIRST tail marker. Raw
    /// slot/team bytes are exposed without player resolution or action inference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective_fields: Option<super::ObjectiveFooterEvent>,
    pub identity_bit: usize,
    pub xuid: u64,
    pub prefix: u8,
    pub window_end_bit: usize,
    /// Stops at the first accepted layout. Rejected tails remain in scan order.
    pub tails: Vec<HighlightTailRead>,
}
/// Reference highlight scanning on an already decompressed v41 source chunk.
/// Identity/tail matches are heuristic and do not partition the source chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightScan {
    pub source_bits: usize,
    pub identities: Vec<HighlightIdentityRead>,
    pub events: Vec<HighlightEvent>,
}
pub(crate) fn read_v41_highlights(data: &[u8]) -> HighlightScan {
    let mut identities = Vec::new();
    let events = scan_highlights(data, Some(&mut identities));
    HighlightScan {
        source_bits: data.len() * 8,
        identities,
        events,
    }
}

fn scan_highlights(
    data: &[u8],
    mut trace: Option<&mut Vec<HighlightIdentityRead>>,
) -> Vec<HighlightEvent> {
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
        let mut identity = trace.as_ref().map(|_| HighlightIdentityRead {
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
                    identity.tails.push(HighlightTailRead {
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
                    identity.objective_fields =
                        super::objective_extract::decode_objective_footer_block(&block, xuid);
                }
                identity.tails.push(HighlightTailRead {
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

fn decode_event(b: &[u8; 60], xuid: u64) -> Option<HighlightEvent> {
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
    Some(HighlightEvent {
        xuid,
        gamertag: String::from_utf16_lossy(&name),
        event_type: event_type.into(),
        type_hint: hint,
        is_medal,
        time_ms: u32::from_be_bytes(b[48..52].try_into().unwrap()),
        medal_type: b[59],
    })
}
