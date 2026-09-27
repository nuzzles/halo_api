//! Native movement replay phase; capture follows completion of all scan phases.
use super::*;
use serde_json::json;
#[derive(Debug)]
pub struct ReplayMovementScan {
    pub scan: ContextMovementScan,
    pub error: Option<ContextMovementError>,
    pub reads: Option<Vec<MovementStateRead>>,
    /// Full native publication, including counters not stored by the cache format.
    pub stats: MovementStateStats,
    pub diagnostic: StatborgDiagnostic,
    pub published: FactsMovement,
}
impl ReplayMovementScan {
    pub fn apply_to_facts(&self, facts: &mut NativeFilmFacts) {
        facts.movement = self.published.clone();
    }
}
pub enum ReplayMovementObservation<'a> {
    Diagnostic(&'a StatborgDiagnostic),
    States(Option<&'a [MovementStateRead]>),
    Stats(&'a MovementStateStats),
}
impl ReplayMovementObservation<'_> {
    pub fn channel_name(&self) -> Option<&'static str> {
        match self {
            Self::Diagnostic(_) => None,
            Self::States(_) => Some("movementStates"),
            Self::Stats(_) => Some("movementStates.stats"),
        }
    }
}
/// Native balayerEtatsDeMouvement: on failure clear both publications, retaining
/// the raw scanner evidence. Diagnostic delivery precedes the two native channels.
pub fn scan_replay_movement_inputs(
    match_id: &[u8],
    context: &NativeFilmContext<'_>,
    mut observe: impl FnMut(ReplayMovementObservation<'_>),
) -> ReplayMovementScan {
    let (scan, error) = scan_context_movement_states(context);
    let (reads, stats, diagnostic) = if let Some(e) = &error {
        (
            None,
            MovementStateStats::default(),
            StatborgDiagnostic {
                level: "WARN".into(),
                message: "etats de mouvement illisibles — rejeu sans intervalles d etat".into(),
                attributes: vec![
                    ("err".into(), json!(e.to_string())),
                    (
                        "match_id".into(),
                        json!(ReplayByteString(match_id.to_vec()).json_text()),
                    ),
                ],
            },
        )
    } else {
        let s = &scan.stats;
        (
            scan.reads.clone(),
            s.clone(),
            StatborgDiagnostic {
                level: "INFO".into(),
                message: "etats de mouvement lus".into(),
                attributes: vec![
                    ("records".into(), json!(s.records)),
                    ("desyncs".into(), json!(s.desyncs)),
                    ("lectures".into(), json!(s.read)),
                    ("paquets".into(), json!(s.packets)),
                    ("paquetsEvenements".into(), json!(s.event_packets)),
                    (
                        "paquetsEvenementsLocalises".into(),
                        json!(s.event_packets_located),
                    ),
                    (
                        "paquetsEvenementsNonLocalises".into(),
                        json!(s.event_packets_unlocated),
                    ),
                    ("slotNonLie".into(), json!(s.slot_unbound)),
                    ("doublons".into(), json!(s.duplicates)),
                    ("largeursCarte".into(), json!(s.map_widths)),
                    ("absent".into(), json!(s.absent)),
                ],
            },
        )
    };
    observe(ReplayMovementObservation::Diagnostic(&diagnostic));
    observe(ReplayMovementObservation::States(reads.as_deref()));
    observe(ReplayMovementObservation::Stats(&stats));
    let published = FactsMovement {
        states: reads
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|r| FactsMovementState {
                timestamp_us: r.timestamp_us,
                slot: r.slot,
                kind: r.kind.as_bytes().to_vec(),
                on: r.on,
                progress: r.progress,
                chunk: r.chunk,
                packet_index: r.packet_index as i64,
            })
            .collect(),
        stats: FactsMovementStats {
            records: stats.records as i64,
            read: stats.read as i64,
            absent: stats.absent,
            scanned: stats.scanned,
            packets: stats.packets as i64,
            event_packets: stats.event_packets as i64,
            event_packets_located: stats.event_packets_located as i64,
            event_packets_unlocated: stats.event_packets_unlocated as i64,
            desyncs: stats.desyncs as i64,
            slot_unbound: stats.slot_unbound as i64,
            duplicates: stats.duplicates as i64,
            map_widths: stats.map_widths,
        },
    };
    ReplayMovementScan {
        scan,
        error,
        reads,
        stats,
        diagnostic,
        published,
    }
}
