//! Session population packets (type 8), LevelUp roster_type8.go.
//! The body is shared with the bootstrap player table; XUID byte order differs.
use super::{
    PlayerTableSlot,
    bits::Cursor,
    player_slot::{SlotReader, slot_body},
};
pub(crate) use crate::theater::film::replication::roster_updates::{
    NativeRosterRead, RosterEntry, RosterReport, RosterUpdate,
};
pub(crate) fn read_native_roster_update(
    payload: &[u8],
    format_version: u32,
    personalization_bits: usize,
) -> NativeRosterRead {
    decode_roster_with_trace(payload, format_version, personalization_bits, true)
}
fn decode_roster_with_trace(
    payload: &[u8],
    format_version: u32,
    personalization_bits: usize,
    trace: bool,
) -> NativeRosterRead {
    let mut r = SlotReader::new(Cursor::new(payload, 0).expect("zero offset"), trace);
    let mut entry_field_starts = Vec::new();
    let announced = r.read("announced", 32).unwrap_or(0) as u32;
    let mut out = RosterUpdate {
        entries: Vec::new(),
        report: RosterReport {
            announced,
            bits_available: payload.len() * 8,
            ..Default::default()
        },
    };
    for _ in 0..announced {
        if r.truncated {
            break;
        }
        if let Some(fields) = &r.fields {
            entry_field_starts.push(fields.len());
        }
        let start = r.cursor.position;
        let entry = (|| {
            let option = if format_version >= 21 && r.read("option_gate", 1)? == 0 {
                r.read("option", 5)? as i32
            } else {
                -1
            };
            let identity = r.read("identity_wire", 64)?.swap_bytes();
            let (gamertag, shorts) = slot_body(&mut r, personalization_bits)?;
            Some(RosterEntry {
                identity,
                option,
                player: PlayerTableSlot {
                    film_index: 0,
                    xuid: identity,
                    gamertag,
                    session_token: 0,
                    bit: start,
                    total_bits: r.cursor.position - start,
                    shorts,
                },
            })
        })();
        let Some(entry) = entry else {
            out.report.refused += 1;
            break;
        };
        out.entries.push(entry);
    }
    out.report.entries = out.entries.len();
    out.report.bits_read = r.cursor.position;
    out.report.remaining = out.report.bits_available - r.cursor.position;
    out.report.overflow = r.truncated;
    NativeRosterRead {
        roster: out,
        fields: r.fields.unwrap_or_default(),
        entry_field_starts,
    }
}
