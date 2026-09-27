//! Native creation-record identity assignment, including recycled biped slots.
use super::{FilmBipedCreation, IdentityLife, PlayerIndexTable, native_sort};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityCreationRecord {
    pub slot: u32,
    pub generation: u32,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub participant_index: u32,
    pub has_index: bool,
}
impl From<&FilmBipedCreation> for IdentityCreationRecord {
    fn from(c: &FilmBipedCreation) -> Self {
        Self {
            slot: c.creation.slot,
            generation: c.creation.generation.into(),
            timestamp_us: c.source.timestamp_us,
            participant_index: c.creation.participant_index.into(),
            has_index: true,
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityCreationReport {
    pub records: usize,
    pub slots: usize,
    pub direct: usize,
    pub propagated: usize,
    pub index_bot: usize,
    pub recycled: usize,
    pub causes: BTreeMap<usize, String>,
    pub read_indices: BTreeMap<usize, u32>,
}
impl IdentityCreationReport {
    /// Report final unresolved causes; pre-scoreboard bot reads are not subtracted
    /// from the final out-of-table residue.
    pub fn log_refusals(&self, match_id: &str, causes: &super::IdentityUnresolvedCauses) {
        tracing::info!(
            match_id = match_id,
            records = self.records,
            corps = self.slots,
            direct = self.direct,
            propage = self.propagated,
            indexBot = self.index_bot,
            slotsRecycles = self.recycled,
            "rejeu : lien direct corps -> joueur"
        );
        if causes.index_out_of_table > 0 {
            tracing::warn!(
                match_id = match_id,
                vies = causes.index_out_of_table,
                "rejeu : index de participant LU mais absent de la table publiee — vies NON rattachees (verdict I0 : participant que PlayerIndexTable ne nomme pas)"
            );
        }
        if causes.divergent_readings > 0 {
            tracing::warn!(
                match_id = match_id,
                vies = causes.divergent_readings,
                slotsRecycles = self.recycled,
                "rejeu : vie anterieure au premier record de son slot RECYCLE — aucun corps etabli a cet instant, le partage se tait"
            );
        }
        if self.slots > 0 && causes.no_creation_record > 0 {
            tracing::warn!(
                match_id = match_id,
                vies = causes.no_creation_record,
                corps = self.slots,
                "rejeu : vies de bipede sans record de creation sur un film qui en porte — classe attendue VIDE (lot E2 : bijection slots<->records mesuree sur 5 films)"
            );
        }
    }
    pub fn named_lives(&self) -> usize {
        self.direct + self.propagated
    }
}
#[derive(Debug, Clone)]
struct CreationDate {
    timestamp: i64,
    index: u32,
    generation: u32,
}
#[derive(Default)]
struct Body {
    dates: Vec<CreationDate>,
    indices: BTreeSet<u32>,
}
fn bodies_by_slot(records: &[IdentityCreationRecord]) -> BTreeMap<u32, Body> {
    let mut bodies = BTreeMap::<u32, Body>::new();
    for c in records.iter().filter(|c| c.has_index) {
        let b = bodies.entry(c.slot).or_default();
        b.dates.push(CreationDate {
            timestamp: c.timestamp_us as i64,
            index: c.participant_index,
            generation: c.generation,
        });
        b.indices.insert(c.participant_index);
    }
    for b in bodies.values_mut() {
        native_sort::sort_by(&mut b.dates, |a, b| {
            a.timestamp
                .cmp(&b.timestamp)
                .then(a.generation.cmp(&b.generation))
        });
    }
    bodies
}
/// Assign identities before deductions. Lives must retain native per-slot order;
/// the index table must already be injective (see compose_identity_tables).
/// As in the reference, existing XUIDs are overwritten on a successful direct read.
/// Life-end causes and bot labels are untouched.
pub fn name_lives_from_creations(
    lives: &mut [IdentityLife],
    records: &[IdentityCreationRecord],
    indices: &PlayerIndexTable,
    bot_indices: &BTreeSet<i64>,
) -> IdentityCreationReport {
    let mut report = IdentityCreationReport {
        records: records.len(),
        ..Default::default()
    };
    if lives.is_empty() {
        return report;
    }
    let bodies = bodies_by_slot(records);
    report.slots = bodies.len();
    report.recycled = bodies.values().filter(|b| b.dates.len() > 1).count();
    let by_index: BTreeMap<_, _> = indices.by_xuid.iter().map(|(&x, &i)| (i, x)).collect();
    let mut by_slot = BTreeMap::<u32, Vec<usize>>::new();
    for (i, l) in lives.iter().enumerate() {
        by_slot.entry(l.slot).or_default().push(i);
    }
    for (slot, life_indices) in by_slot {
        let Some(body) = bodies.get(&slot) else {
            for i in life_indices {
                report.causes.insert(i, "sans_record".into());
            }
            continue;
        };
        let mut opened = BTreeMap::<usize, u32>::new();
        for d in &body.dates {
            for &i in &life_indices {
                if opened.contains_key(&i) || lives[i].to < d.timestamp {
                    continue;
                }
                opened.insert(i, d.index);
                break;
            }
        }
        for i in life_indices {
            let direct = opened.contains_key(&i);
            let index = opened.get(&i).copied().or_else(|| {
                body.dates
                    .iter()
                    .take_while(|d| d.timestamp <= lives[i].from)
                    .last()
                    .map(|d| d.index)
                    .or_else(|| (body.indices.len() == 1).then(|| *body.indices.first().unwrap()))
            });
            let Some(index) = index else {
                report.causes.insert(i, "lectures_divergentes".into());
                continue;
            };
            let bot = bot_indices.contains(&(index as i64));
            let xuid = by_index.get(&(index as i64));
            if bot || xuid.is_none() {
                if bot {
                    report.index_bot += 1;
                }
                report.causes.insert(i, "index_hors_table".into());
                report.read_indices.insert(i, index);
                continue;
            }
            lives[i].xuid = *xuid.unwrap();
            if direct {
                lives[i].named_by = "biped_creation".into();
                report.direct += 1;
            } else {
                lives[i].named_by = "biped_creation_propagee".into();
                report.propagated += 1;
            }
        }
    }
    report
}
/// Native creation-only slot bridge includes declared bots and missing XUIDs,
/// but omits slots whose creation records disagree about the participant index.
pub fn owners_from_creations(records: &[IdentityCreationRecord]) -> BTreeMap<u32, i64> {
    bodies_by_slot(records)
        .into_iter()
        .filter_map(|(slot, b)| {
            (b.indices.len() == 1).then(|| (slot, *b.indices.first().unwrap() as i64))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        lives: Vec<IdentityLife>,
        records: Vec<IdentityCreationRecord>,
        indices: PlayerIndexTable,
        bots: BTreeSet<i64>,
        output: Vec<IdentityLife>,
        owners: BTreeMap<u32, i64>,
        report: IdentityCreationReport,
        named: usize,
    }
    #[test]
    fn native_creation_identity_assignment() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-creations-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, mut c) in rows.into_iter().enumerate() {
            let report = name_lives_from_creations(&mut c.lives, &c.records, &c.indices, &c.bots);
            assert_eq!(report, c.report, "report {i}");
            assert_eq!(report.named_lives(), c.named, "named {i}");
            assert_eq!(c.lives, c.output, "lives {i}");
            assert_eq!(owners_from_creations(&c.records), c.owners, "owners {i}");
        }
    }
}
