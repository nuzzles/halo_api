//! Reference footer objective interactions and capture-burst event extraction.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectiveFooterEvent {
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
    pub slot: u8,
    pub team: u8,
    #[serde(rename = "XUID")]
    pub xuid: u64,
}
/// Reference v41 mode-footer fields from one complete 60-byte block. Slot and
/// team are raw byte values; no roster, team mapping or objective inference.
pub(crate) fn decode_objective_footer_block(
    block: &[u8; 60],
    xuid: u64,
) -> Option<ObjectiveFooterEvent> {
    (block[47] == 10).then(|| ObjectiveFooterEvent {
        time_ms: u32::from_be_bytes(block[48..52].try_into().unwrap()).into(),
        slot: block[36],
        team: block[37],
        xuid,
    })
}
