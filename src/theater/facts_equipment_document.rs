//! Cached equipment/movement pass, before the document coverage envelope.
use super::*;
use std::collections::BTreeSet;

pub struct FactsReplayEquipmentContext<'a> {
    pub identity: &'a ReplayIdentityState,
    pub clock: IdentityClock,
    /// The assembler's effective interval; not the value on a seeded document.
    pub interval_ms: i64,
}
#[derive(Default)]
pub struct FactsReplayEquipmentInput<'a> {
    pub positions: &'a [FactsBipedPosition],
    pub camo: &'a [FactsCamoState],
    pub movement: &'a [FactsMovementState],
    pub movement_stats: FactsMovementStats,
    pub kills: &'a [ReplayEquipmentKill],
    pub kills_read: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FactsReplayEquipmentReport {
    pub closed_by_death: BTreeSet<usize>,
    pub camo_non_binary: usize,
    pub stances: ReplayStanceCoverage<ReplayByteString>,
    pub kills_read: bool,
    /// Ordered native messages/attributes for the composer's logging sink.
    /// Raw movement kind identities remain in `stances.by_kind` and the document;
    /// this JSON-shaped diagnostic projection follows native JSON replacement.
    pub diagnostics: Vec<StatborgDiagnostic>,
}
/// Resolve read-death boundaries against current published tracks, construct
/// equipment and movement intervals, then credit kills using temporal occupants.
/// Existing layers are replaced, but coverage is not attached until the next pass.
/// Caller kill evidence is explicit; this never applies a cache's Kills section.
pub fn assemble_facts_replay_equipment(
    doc: &mut ReplayDocument,
    input: FactsReplayEquipmentInput<'_>,
    ctx: FactsReplayEquipmentContext<'_>,
) -> FactsReplayEquipmentReport {
    let tracks = doc.content.tracks.as_deref().unwrap_or_default();
    let identities: Vec<_> = tracks.iter().map(ReplayTrack::identity).collect();
    let closed_by_death =
        ctx.identity
            .death_closed_tracks(&identities, ctx.clock.origin_us, ctx.clock.step_us);
    let equipment = build_facts_replay_equipment_episodes(
        input.positions,
        input.camo,
        ctx.clock.origin_us,
        ctx.clock.step_us,
        tracks,
        &closed_by_death,
    );
    doc.content.equipment_episodes = equipment.episodes;
    let mut diagnostics = Vec::new();
    if equipment.non_binary > 0 {
        diagnostics.push(StatborgDiagnostic {
            level: "WARN".into(),
            message: "rejeu : lectures camo NON BINAIRES ignorees — l'interrupteur mesure ne connait que 0 et 4095".into(),
            attributes: vec![("lectures".into(), serde_json::json!(equipment.non_binary))],
        });
    }
    let stances = build_facts_replay_stances(
        input.movement,
        &input.movement_stats,
        ctx.clock.origin_us,
        ctx.clock.step_us,
        tracks,
        &closed_by_death,
    );
    doc.content.stances = stances.stances;
    let c = &stances.coverage;
    diagnostics.push(StatborgDiagnostic {
        level: "INFO".into(),
        message: "rejeu : etats de mouvement".into(),
        attributes: vec![
            ("balaye".into(), serde_json::json!(c.scanned)),
            ("absent".into(), serde_json::json!(c.absent)),
            ("records".into(), serde_json::json!(c.records)),
            ("desyncs".into(), serde_json::json!(c.desyncs)),
            ("lectures".into(), serde_json::json!(c.reads)),
            ("intervalles".into(), serde_json::json!(c.intervals)),
            (
                "parGenre".into(),
                if c.by_kind.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::json!(c.by_kind)
                },
            ),
            ("vies".into(), serde_json::json!(c.lives)),
            ("viesPubliees".into(), serde_json::json!(c.tracks_total)),
            ("ecartees".into(), serde_json::json!(c.dropped)),
            ("largeursCarte".into(), serde_json::json!(c.map_widths)),
        ],
    });
    let occupant = |slot, frame| replay_occupant_at_frame(ctx.identity, ctx.clock, slot, frame);
    let kills_read = attach_replay_equipment_kills(
        &mut doc.content.equipment_episodes,
        input.kills,
        input.kills_read,
        Some(&occupant),
        doc.content.origin_ms,
        ctx.interval_ms,
    );
    FactsReplayEquipmentReport {
        closed_by_death,
        camo_non_binary: equipment.non_binary,
        stances: stances.coverage,
        kills_read,
        diagnostics,
    }
}
