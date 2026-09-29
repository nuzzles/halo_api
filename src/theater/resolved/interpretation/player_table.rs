//! Bootstrap player table, ported from LevelUp's player_table*.go.
//! See docs/CREDIT.md. Slot ranks include vacant entries.
use super::{
    FilmIdentity,
    bits::{Bits, Cursor},
};
use std::collections::BTreeMap;

const VACANT_FIXED: usize = 1683;
fn printable(s: &str) -> bool {
    s.len() >= 3 && s.bytes().all(|c| (32..=126).contains(&c))
}
fn vacant(d: &[u8], p: usize) -> bool {
    // Deliberately matches the reference prefix predicate: it does not inspect the
    // personalization body or require the entire vacant record in the buffer.
    if p.checked_add(VACANT_FIXED)
        .is_none_or(|end| end > d.len() * 8)
    {
        return false;
    }
    let Some(mut r) = Cursor::new(d, p) else {
        return false;
    };
    [3, 32, 2, 48, 64, 12, 12, 8]
        .into_iter()
        .chain([64; 13])
        .chain([16, 64, 64, 32])
        .all(|n| r.read(n) == Some(0))
}

fn walk(d: &[u8], mut p: usize, perso: usize) -> Option<(Vec<PlayerTableSlot>, usize)> {
    let mut slots = Vec::new();
    let mut vac = 0;
    for rank in 0..32 {
        if vacant(d, p) {
            vac += 1;
            p += VACANT_FIXED + perso;
            continue;
        }
        let mut s = slot(d, p, perso)?;
        if !printable(&s.gamertag) {
            return None;
        }
        s.film_index = rank;
        p += s.total_bits;
        slots.push(s);
    }
    Some((slots, vac))
}
/// Read a decompressed v41 bootstrap with a previously decoded identity.
/// Unknown builds are reported explicitly; no neighboring-build fallback is used.
pub(crate) fn decode_player_table(d: &[u8], identity: &FilmIdentity) -> PlayerTable {
    let mut out = PlayerTable {
        slots: Vec::new(),
        report: PlayerTableReport {
            build: identity.build.clone(),
            ..Default::default()
        },
        error: None,
    };
    let perso = match identity.build.as_str() {
        "HI_1_12_0" | "HI_1_13_0" => 1852,
        _ => {
            out.error = Some(PlayerTableError::UnknownBuild);
            return out;
        }
    };
    out.report.perso_bytes = perso;
    let end = d.iter().rposition(|b| *b != 0).map_or(0, |n| (n + 1) * 8);
    if identity.body_bit == 0 || end <= identity.body_bit {
        out.error = Some(PlayerTableError::Truncated);
        return out;
    }
    let bits = Bits(d);
    let mut candidates = Vec::new();
    for p in identity.body_bit..=end.saturating_sub(149) {
        if bits.read(p, 3) != Some(4) || bits.read(p + 3, 32) != Some(0) {
            continue;
        }
        let x = bits.read(p + 85, 64).unwrap_or(0);
        if x > 0x0009000000000000
            && x < 0x000a000000000000
            && bits.read(p + 37, 48).is_some_and(|x| x != 0)
        {
            candidates.push(p);
        }
    }
    out.report.candidates_scanned = candidates.len();
    let real: Vec<_> = candidates
        .into_iter()
        .filter_map(|p| slot(d, p, perso * 8))
        .filter(|s| printable(&s.gamertag))
        .collect();
    out.report.candidates_real = real.len();
    let Some(first) = real.first() else {
        out.error = Some(PlayerTableError::NotFound);
        return out;
    };
    let stride = VACANT_FIXED + perso * 8;
    for head in 0..32 {
        let Some(start) = first.bit.checked_sub(head * stride) else {
            break;
        };
        if let Some((slots, vac)) = walk(d, start, perso * 8)
            && slots.len() > out.slots.len()
            && real.iter().all(|r| slots.iter().any(|s| s.bit == r.bit))
        {
            out.slots = slots;
            out.report.vacant = vac;
            out.report.head_vacant = head;
            out.report.first_record_bit = start;
        }
        if !start.checked_sub(stride).is_some_and(|p| vacant(d, p)) {
            break;
        }
    }
    if out.slots.is_empty() {
        out.error = Some(PlayerTableError::NotFound);
        return out;
    }
    let rep = &mut out.report;
    rep.occupied = out.slots.len();
    rep.interleaved_vacant = out
        .slots
        .last()
        .is_some_and(|s| s.film_index != rep.head_vacant + rep.occupied - 1);
    let mut surplus = BTreeMap::<i64, usize>::new();
    for pair in real.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        let after = a.bit + a.total_bits;
        let gap = b.bit as i64 - after as i64;
        *surplus.entry(gap).or_default() += 1;
        if gap == 0 {
            rep.gaps_agree += 1;
        } else if gap > 0
            && (gap as usize).is_multiple_of(stride)
            && (0..gap as usize / stride).all(|k| vacant(d, after + k * stride))
        {
            rep.gaps_vacant += 1;
        } else if out.slots.iter().any(|s| s.bit > a.bit && s.bit < b.bit) {
            rep.gaps_hidden += 1;
        } else {
            rep.gaps_contradict += 1;
        }
    }
    let mut modal = 0;
    for (gap, count) in surplus {
        if count > rep.film_delta_gaps {
            modal = gap;
            rep.film_delta_gaps = count;
        }
    }
    rep.film_delta_bits = modal + rep.profile_delta_bits;
    rep.calibration_agrees = rep.film_delta_gaps > 0 && modal == 0;
    out
}

use super::player_slot::*;
use crate::theater::film::PlayerTableSlot;

use crate::theater::resolved::identity::{PlayerTable, PlayerTableError, PlayerTableReport};
