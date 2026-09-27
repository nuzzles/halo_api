//! Publish written translocator jumps with optional paired world coordinates.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayTranslocation {
    pub t: i64,
    pub slot: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fx: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fy: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fz: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tz: Option<f32>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayTranslocationCoverage {
    pub events: usize,
    pub published: usize,
    pub before_origin: usize,
    pub unpublished: usize,
    pub positioned: usize,
}
impl ReplayTranslocationCoverage {
    /// Emit the native publication coverage observation, including empty layers.
    /// Counts describe publication, not additional recorded teleportation events.
    pub fn log(&self) {
        tracing::info!(
            evenements = self.events,
            publiees = self.published,
            positionnees = self.positioned,
            sansPosition = (self.published as i64).wrapping_sub(self.positioned as i64),
            avantOrigine = self.before_origin,
            sansPiste = self.unpublished,
            "rejeu : teleportations du translocateur"
        );
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayTranslocations {
    pub jumps: Vec<ReplayTranslocation>,
    pub coverage: ReplayTranslocationCoverage,
}
/// Preserve event order and retain jumps with unread coordinates. Coordinates
/// are rounded to centimeters and published only when both endpoints were read.
pub fn build_replay_translocations(
    events: &[FilmTranslocatorEvent],
    slots: &BTreeSet<u32>,
    origin: u64,
    step: u64,
) -> ReplayTranslocations {
    publish_translocations(
        events
            .iter()
            .map(|e| (e.source.timestamp_us, e.event.slot, e.event.positions())),
        slots,
        origin,
        step,
    )
}

/// Native-compatible publication, including the reference reader's padded-header
/// outcomes. Consult each retained read's provenance before treating its actor as
/// fully recorded. This is the native replay projection, not a semantic event log.
pub fn build_native_replay_translocations(
    events: &[FilmNativeTranslocatorEvent],
    slots: &BTreeSet<u32>,
    origin: u64,
    step: u64,
) -> ReplayTranslocations {
    publish_translocations(
        events.iter().map(|e| {
            (
                e.source.timestamp_us,
                e.read.event.slot,
                e.read.event.positions(),
            )
        }),
        slots,
        origin,
        step,
    )
}

/// Cache publication uses the recorded presence flag; inactive coordinate
/// payloads remain in the cache and do not invent positioned teleportations.
pub fn build_facts_replay_translocations(
    events: &[FactsTranslocation],
    slots: &BTreeSet<u32>,
    origin: u64,
    step: u64,
) -> ReplayTranslocations {
    publish_translocations(
        events.iter().map(|e| {
            (
                e.timestamp_us,
                e.slot,
                e.has_positions.then_some([e.from, e.to]),
            )
        }),
        slots,
        origin,
        step,
    )
}
fn publish_translocations(
    events: impl ExactSizeIterator<Item = (u64, u32, Option<[[f32; 3]; 2]>)>,
    slots: &BTreeSet<u32>,
    origin: u64,
    step: u64,
) -> ReplayTranslocations {
    let mut out = ReplayTranslocations {
        coverage: ReplayTranslocationCoverage {
            events: events.len(),
            ..Default::default()
        },
        ..Default::default()
    };
    if step == 0 {
        return out;
    }
    for (timestamp_us, slot, positions) in events {
        if timestamp_us < origin {
            out.coverage.before_origin += 1;
            continue;
        }
        if !slots.contains(&slot) {
            out.coverage.unpublished += 1;
            continue;
        }
        let mut jump = ReplayTranslocation {
            t: ((timestamp_us - origin) / step) as i64,
            slot,
            ..Default::default()
        };
        if let Some([from, to]) = positions {
            let round = |v: f32| Some(((f64::from(v) * 100.).round() / 100.) as f32);
            jump.fx = round(from[0]);
            jump.fy = round(from[1]);
            jump.fz = round(from[2]);
            jump.tx = round(to[0]);
            jump.ty = round(to[1]);
            jump.tz = round(to[2]);
            out.coverage.positioned += 1;
        }
        out.jumps.push(jump);
        out.coverage.published += 1;
    }
    out
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_translocations(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> ReplayTranslocations {
    let slots = players
        .players
        .publication
        .tracks
        .iter()
        .map(|t| t.slot)
        .collect();
    build_retained_translocations(film, &slots, players.clock.origin_us, players.clock.step_us)
}

pub(super) fn build_retained_translocations(
    film: &LegacyFilm,
    slots: &BTreeSet<u32>,
    origin: u64,
    step: u64,
) -> ReplayTranslocations {
    if film.native_translocations_scanned || !film.native_translocations.is_empty() {
        build_native_replay_translocations(&film.native_translocations, slots, origin, step)
    } else {
        build_replay_translocations(&film.translocations, slots, origin, step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Raw {
        #[serde(rename = "TimestampUS")]
        time: u64,
        slot: u32,
        has_positions: bool,
        from: [f32; 3],
        to: [f32; 3],
    }
    #[derive(Deserialize)]
    struct Case {
        log: serde_json::Value,
        raw: Vec<Raw>,
        origin: u64,
        step: u64,
        slots: BTreeSet<u32>,
        output: ReplayTranslocations,
    }
    #[test]
    fn native_translocation_publication() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-translocations-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                super::super::log_test_support::capture_log(|| c.output.coverage.log()),
                c.log,
                "log {i}"
            );
            let pos = |world| TeleportPosition {
                start_bit: 0,
                end_bit: 0,
                region: None,
                axis_bits: [0; 3],
                quantized: [0; 3],
                world,
            };
            let raw: Vec<_> = c
                .raw
                .into_iter()
                .map(|r| FilmTranslocatorEvent {
                    source: FilmPacket {
                        chunk_index: 1,
                        packet_type: 0,
                        byte_2: 0,
                        byte_3: 0,
                        payload_offset: 0,
                        payload_size: 0,
                        timestamp_us: r.time,
                    },
                    event: TranslocatorEvent {
                        config: false,
                        slot: r.slot,
                        generation: 0,
                        other_reference_gates: None,
                        effect_present: None,
                        effect: None,
                        from: Some(pos(r.from)),
                        to: r.has_positions.then(|| pos(r.to)),
                        end_bit: 0,
                        stop: if r.has_positions {
                            TranslocatorStop::PositionsRead
                        } else {
                            TranslocatorStop::Truncated
                        },
                    },
                })
                .collect();
            let native: Vec<_> = raw
                .iter()
                .enumerate()
                .map(|(packet_index, r)| FilmNativeTranslocatorEvent {
                    source: r.source,
                    packet_index,
                    read: NativeTranslocatorEvent {
                        event: r.event.clone(),
                        source_bits: 0,
                        padded_bits: 0,
                    },
                })
                .collect();
            assert_eq!(
                build_native_replay_translocations(&native, &c.slots, c.origin, c.step),
                c.output,
                "native records {i}"
            );
            assert_eq!(
                build_replay_translocations(&raw, &c.slots, c.origin, c.step),
                c.output,
                "case {i}"
            );
        }
    }
}
