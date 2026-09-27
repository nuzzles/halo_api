//! Native v41 grenade creation signatures and their player indices.
use super::{
    DecodeError, FilmIdentity, FilmRegistry,
    bits::Bits,
    fire_events::{native_chunk_packets, native_chunk_prefix},
};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
pub const GRENADE_TYPE_IDS_BY_RANK: [u32; 4] = [0xb0171062, 0xc0e34c44, 0x3b2567d4, 0x9212e428];
pub fn grenade_rank_of(id: u32) -> Option<usize> {
    GRENADE_TYPE_IDS_BY_RANK.iter().position(|x| *x == id)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FilmGrenadeThrow {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub chunk: i64,
    pub packet_index: usize,
    pub bit_pos: usize,
    pub film_index: u8,
    #[serde(rename = "TypeID")]
    pub type_id: u32,
}
impl FilmGrenadeThrow {
    pub fn rank(&self) -> Option<usize> {
        grenade_rank_of(self.type_id)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrenadeGrammar {
    pub projectile_archetype: usize,
    pub resolved_by_name: bool,
    /// False records the reference-profile fallback for an absent/unknown build.
    pub known_build: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrenadeThrowStats {
    pub patterns: usize,
    pub other_archetypes: usize,
    pub rejected_known_ids: usize,
    pub indeterminate_archetype: usize,
    pub published: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrenadeThrowStream {
    pub grammar: GrenadeGrammar,
    pub records: Vec<FilmGrenadeThrow>,
    pub stats: GrenadeThrowStats,
}
pub fn v41_grenade_grammar(
    registry: Option<&FilmRegistry>,
    identity: Option<&FilmIdentity>,
) -> GrenadeGrammar {
    let mut g = GrenadeGrammar {
        projectile_archetype: 41,
        resolved_by_name: false,
        known_build: identity
            .is_some_and(|i| matches!(i.build.as_str(), "HI_1_12_0" | "HI_1_13_0")),
    };
    let names = [
        "projectile-at-rest-state",
        "projectile-tether-state",
        "projectile-command_tick",
        "projectile-deceleration-disabled-state",
    ];
    let mut best = 0;
    if let Some(reg) = registry {
        for a in &reg.archetypes {
            let count = names
                .iter()
                .filter(|n| a.components.iter().any(|c| c == *n))
                .count();
            if count > best {
                best = count;
                g.projectile_archetype = a.index;
                g.resolved_by_name = true;
            }
        }
    }
    g
}
/// Scan bounded 108-bit structures. The preceding sixth archetype bit must exist;
/// matching only the five marker bits would confuse projectiles with players.
pub fn decode_grenade_throws(payload: &[u8], grammar: GrenadeGrammar) -> GrenadeThrowStream {
    decode_grenade_throws_with_preamble(payload, grammar, super::GrenadePreamble::default())
}
pub(super) fn decode_grenade_throws_with_preamble(
    payload: &[u8],
    grammar: GrenadeGrammar,
    preamble: super::GrenadePreamble,
) -> GrenadeThrowStream {
    let mut out = GrenadeThrowStream {
        grammar,
        records: Vec::new(),
        stats: Default::default(),
    };
    let b = Bits(payload);
    let Some(limit) = b.len().checked_sub(preamble.author_bit + 5) else {
        return out;
    };
    let pattern = ((out.grammar.projectile_archetype as u64 & 31) << (preamble.bits - 5))
        | u64::from(preamble.default_state);
    for (p, word) in b.windows().take(limit + 1) {
        if word >> (64 - preamble.bits) != pattern {
            continue;
        }
        out.stats.patterns += 1;
        if p == 0 {
            out.stats.indeterminate_archetype += 1;
            continue;
        }
        let id = b.read(p + preamble.bits, 32).unwrap() as u32;
        if b.read(p - 1, 6) != Some(out.grammar.projectile_archetype as u64) {
            out.stats.other_archetypes += 1;
            if grenade_rank_of(id).is_some() {
                out.stats.rejected_known_ids += 1;
            }
            continue;
        }
        if grenade_rank_of(id).is_none() {
            continue;
        }
        out.records.push(FilmGrenadeThrow {
            timestamp_us: 0,
            chunk: 0,
            packet_index: 0,
            bit_pos: p,
            film_index: b.read(p + preamble.author_bit, 5).unwrap() as u8,
            type_id: id,
        });
    }
    out.stats.published = out.records.len();
    out
}
pub fn scan_grenade_throws(
    chunks: &[FilmChunkData],
    registry: Option<&FilmRegistry>,
    identity: Option<&FilmIdentity>,
) -> Result<GrenadeThrowStream, DecodeError> {
    let grammar = v41_grenade_grammar(registry, identity);
    let mut out = GrenadeThrowStream {
        grammar: grammar.clone(),
        records: Vec::new(),
        stats: Default::default(),
    };
    for c in native_chunk_prefix(chunks)? {
        for (index, p) in native_chunk_packets(c).into_iter().enumerate() {
            if p.packet_type != 0 {
                continue;
            }
            let scan = decode_grenade_throws(
                &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                grammar.clone(),
            );
            out.stats.patterns += scan.stats.patterns;
            out.stats.other_archetypes += scan.stats.other_archetypes;
            out.stats.rejected_known_ids += scan.stats.rejected_known_ids;
            out.stats.indeterminate_archetype += scan.stats.indeterminate_archetype;
            out.records.extend(scan.records.into_iter().map(|mut e| {
                e.timestamp_us = p.timestamp_us;
                e.chunk = i64::from(c.metadata.index);
                e.packet_index = index;
                e
            }));
        }
    }
    out.stats.published = out.records.len();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_grenade_signatures_and_rejections() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/grenade-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let pay: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                .collect();
            let g = serde_json::from_value(row["grammar"].clone()).unwrap();
            let actual = decode_grenade_throws(&pay, g);
            let mut expected: Vec<FilmGrenadeThrow> =
                serde_json::from_value(if row["events"].is_null() {
                    serde_json::json!([])
                } else {
                    row["events"].clone()
                })
                .unwrap();
            for e in &mut expected {
                e.chunk = 0;
                e.packet_index = 0;
                e.timestamp_us = 0;
            }
            assert_eq!(actual.records, expected, "records {i}");
            assert_eq!(
                serde_json::to_value(actual.stats).unwrap(),
                row["stats"],
                "stats {i}"
            );
        }
    }
}
