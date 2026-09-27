//! Session population packets (type 8), LevelUp roster_type8.go.
//! The body is shared with the bootstrap player table; XUID byte order differs.
use super::{
    FilmPacket, PlayerTableSlot,
    bits::Cursor,
    player_table::{SlotReader, slot_body},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterEntry {
    pub player: PlayerTableSlot,
    pub identity: u64,
    /// -1 when the option gate is open or absent in the supplied format.
    pub option: i32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterReport {
    pub announced: u32,
    pub entries: usize,
    pub refused: usize,
    pub bits_read: usize,
    pub bits_available: usize,
    pub remaining: usize,
    /// Native overflow verdict; a rejected list count is not necessarily overflow.
    pub overflow: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterUpdate {
    pub entries: Vec<RosterEntry>,
    pub report: RosterReport,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmRosterUpdate {
    pub source: FilmPacket,
    pub roster: RosterUpdate,
}
/// Decode a population payload with explicit format and build personalization width.
/// Completed entries preceding a refusal are retained, matching the reference.
pub fn decode_roster_update(
    payload: &[u8],
    format_version: u32,
    personalization_bits: usize,
) -> RosterUpdate {
    decode_roster_with_trace(payload, format_version, personalization_bits, false).roster
}
/// Native roster fields include reads in the first rejected entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeRosterRead {
    pub roster: RosterUpdate,
    pub fields: Vec<super::NativeSlotField>,
    /// Index into fields for each attempted entry, including a rejected entry.
    pub entry_field_starts: Vec<usize>,
}
pub fn read_native_roster_update(
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

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    pub(in crate::theater) fn normalize(v: Value) -> Value {
        match v {
            Value::Object(o) => Value::Object(
                o.into_iter()
                    .map(|(k, v)| {
                        let k = k.replace('_', "").to_ascii_lowercase();
                        let k = match k.as_str() {
                            "joueur" => "player",
                            "identite" => "identity",
                            "annonce" => "announced",
                            "entrees" => "entries",
                            "refusees" => "refused",
                            "bitslus" => "bitsread",
                            "bitsdisponibles" => "bitsavailable",
                            "reste" => "remaining",
                            "debordement" => "overflow",
                            s => s,
                        }
                        .to_string();
                        (k, normalize(v))
                    })
                    .collect(),
            ),
            Value::Array(a) => Value::Array(a.into_iter().map(normalize).collect()),
            v => v,
        }
    }
    #[test]
    fn native_roster_values_and_stopping_positions() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/roster-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                .collect();
            let actual = decode_roster_update(
                &bytes,
                row["format"].as_u64().unwrap() as u32,
                row["perso"].as_u64().unwrap() as usize,
            );
            assert_eq!(
                normalize(serde_json::to_value(&actual.report).unwrap()),
                normalize(row["report"].clone()),
                "report {i}"
            );
            assert_eq!(
                normalize(serde_json::to_value(&actual.entries).unwrap()),
                normalize(if row["entries"].is_null() {
                    json!([])
                } else {
                    row["entries"].clone()
                }),
                "entries {i}"
            );
            let native = read_native_roster_update(
                &bytes,
                row["format"].as_u64().unwrap() as u32,
                row["perso"].as_u64().unwrap() as usize,
            );
            assert_eq!(native.roster, actual, "trace changed decode {i}");
            assert_eq!(
                native.entry_field_starts.len(),
                actual.report.entries + actual.report.refused
            );
            let mut end = 0;
            for field in &native.fields {
                assert_eq!(field.bit, end, "noncontiguous field {i}");
                match field.value {
                    super::super::NativeSlotValue::Scalar(value) => {
                        // Independent MSB-first extraction from the source.
                        let raw = (field.bit..field.bit + field.bits).fold(0u64, |v, bit| {
                            (v << 1) | u64::from((bytes[bit / 8] >> (7 - bit % 8)) & 1)
                        });
                        assert_eq!(value, raw, "raw field {i}: {}", field.name);
                        end += field.bits;
                    }
                    super::super::NativeSlotValue::Opaque => {
                        end += field.bits;
                        assert!(end <= bytes.len() * 8);
                    }
                    super::super::NativeSlotValue::Unavailable => {
                        assert!(field.bit + field.bits > bytes.len() * 8);
                        assert_eq!(Some(field), native.fields.last());
                    }
                }
            }
            assert_eq!(end, actual.report.bits_read);
            let native_json = serde_json::to_vec(&native).unwrap();
            assert_eq!(native, serde_json::from_slice(&native_json).unwrap());
            let encoded = serde_json::to_vec(&actual).unwrap();
            assert_eq!(actual, serde_json::from_slice(&encoded).unwrap());
        }
    }
}
