//! Attribute transmitted equipment charges using the ability rank in the same life.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub struct ReplayAbilityRankIndex<'a> {
    ranks: BTreeMap<u32, Vec<(u64, i64)>>,
    lives: BTreeMap<u32, Vec<&'a IdentityLife>>,
}
impl<'a> ReplayAbilityRankIndex<'a> {
    pub fn new(ranks: &'a [BipedAbilityEmission], lives: &'a [IdentityLife]) -> Self {
        Self::from_values(
            ranks.iter().filter_map(|r| {
                r.rank
                    .map(|rank| (r.slot, r.source.timestamp_us, i64::from(rank)))
            }),
            lives,
        )
    }
    pub fn from_facts(ranks: &[FactsAbilityRank], lives: &'a [IdentityLife]) -> Self {
        Self::from_values(
            ranks.iter().map(|r| (r.slot, r.timestamp_us, r.rank)),
            lives,
        )
    }
    fn from_values(
        ranks: impl Iterator<Item = (u32, u64, i64)>,
        lives: &'a [IdentityLife],
    ) -> Self {
        let mut out = Self {
            ranks: BTreeMap::new(),
            lives: BTreeMap::new(),
        };
        for (slot, time, rank) in ranks {
            out.ranks.entry(slot).or_default().push((time, rank));
        }
        for life in lives {
            out.lives.entry(life.slot).or_default().push(life);
        }
        out
    }
    /// First supplied life within the native five-second tolerance. The latest
    /// prior rank at/after its start wins; equal timestamps take the last reading.
    /// The selected rank is not additionally bounded by the life end.
    pub fn rank_in_life(&self, slot: u32, at: u64) -> Option<i64> {
        let span = self.lives.get(&slot)?.iter().find(|l| {
            (at as i64).wrapping_add(5_000_000) >= l.from
                && (at as i64) <= l.to.wrapping_add(5_000_000)
        })?;
        let mut best = None;
        let mut best_time = 0;
        for &(time, rank) in self.ranks.get(&slot)? {
            if (time as i64) < span.from || time > at {
                continue;
            }
            if best.is_none() || time >= best_time {
                best = Some(rank);
                best_time = time;
            }
        }
        best
    }
}
pub struct ReplayAbilityContext<'a, R = BipedAbilityEmission> {
    pub ranks: &'a [R],
    pub lives: &'a [IdentityLife],
    /// A resolved palette's stable equipment family names. None is unclassified.
    pub palette: Option<&'a BTreeMap<i64, String>>,
    pub measured_families: &'a BTreeSet<String>,
    pub published_slots: &'a BTreeSet<u32>,
    pub origin_us: u64,
    pub step_us: u64,
}
pub type FactsReplayAbilityContext<'a> = ReplayAbilityContext<'a, FactsAbilityRank>;
pub(super) struct AbilityResolutionContext<'a> {
    pub index: ReplayAbilityRankIndex<'a>,
    pub has_lives: bool,
    pub palette: Option<&'a BTreeMap<i64, String>>,
    pub measured_families: &'a BTreeSet<String>,
    pub published_slots: &'a BTreeSet<u32>,
    pub origin_us: u64,
    pub step_us: u64,
}
impl<'a, R> ReplayAbilityContext<'a, R> {
    pub(super) fn with_index(
        self,
        index: ReplayAbilityRankIndex<'a>,
    ) -> AbilityResolutionContext<'a> {
        AbilityResolutionContext {
            index,
            has_lives: !self.lives.is_empty(),
            palette: self.palette,
            measured_families: self.measured_families,
            published_slots: self.published_slots,
            origin_us: self.origin_us,
            step_us: self.step_us,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityCharge {
    pub t: i64,
    pub slot: u32,
    pub family: String,
    pub charges: i64,
}
fn is_false(v: &bool) -> bool {
    !v
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayAbilityChargeCoverage {
    pub reads: usize,
    pub published: usize,
    pub before_origin: usize,
    pub unpublished: usize,
    pub no_identity: usize,
    pub other_family: usize,
    pub no_resolver: usize,
    #[serde(default, skip_serializing_if = "is_false")]
    pub component_absent: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAbilityChargePublication {
    pub charges: Vec<ReplayAbilityCharge>,
    pub coverage: ReplayAbilityChargeCoverage,
}
/// Publish every measured reading in input order, including zero. This does not
/// infer usage counts, fill missing charges, or clamp readings to a final frame.
pub fn build_replay_ability_charges(
    reads: &[AbilityCharge],
    stats: &AbilityChargeStats,
    context: ReplayAbilityContext<'_>,
) -> ReplayAbilityChargePublication {
    let index = ReplayAbilityRankIndex::new(context.ranks, context.lives);
    build_charge_values(
        reads
            .iter()
            .map(|r| (r.source.timestamp_us, r.slot, i64::from(r.charges))),
        stats.absent,
        context.with_index(index),
    )
}
/// Publish signed native cache charge counts without imposing wire-field limits.
pub fn build_facts_replay_ability_charges(
    reads: &[FactsAbilityCharge],
    stats: &FactsAbilityChargeStats,
    context: FactsReplayAbilityContext<'_>,
) -> ReplayAbilityChargePublication {
    let index = ReplayAbilityRankIndex::from_facts(context.ranks, context.lives);
    build_charge_values(
        reads.iter().map(|r| (r.timestamp_us, r.slot, r.charges)),
        stats.absent,
        context.with_index(index),
    )
}
fn build_charge_values(
    reads: impl ExactSizeIterator<Item = (u64, u32, i64)>,
    absent: bool,
    context: AbilityResolutionContext<'_>,
) -> ReplayAbilityChargePublication {
    let mut out = ReplayAbilityChargePublication::default();
    out.coverage.reads = reads.len();
    out.coverage.component_absent = absent;
    if reads.len() == 0 || context.step_us == 0 {
        return out;
    }
    let index = &context.index;
    let resolvable =
        context.palette.is_some() && !context.measured_families.is_empty() && context.has_lives;
    for (timestamp_us, slot, charges) in reads {
        if timestamp_us < context.origin_us {
            out.coverage.before_origin += 1;
            continue;
        }
        if !context.published_slots.contains(&slot) {
            out.coverage.unpublished += 1;
            continue;
        }
        if !resolvable {
            out.coverage.no_resolver += 1;
            continue;
        }
        let Some(rank) = index.rank_in_life(slot, timestamp_us) else {
            out.coverage.no_identity += 1;
            continue;
        };
        let family = context
            .palette
            .and_then(|p| p.get(&rank))
            .map_or("", String::as_str);
        if family.is_empty() || !context.measured_families.contains(family) {
            out.coverage.other_family += 1;
            continue;
        }
        out.charges.push(ReplayAbilityCharge {
            t: ((timestamp_us - context.origin_us) / context.step_us) as i64,
            slot,
            family: family.into(),
            charges,
        });
    }
    out.coverage.published = out.charges.len();
    out
}

impl ReplayAbilityChargeCoverage {
    /// Emit the pinned reference parser's coverage observation.
    pub fn log(&self) {
        tracing::info!(
            lectures = self.reads,
            publiees = self.published,
            sansIdentite = self.no_identity,
            familleNonMesuree = self.other_family,
            attributionIndisponible = self.no_resolver,
            avantOrigine = self.before_origin,
            sansPiste = self.unpublished,
            composantAbsent = self.component_absent,
            "rejeu : charges d equipement"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::io::Read;
    fn source(time: u64) -> FilmPacket {
        FilmPacket {
            chunk_index: 0,
            packet_type: 0,
            byte_2: 0,
            byte_3: 0,
            payload_offset: 0,
            payload_size: 0,
            timestamp_us: time,
        }
    }
    #[test]
    fn native_charge_publication_and_life_ranks() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-ability-charges-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            let lives: Vec<_> = c["lives"]
                .as_array()
                .unwrap()
                .iter()
                .map(|l| IdentityLife {
                    slot: l["slot"].as_u64().unwrap() as u32,
                    from: l["from"].as_i64().unwrap(),
                    to: l["to"].as_i64().unwrap(),
                    ..Default::default()
                })
                .collect();
            let ranks: Vec<_> = c["ranks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| BipedAbilityEmission {
                    packet_index: None,
                    source: source(r["time"].as_u64().unwrap()),
                    slot: r["slot"].as_u64().unwrap() as u32,
                    counter: 0,
                    rank: Some(r["rank"].as_u64().unwrap() as u8),
                })
                .collect();
            let reads: Vec<_> = c["reads"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| AbilityCharge {
                    packet_index: None,
                    source: source(r["time"].as_u64().unwrap()),
                    slot: r["slot"].as_u64().unwrap() as u32,
                    emplacement: 0,
                    charges: r["charges"].as_u64().unwrap() as u8,
                    low: 0,
                })
                .collect();
            let palette: Option<BTreeMap<i64, String>> =
                serde_json::from_value(c["palette"].clone()).unwrap();
            let measured = serde_json::from_value(c["measured"].clone()).unwrap();
            let published = serde_json::from_value(c["published"].clone()).unwrap();
            let stats = AbilityChargeStats {
                absent: c["absent"].as_bool().unwrap(),
                ..Default::default()
            };
            let out = build_replay_ability_charges(
                &reads,
                &stats,
                ReplayAbilityContext {
                    ranks: &ranks,
                    lives: &lives,
                    palette: palette.as_ref(),
                    measured_families: &measured,
                    published_slots: &published,
                    origin_us: c["origin"].as_u64().unwrap(),
                    step_us: c["step"].as_u64().unwrap(),
                },
            );
            assert_eq!(
                out,
                serde_json::from_value::<ReplayAbilityChargePublication>(c["output"].clone())
                    .unwrap(),
                "charges {i}"
            );
            let impulses: Vec<_> = reads
                .iter()
                .enumerate()
                .map(|(i, r)| AbilityImpulse {
                    packet_index: None,
                    source: r.source,
                    slot: r.slot,
                    predicted: i % 2 == 0,
                })
                .collect();
            assert_eq!(
                fold_replay_ability_impulses(&impulses),
                serde_json::from_value::<Vec<ReplayAbilityImpulseEpisode>>(c["folded"].clone())
                    .unwrap(),
                "folded impulses {i}"
            );
            let stats: AbilityImpulseStats =
                serde_json::from_value(c["impulse_stats"].clone()).unwrap();
            let impulses = build_replay_ability_impulses(
                &impulses,
                &stats,
                ReplayAbilityContext {
                    ranks: &ranks,
                    lives: &lives,
                    palette: palette.as_ref(),
                    measured_families: &measured,
                    published_slots: &published,
                    origin_us: c["origin"].as_u64().unwrap(),
                    step_us: c["step"].as_u64().unwrap(),
                },
            );
            assert_eq!(
                impulses,
                serde_json::from_value::<ReplayAbilityImpulsePublication>(
                    c["impulse_output"].clone()
                )
                .unwrap(),
                "impulses {i}"
            );
            let inventory: Vec<KeyframeInventory> =
                serde_json::from_value(c["inventory"].clone()).unwrap();
            let palettes: Vec<ReplayAbilityPalette> =
                serde_json::from_value(c["palettes"].clone()).unwrap();
            let ability_reads = build_replay_ability_reads(
                &ranks,
                &inventory,
                c["origin"].as_u64().unwrap(),
                std::num::NonZeroU64::new(c["step"].as_u64().unwrap().max(1)).unwrap(),
            );
            assert_eq!(
                ability_reads,
                serde_json::from_value::<Vec<ReplayAbilityRead>>(c["raw_abilities"].clone())
                    .unwrap(),
                "ability projection {i}"
            );
            let ability_publication =
                publish_replay_abilities(ability_reads, &published, &palettes);
            assert_eq!(
                ability_publication,
                serde_json::from_value::<ReplayAbilityPublication>(c["ability_output"].clone())
                    .unwrap(),
                "ability publication {i}"
            );
            let logs = super::super::log_test_support::capture_logs(|| {
                ability_publication.coverage.log();
                super::super::replay_abilities_film::log_replay_ability_scans(
                    stats.scanned.then_some(&impulses.coverage),
                    reads.len(),
                    c["charge_scanned"]
                        .as_bool()
                        .unwrap()
                        .then_some(&out.coverage),
                    reads.len(),
                );
            });
            assert_eq!(
                serde_json::to_value(logs).unwrap(),
                c["logs"],
                "ability logs {i}"
            );
            let palette_log = super::super::log_test_support::capture_log(|| {
                super::super::replay_abilities::log_replay_ability_palette(
                    ability_publication.palette.as_ref(),
                    ability_publication.reads.len(),
                    ability_publication.labels.len(),
                );
            });
            assert_eq!(palette_log, c["palette_log"], "palette log {i}");
            for probe in c["classifications"].as_array().unwrap() {
                let reads: Vec<ReplayAbilityRead> =
                    serde_json::from_value(probe["reads"].clone()).unwrap();
                assert_eq!(
                    classify_replay_ability_palette(&reads, &palettes).map(|p| p.id.as_str()),
                    probe["selected"].as_str(),
                    "palette classification {i}"
                );
            }
            let index = ReplayAbilityRankIndex::new(&ranks, &lives);
            for probe in c["probes"].as_array().unwrap() {
                assert_eq!(
                    index.rank_in_life(
                        probe["slot"].as_u64().unwrap() as u32,
                        probe["at"].as_u64().unwrap()
                    ),
                    serde_json::from_value::<Option<i64>>(probe["rank"].clone()).unwrap(),
                    "rank {i}"
                );
            }
        }
    }
}
