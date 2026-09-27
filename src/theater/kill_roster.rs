//! Kill-source roster pinning before permutation inference.
use super::{FilmBotEntry, KillRosterPins};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Native `BotSuffix`, appended when publishing a bot's roster identity.
pub const BOT_SUFFIX: &str = " [bot]";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillFilmTable {
    pub seats: BTreeMap<i32, String>,
    pub build: String,
    pub occupied: usize,
    pub vacant: usize,
    pub interleaved_vacant: bool,
    /// Empty means read; otherwise the native refusal identifier is retained.
    pub refusal: String,
}
impl KillFilmTable {
    pub fn read(&self) -> bool {
        self.refusal.is_empty() && !self.seats.is_empty()
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillIndexMotif {
    pub names: BTreeMap<i32, String>,
    pub readings: usize,
    pub disagreements: usize,
    pub absent: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillTablePinning {
    pub refusal: String,
    pub build: String,
    pub seats: usize,
    pub pinned: usize,
    pub added_names: usize,
    pub bot_conflict: usize,
    pub duplicate_name: usize,
    pub out_of_range: usize,
    pub inferred: usize,
    pub free_names: usize,
    pub motif_pinned: usize,
    pub motif_agree: usize,
    pub motif_contradict: usize,
    pub motif_duplicate: usize,
    pub motif_readings: usize,
    pub motif_disagreements: usize,
    pub motif_absent: usize,
    pub agree: usize,
    pub contradict: usize,
    pub silent: usize,
}
impl KillTablePinning {
    pub fn assignment_unique(&self) -> bool {
        self.inferred == 0 || (self.inferred == 1 && self.free_names <= 1)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillRosterBot {
    pub slot: i32,
    pub bot_id: u32,
    pub name: String,
}
impl From<&FilmBotEntry> for KillRosterBot {
    fn from(b: &FilmBotEntry) -> Self {
        Self {
            slot: b.slot as i32,
            bot_id: b.bot_id,
            name: b.name.clone(),
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillRoster {
    pub evidence: KillRosterPins,
    pub player_count: usize,
    pub humans: usize,
    pub bots: Vec<KillRosterBot>,
    pub unpinned_bots: Vec<KillRosterBot>,
    pub permutation: Vec<i32>,
    pub bots_succeeded: usize,
    pub table: KillTablePinning,
}
impl KillRoster {
    pub fn name_of(&self, index: i32) -> &str {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.permutation.get(i))
            .and_then(|&p| usize::try_from(p).ok())
            .and_then(|p| self.evidence.names.get(p))
            .map(String::as_str)
            .unwrap_or("?")
    }
    pub fn origin_of(&self, index: i32) -> &'static str {
        if self.evidence.seat_pins.contains(&index) {
            "table_film"
        } else if self.evidence.motif_pins.contains(&index) {
            "motif_xuid"
        } else if self.evidence.is_bot_index(index) {
            "bot_meta"
        } else if self.name_of(index).starts_with('?') {
            "silence"
        } else {
            "inference"
        }
    }
    pub fn free_slots(&self) -> (Vec<i32>, Vec<i32>) {
        let used: BTreeSet<_> = self.evidence.pins.values().copied().collect();
        (
            (0..self.player_count as i32)
                .filter(|i| !self.evidence.pins.contains_key(i))
                .collect(),
            (0..self.evidence.names.len() as i32)
                .filter(|p| !used.contains(p))
                .collect(),
        )
    }
}
/// Preserve native precedence: bot metadata, film table, then XUID motifs.
/// Human count is the original feed roster size, even when later names are added.
pub fn build_kill_roster(
    names: &[String],
    bots: &[KillRosterBot],
    use_bots: bool,
    table: &KillFilmTable,
    motif: &KillIndexMotif,
) -> KillRoster {
    let mut r = KillRoster {
        evidence: KillRosterPins {
            names: names.to_vec(),
            ..Default::default()
        },
        player_count: names.len(),
        humans: names.len(),
        bots: bots.to_vec(),
        ..Default::default()
    };
    if use_bots {
        for b in bots {
            if b.slot < r.humans as i32 || b.slot >= 32 {
                r.unpinned_bots.push(b.clone());
                continue;
            }
            if let Some(&pos) = r.evidence.pins.get(&b.slot) {
                r.evidence.names[pos as usize] = format!("{}{BOT_SUFFIX}", b.name);
                r.bots_succeeded += 1;
                continue;
            }
            let pos = r.evidence.names.len() as i32;
            r.evidence.names.push(format!("{}{BOT_SUFFIX}", b.name));
            r.evidence.pins.insert(b.slot, pos);
            r.player_count = r.player_count.max((b.slot + 1) as usize);
        }
    }
    r.table = KillTablePinning {
        refusal: table.refusal.clone(),
        build: table.build.clone(),
        seats: table.seats.len(),
        ..Default::default()
    };
    if table.read() {
        let mut positions = BTreeMap::new();
        for (i, name) in r.evidence.names.iter().take(r.humans).enumerate() {
            positions.entry(name.clone()).or_insert(i as i32);
        }
        let mut taken: BTreeSet<_> = r.evidence.pins.values().copied().collect();
        for (&index, name) in &table.seats {
            if !(0..32).contains(&index) {
                r.table.out_of_range += 1;
                continue;
            }
            if r.evidence.pins.contains_key(&index) {
                r.table.bot_conflict += 1;
                continue;
            }
            let pos = if let Some(&pos) = positions.get(name) {
                if taken.contains(&pos) {
                    r.table.duplicate_name += 1;
                    continue;
                }
                pos
            } else {
                let pos = r.evidence.names.len() as i32;
                r.evidence.names.push(name.clone());
                positions.insert(name.clone(), pos);
                r.table.added_names += 1;
                pos
            };
            r.evidence.pins.insert(index, pos);
            taken.insert(pos);
            r.evidence.seat_pins.insert(index);
            r.table.pinned += 1;
            r.player_count = r.player_count.max((index + 1) as usize);
        }
    }
    r.table.motif_readings = motif.readings;
    r.table.motif_disagreements = motif.disagreements;
    r.table.motif_absent = motif.absent;
    let mut positions = BTreeMap::new();
    for (i, name) in r.evidence.names.iter().enumerate().rev() {
        positions.insert(name.clone(), i as i32);
    }
    let mut taken: BTreeSet<_> = r.evidence.pins.values().copied().collect();
    for (&index, name) in &motif.names {
        if let Some(&pos) = r.evidence.pins.get(&index) {
            if r.evidence.names.get(pos as usize) == Some(name) {
                r.table.motif_agree += 1;
            } else {
                r.table.motif_contradict += 1;
            }
            continue;
        }
        let pos = if let Some(&pos) = positions.get(name) {
            if taken.contains(&pos) {
                r.table.motif_duplicate += 1;
                continue;
            }
            pos
        } else {
            let pos = r.evidence.names.len() as i32;
            r.evidence.names.push(name.clone());
            positions.insert(name.clone(), pos);
            pos
        };
        r.evidence.pins.insert(index, pos);
        taken.insert(pos);
        r.evidence.motif_pins.insert(index);
        r.table.motif_pinned += 1;
        // The native motif reader supplies bounded indices; unlike film-table
        // pinning this stage has no additional index rejection.
        if index >= 0 {
            r.player_count = r.player_count.max((index + 1) as usize);
        }
    }
    while r.evidence.names.len() < r.player_count {
        r.evidence
            .names
            .push(format!("?{}", r.evidence.names.len()));
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Row {
        names: Vec<String>,
        bots: Vec<KillRosterBot>,
        use_bots: bool,
        table: KillFilmTable,
        motif: KillIndexMotif,
        expected: KillRoster,
        free: Vec<i32>,
        free_names: Vec<i32>,
        resolved: Vec<String>,
        origins: Vec<String>,
    }
    #[test]
    fn native_kill_roster_oracle() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kill-roster-v41.json.zlib")[..])
            .read_to_end(&mut json)
            .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&json).unwrap();
        assert_eq!(rows.len(), 1024);
        for (i, row) in rows.into_iter().enumerate() {
            let mut actual =
                build_kill_roster(&row.names, &row.bots, row.use_bots, &row.table, &row.motif);
            actual.permutation = row.expected.permutation.clone();
            assert_eq!(actual, row.expected, "roster {i}");
            assert_eq!(actual.free_slots(), (row.free, row.free_names), "free {i}");
            for (j, index) in (-1..=actual.player_count as i32).enumerate() {
                assert_eq!(actual.name_of(index), row.resolved[j], "name {i}/{index}");
                assert_eq!(
                    actual.origin_of(index),
                    row.origins[j],
                    "origin {i}/{index}"
                );
            }
        }
    }
}
